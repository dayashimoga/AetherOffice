use crate::ast::*;
use crate::error::MacroError;
use crate::sandbox::MacroSandbox;
use crate::security::Capability;
use crate::virtual_dom::{RangeAddress, VbaValue, VirtualOfficeDom};
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum ScopeValue {
    Val(VbaValue),
    Range(RangeAddress),
}

pub struct VbaEvaluator<'a> {
    pub dom: &'a mut VirtualOfficeDom,
    pub sandbox: MacroSandbox,
    variables: HashMap<String, ScopeValue>,
    with_context: Option<RangeAddress>,
}

impl<'a> VbaEvaluator<'a> {
    pub fn new(dom: &'a mut VirtualOfficeDom) -> Self {
        Self::with_sandbox(dom, MacroSandbox::default())
    }

    pub fn with_sandbox(dom: &'a mut VirtualOfficeDom, sandbox: MacroSandbox) -> Self {
        Self {
            dom,
            sandbox,
            variables: HashMap::new(),
            with_context: None,
        }
    }

    pub fn execute_program(&mut self, program: &VbaProgram) -> Result<(), MacroError> {
        for stmt in &program.top_level_statements {
            self.execute_statement(stmt)?;
        }
        Ok(())
    }

    pub fn execute_routine(
        &mut self,
        program: &VbaProgram,
        name: &str,
        args: &[VbaValue],
    ) -> Result<VbaValue, MacroError> {
        let routine = program
            .routines
            .iter()
            .find(|r| r.name.eq_ignore_ascii_case(name))
            .ok_or_else(|| MacroError::EvalError(format!("Routine '{name}' not found")))?;

        // Bind parameters
        for (i, param) in routine.params.iter().enumerate() {
            let val = args.get(i).cloned().unwrap_or(VbaValue::Empty);
            self.variables
                .insert(param.name.to_ascii_lowercase(), ScopeValue::Val(val));
        }

        for stmt in &routine.body {
            self.execute_statement(stmt)?;
        }

        // Return value of Function is variable matching function name
        if routine.kind == RoutineKind::Function {
            let key = routine.name.to_ascii_lowercase();
            if let Some(ScopeValue::Val(v)) = self.variables.get(&key) {
                return Ok(v.clone());
            }
        }

        Ok(VbaValue::Empty)
    }

    pub fn execute_statement(&mut self, stmt: &VbaStatement) -> Result<(), MacroError> {
        self.sandbox.consume_gas(1)?;
        match stmt {
            VbaStatement::Dim { name, .. } => {
                self.variables
                    .insert(name.to_ascii_lowercase(), ScopeValue::Val(VbaValue::Empty));
                Ok(())
            }

            VbaStatement::Set { target, value } => {
                if let VbaExpr::Variable(var_name) = target {
                    if let VbaExpr::IndexOrCall { callee, args } = value {
                        if let VbaExpr::Variable(name) = callee.as_ref() {
                            if name.eq_ignore_ascii_case("range") && !args.is_empty() {
                                let a1 = self.eval_expr(&args[0])?.to_display_string();
                                let range = RangeAddress::from_a1(&a1)?;
                                self.variables.insert(
                                    var_name.to_ascii_lowercase(),
                                    ScopeValue::Range(range),
                                );
                                return Ok(());
                            }
                        }
                    }
                }

                let eval_val = self.eval_expr(value)?;
                if let VbaExpr::Variable(var_name) = target {
                    let key = var_name.to_ascii_lowercase();
                    self.variables.insert(key, ScopeValue::Val(eval_val));
                    Ok(())
                } else if let VbaExpr::MemberAccess { object, member } = target {
                    self.set_member_value(object, member, eval_val)
                } else {
                    Err(MacroError::EvalError(format!(
                        "Invalid Set assignment target: {target:?}"
                    )))
                }
            }

            VbaStatement::Assign { target, value } => {
                let eval_val = self.eval_expr(value)?;

                match target {
                    VbaExpr::Variable(var_name) => {
                        let key = var_name.to_ascii_lowercase();
                        self.variables.insert(key, ScopeValue::Val(eval_val));
                        Ok(())
                    }
                    VbaExpr::MemberAccess { object, member } => {
                        self.set_member_value(object, member, eval_val)
                    }
                    VbaExpr::ImplicitMember(member) => {
                        if let Some(range) = self.with_context {
                            self.set_range_member(range, member, eval_val)
                        } else {
                            Err(MacroError::EvalError(
                                "Implicit member access without With context".to_string(),
                            ))
                        }
                    }
                    VbaExpr::IndexOrCall { callee, args } => {
                        // e.g. Cells(r, c) = val
                        if let VbaExpr::Variable(name) = callee.as_ref() {
                            if name.eq_ignore_ascii_case("cells") && args.len() == 2 {
                                let r = self.eval_expr(&args[0])?.as_i64().unwrap_or(1) as u32;
                                let c = self.eval_expr(&args[1])?.as_i64().unwrap_or(1) as u32;
                                let sheet = self.dom.active_sheet_mut()?;
                                sheet.set_value(r, c, eval_val);
                                return Ok(());
                            }
                        }
                        Err(MacroError::EvalError(format!(
                            "Unsupported assignment target: {target:?}"
                        )))
                    }
                    _ => Err(MacroError::EvalError(format!(
                        "Invalid assignment target: {target:?}"
                    ))),
                }
            }

            VbaStatement::Call { target, args } => {
                match target {
                    VbaExpr::MemberAccess { object, member } => {
                        if let VbaExpr::IndexOrCall {
                            callee,
                            args: c_args,
                        } = object.as_ref()
                        {
                            if let VbaExpr::Variable(name) = callee.as_ref() {
                                if name.eq_ignore_ascii_case("range") && !c_args.is_empty() {
                                    let a1 = self.eval_expr(&c_args[0])?.to_display_string();
                                    let range = RangeAddress::from_a1(&a1)?;
                                    return self.execute_range_method(range, member, args);
                                }
                            }
                        }
                        Ok(())
                    }
                    VbaExpr::ImplicitMember(member) => {
                        if let Some(range) = self.with_context {
                            self.execute_range_method(range, member, args)
                        } else {
                            Err(MacroError::EvalError(
                                "Implicit call without With context".to_string(),
                            ))
                        }
                    }
                    VbaExpr::Variable(name) => {
                        let lower = name.to_ascii_lowercase();
                        if lower == "shell" {
                            self.sandbox
                                .check_capability(Capability::SystemCommand, "Shell")?;
                            Ok(())
                        } else if lower == "msgbox" {
                            // Mock MsgBox execution
                            if let Some(first_arg) = args.first() {
                                let _msg = self.eval_expr(first_arg)?;
                            }
                            Ok(())
                        } else {
                            Ok(())
                        }
                    }
                    _ => Ok(()),
                }
            }

            VbaStatement::If {
                condition,
                then_branch,
                else_ifs,
                else_branch,
            } => {
                if self.eval_expr(condition)?.as_bool() {
                    for s in then_branch {
                        self.execute_statement(s)?;
                    }
                    return Ok(());
                }

                for (ei_cond, ei_branch) in else_ifs {
                    if self.eval_expr(ei_cond)?.as_bool() {
                        for s in ei_branch {
                            self.execute_statement(s)?;
                        }
                        return Ok(());
                    }
                }

                if let Some(else_stmts) = else_branch {
                    for s in else_stmts {
                        self.execute_statement(s)?;
                    }
                }

                Ok(())
            }

            VbaStatement::For {
                counter,
                start,
                end,
                step,
                body,
            } => {
                let start_val = self.eval_expr(start)?.as_i64().unwrap_or(0);
                let end_val = self.eval_expr(end)?.as_i64().unwrap_or(0);
                let step_val = match step {
                    Some(s) => self.eval_expr(s)?.as_i64().unwrap_or(1),
                    None => 1,
                };

                let mut current = start_val;
                let counter_key = counter.to_ascii_lowercase();

                loop {
                    if (step_val > 0 && current > end_val) || (step_val < 0 && current < end_val) {
                        break;
                    }

                    self.sandbox.consume_gas(1)?;
                    self.variables.insert(
                        counter_key.clone(),
                        ScopeValue::Val(VbaValue::Integer(current)),
                    );

                    for s in body {
                        self.execute_statement(s)?;
                    }

                    current += step_val;
                }

                Ok(())
            }

            VbaStatement::DoLoop {
                condition,
                is_while,
                is_post_test,
                body,
            } => {
                let check_cond = |eval: &mut Self| -> Result<bool, MacroError> {
                    if let Some(cond) = condition {
                        let b = eval.eval_expr(cond)?.as_bool();
                        if *is_while {
                            Ok(b)
                        } else {
                            Ok(!b)
                        }
                    } else {
                        Ok(true)
                    }
                };

                if !is_post_test && !check_cond(self)? {
                    return Ok(());
                }

                loop {
                    self.sandbox.consume_gas(1)?;
                    for s in body {
                        self.execute_statement(s)?;
                    }

                    if !check_cond(self)? {
                        break;
                    }
                }

                Ok(())
            }

            VbaStatement::WhileWend { condition, body } => {
                loop {
                    self.sandbox.consume_gas(1)?;
                    if !self.eval_expr(condition)?.as_bool() {
                        break;
                    }
                    for s in body {
                        self.execute_statement(s)?;
                    }
                }
                Ok(())
            }

            VbaStatement::With { object, body } => {
                let saved_with = self.with_context;

                if let VbaExpr::IndexOrCall { callee, args } = object {
                    if let VbaExpr::Variable(name) = callee.as_ref() {
                        if name.eq_ignore_ascii_case("range") && !args.is_empty() {
                            let a1 = self.eval_expr(&args[0])?.to_display_string();
                            let range = RangeAddress::from_a1(&a1)?;
                            self.with_context = Some(range);
                        }
                    }
                }

                for s in body {
                    self.execute_statement(s)?;
                }

                self.with_context = saved_with;
                Ok(())
            }

            VbaStatement::Exit(_) => Ok(()),

            VbaStatement::Expr(expr) => {
                self.eval_expr(expr)?;
                Ok(())
            }
        }
    }

    fn set_member_value(
        &mut self,
        object: &VbaExpr,
        member: &str,
        val: VbaValue,
    ) -> Result<(), MacroError> {
        // e.g. Range("A1").Value = 42
        if let VbaExpr::IndexOrCall { callee, args } = object {
            if let VbaExpr::Variable(name) = callee.as_ref() {
                if name.eq_ignore_ascii_case("range") && !args.is_empty() {
                    let a1 = self.eval_expr(&args[0])?.to_display_string();
                    let range = RangeAddress::from_a1(&a1)?;
                    return self.set_range_member(range, member, val);
                }
            }
        }

        // e.g. Range("A1").Interior.Color = 255
        if let VbaExpr::MemberAccess {
            object: inner_obj,
            member: inner_mem,
        } = object
        {
            if inner_mem.eq_ignore_ascii_case("interior") && member.eq_ignore_ascii_case("color") {
                if let VbaExpr::IndexOrCall { callee, args } = inner_obj.as_ref() {
                    if let VbaExpr::Variable(name) = callee.as_ref() {
                        if name.eq_ignore_ascii_case("range") && !args.is_empty() {
                            let a1 = self.eval_expr(&args[0])?.to_display_string();
                            let range = RangeAddress::from_a1(&a1)?;
                            let color = val.as_i64().unwrap_or(0) as u32;
                            let sheet = self.dom.active_sheet_mut()?;
                            sheet.set_range_interior_color(range, color);
                            return Ok(());
                        }
                    }
                }
            }
        }

        // Application properties
        if let VbaExpr::Variable(name) = object {
            let key = name.to_ascii_lowercase();
            if let Some(ScopeValue::Range(r)) = self.variables.get(&key) {
                let range = *r;
                return self.set_range_member(range, member, val);
            }

            if name.eq_ignore_ascii_case("application") {
                if member.eq_ignore_ascii_case("screenupdating") {
                    self.dom.application.screen_updating = val.as_bool();
                    return Ok(());
                } else if member.eq_ignore_ascii_case("calculation") {
                    self.dom.application.calculation = val.to_display_string();
                    return Ok(());
                }
            }
        }

        Err(MacroError::EvalError(format!(
            "Unsupported member assignment: {object:?}.{member}"
        )))
    }

    fn set_range_member(
        &mut self,
        range: RangeAddress,
        member: &str,
        val: VbaValue,
    ) -> Result<(), MacroError> {
        let sheet = self.dom.active_sheet_mut()?;
        match member.to_ascii_lowercase().as_str() {
            "value" => {
                sheet.set_range_value(range, val);
                Ok(())
            }
            "formula" => {
                sheet.set_formula(range.start_row, range.start_col, val.to_display_string());
                Ok(())
            }
            "color" => {
                let color = val.as_i64().unwrap_or(0) as u32;
                sheet.set_range_interior_color(range, color);
                Ok(())
            }
            _ => Err(MacroError::EvalError(format!(
                "Unknown Range property '{member}'"
            ))),
        }
    }

    fn execute_range_method(
        &mut self,
        range: RangeAddress,
        method: &str,
        args: &[VbaExpr],
    ) -> Result<(), MacroError> {
        let dest_range_opt = if method.eq_ignore_ascii_case("copy") {
            if let Some(VbaExpr::IndexOrCall {
                callee,
                args: d_args,
            }) = args.first()
            {
                if let VbaExpr::Variable(name) = callee.as_ref() {
                    if name.eq_ignore_ascii_case("range") && !d_args.is_empty() {
                        let a1 = self.eval_expr(&d_args[0])?.to_display_string();
                        Some(RangeAddress::from_a1(&a1)?)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let sheet = self.dom.active_sheet_mut()?;
        match method.to_ascii_lowercase().as_str() {
            "clear" => {
                sheet.clear_range(range);
                Ok(())
            }
            "select" => {
                sheet.selected_range = range;
                Ok(())
            }
            "copy" => {
                if let Some(dest_range) = dest_range_opt {
                    sheet.copy_range(range, dest_range.start_row, dest_range.start_col);
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn eval_expr(&mut self, expr: &VbaExpr) -> Result<VbaValue, MacroError> {
        self.sandbox.consume_gas(1)?;
        match expr {
            VbaExpr::Literal(lit) => match lit {
                VbaLiteral::Empty => Ok(VbaValue::Empty),
                VbaLiteral::Null => Ok(VbaValue::Null),
                VbaLiteral::Nothing => Ok(VbaValue::Nothing),
                VbaLiteral::Bool(b) => Ok(VbaValue::Boolean(*b)),
                VbaLiteral::Int(i) => Ok(VbaValue::Integer(*i)),
                VbaLiteral::Float(f) => Ok(VbaValue::Double(*f)),
                VbaLiteral::Str(s) => Ok(VbaValue::String(s.clone())),
            },

            VbaExpr::Variable(name) => {
                let key = name.to_ascii_lowercase();
                if let Some(ScopeValue::Val(v)) = self.variables.get(&key) {
                    return Ok(v.clone());
                }

                // Global constants / keywords
                match key.as_str() {
                    "vbokonly" => Ok(VbaValue::Integer(0)),
                    "vbcritical" => Ok(VbaValue::Integer(16)),
                    "vbinformation" => Ok(VbaValue::Integer(64)),
                    "xlautomatic" => Ok(VbaValue::Integer(-4105)),
                    "xlmanual" => Ok(VbaValue::Integer(-4135)),
                    _ => Ok(VbaValue::Empty),
                }
            }

            VbaExpr::UnaryOp { op, expr } => {
                let val = self.eval_expr(expr)?;
                match op {
                    UnaryOp::Neg => {
                        if let Some(i) = val.as_i64() {
                            Ok(VbaValue::Integer(-i))
                        } else if let Some(f) = val.as_f64() {
                            Ok(VbaValue::Double(-f))
                        } else {
                            Ok(VbaValue::Integer(0))
                        }
                    }
                    UnaryOp::Not => Ok(VbaValue::Boolean(!val.as_bool())),
                }
            }

            VbaExpr::BinaryOp { op, left, right } => {
                let l_val = self.eval_expr(left)?;
                let r_val = self.eval_expr(right)?;

                match op {
                    BinaryOp::Add => {
                        if matches!(l_val, VbaValue::Double(_))
                            || matches!(r_val, VbaValue::Double(_))
                        {
                            let l = l_val.as_f64().unwrap_or(0.0);
                            let r = r_val.as_f64().unwrap_or(0.0);
                            Ok(VbaValue::Double(l + r))
                        } else if let (Some(l), Some(r)) = (l_val.as_i64(), r_val.as_i64()) {
                            Ok(VbaValue::Integer(l + r))
                        } else {
                            let l = l_val.as_f64().unwrap_or(0.0);
                            let r = r_val.as_f64().unwrap_or(0.0);
                            Ok(VbaValue::Double(l + r))
                        }
                    }
                    BinaryOp::Sub => {
                        if matches!(l_val, VbaValue::Double(_))
                            || matches!(r_val, VbaValue::Double(_))
                        {
                            let l = l_val.as_f64().unwrap_or(0.0);
                            let r = r_val.as_f64().unwrap_or(0.0);
                            Ok(VbaValue::Double(l - r))
                        } else if let (Some(l), Some(r)) = (l_val.as_i64(), r_val.as_i64()) {
                            Ok(VbaValue::Integer(l - r))
                        } else {
                            let l = l_val.as_f64().unwrap_or(0.0);
                            let r = r_val.as_f64().unwrap_or(0.0);
                            Ok(VbaValue::Double(l - r))
                        }
                    }
                    BinaryOp::Mul => {
                        if matches!(l_val, VbaValue::Double(_))
                            || matches!(r_val, VbaValue::Double(_))
                        {
                            let l = l_val.as_f64().unwrap_or(0.0);
                            let r = r_val.as_f64().unwrap_or(0.0);
                            Ok(VbaValue::Double(l * r))
                        } else if let (Some(l), Some(r)) = (l_val.as_i64(), r_val.as_i64()) {
                            Ok(VbaValue::Integer(l * r))
                        } else {
                            let l = l_val.as_f64().unwrap_or(0.0);
                            let r = r_val.as_f64().unwrap_or(0.0);
                            Ok(VbaValue::Double(l * r))
                        }
                    }
                    BinaryOp::Div => {
                        let l = l_val.as_f64().unwrap_or(0.0);
                        let r = r_val.as_f64().unwrap_or(1.0);
                        if r == 0.0 {
                            return Err(MacroError::EvalError("Division by zero".to_string()));
                        }
                        Ok(VbaValue::Double(l / r))
                    }
                    BinaryOp::IntDiv => {
                        let l = l_val.as_i64().unwrap_or(0);
                        let r = r_val.as_i64().unwrap_or(1);
                        if r == 0 {
                            return Err(MacroError::EvalError("Division by zero".to_string()));
                        }
                        Ok(VbaValue::Integer(l / r))
                    }
                    BinaryOp::Pow => {
                        let l = l_val.as_f64().unwrap_or(0.0);
                        let r = r_val.as_f64().unwrap_or(1.0);
                        Ok(VbaValue::Double(l.powf(r)))
                    }
                    BinaryOp::Concat => Ok(VbaValue::String(format!(
                        "{}{}",
                        l_val.to_display_string(),
                        r_val.to_display_string()
                    ))),
                    BinaryOp::Mod => {
                        let l = l_val.as_i64().unwrap_or(0);
                        let r = r_val.as_i64().unwrap_or(1);
                        if r == 0 {
                            return Err(MacroError::EvalError("Modulo by zero".to_string()));
                        }
                        Ok(VbaValue::Integer(l % r))
                    }
                    BinaryOp::Eq => {
                        if let (Some(l), Some(r)) = (l_val.as_f64(), r_val.as_f64()) {
                            Ok(VbaValue::Boolean((l - r).abs() < f64::EPSILON))
                        } else {
                            Ok(VbaValue::Boolean(
                                l_val.to_display_string() == r_val.to_display_string(),
                            ))
                        }
                    }
                    BinaryOp::NotEq => {
                        if let (Some(l), Some(r)) = (l_val.as_f64(), r_val.as_f64()) {
                            Ok(VbaValue::Boolean((l - r).abs() >= f64::EPSILON))
                        } else {
                            Ok(VbaValue::Boolean(
                                l_val.to_display_string() != r_val.to_display_string(),
                            ))
                        }
                    }
                    BinaryOp::Lt => {
                        let l = l_val.as_f64().unwrap_or(0.0);
                        let r = r_val.as_f64().unwrap_or(0.0);
                        Ok(VbaValue::Boolean(l < r))
                    }
                    BinaryOp::LtEq => {
                        let l = l_val.as_f64().unwrap_or(0.0);
                        let r = r_val.as_f64().unwrap_or(0.0);
                        Ok(VbaValue::Boolean(l <= r))
                    }
                    BinaryOp::Gt => {
                        let l = l_val.as_f64().unwrap_or(0.0);
                        let r = r_val.as_f64().unwrap_or(0.0);
                        Ok(VbaValue::Boolean(l > r))
                    }
                    BinaryOp::GtEq => {
                        let l = l_val.as_f64().unwrap_or(0.0);
                        let r = r_val.as_f64().unwrap_or(0.0);
                        Ok(VbaValue::Boolean(l >= r))
                    }
                    BinaryOp::And => Ok(VbaValue::Boolean(l_val.as_bool() && r_val.as_bool())),
                    BinaryOp::Or => Ok(VbaValue::Boolean(l_val.as_bool() || r_val.as_bool())),
                    BinaryOp::Xor => Ok(VbaValue::Boolean(l_val.as_bool() ^ r_val.as_bool())),
                }
            }

            VbaExpr::MemberAccess { object, member } => {
                if let VbaExpr::Variable(name) = object.as_ref() {
                    let key = name.to_ascii_lowercase();
                    if let Some(ScopeValue::Range(range)) = self.variables.get(&key) {
                        let r = *range;
                        let sheet = self.dom.active_sheet()?;
                        if member.eq_ignore_ascii_case("value") {
                            return Ok(sheet.get_value(r.start_row, r.start_col));
                        } else if member.eq_ignore_ascii_case("formula") {
                            return Ok(VbaValue::String(
                                sheet
                                    .get_formula(r.start_row, r.start_col)
                                    .unwrap_or_default(),
                            ));
                        }
                    }
                }

                // e.g. Range("A1").Value
                if let VbaExpr::IndexOrCall { callee, args } = object.as_ref() {
                    if let VbaExpr::Variable(name) = callee.as_ref() {
                        if name.eq_ignore_ascii_case("range") && !args.is_empty() {
                            let a1 = self.eval_expr(&args[0])?.to_display_string();
                            let range = RangeAddress::from_a1(&a1)?;
                            let sheet = self.dom.active_sheet()?;
                            if member.eq_ignore_ascii_case("value") {
                                return Ok(sheet.get_value(range.start_row, range.start_col));
                            } else if member.eq_ignore_ascii_case("formula") {
                                return Ok(VbaValue::String(
                                    sheet
                                        .get_formula(range.start_row, range.start_col)
                                        .unwrap_or_default(),
                                ));
                            }
                        }
                    }
                }

                Err(MacroError::EvalError(format!(
                    "Unsupported member read: {object:?}.{member}"
                )))
            }

            VbaExpr::ImplicitMember(member) => {
                if let Some(range) = self.with_context {
                    let sheet = self.dom.active_sheet()?;
                    if member.eq_ignore_ascii_case("value") {
                        Ok(sheet.get_value(range.start_row, range.start_col))
                    } else if member.eq_ignore_ascii_case("formula") {
                        Ok(VbaValue::String(
                            sheet
                                .get_formula(range.start_row, range.start_col)
                                .unwrap_or_default(),
                        ))
                    } else if member.eq_ignore_ascii_case("color") {
                        let c = sheet.get_cell(range.start_row, range.start_col);
                        Ok(VbaValue::Integer(c.interior_color.unwrap_or(0) as i64))
                    } else {
                        Err(MacroError::EvalError(format!(
                            "Unsupported implicit member '.{member}'"
                        )))
                    }
                } else {
                    Err(MacroError::EvalError(
                        "Implicit member access without With block context".to_string(),
                    ))
                }
            }

            VbaExpr::IndexOrCall { callee, args } => {
                if let VbaExpr::Variable(name) = callee.as_ref() {
                    let lower = name.to_ascii_lowercase();

                    if lower == "shell" {
                        self.sandbox
                            .check_capability(Capability::SystemCommand, "Shell")?;
                        return Ok(VbaValue::Integer(1000));
                    }

                    if lower == "environ" {
                        self.sandbox
                            .check_capability(Capability::Environment, "Environ")?;
                        return Ok(VbaValue::String("SIMULATED_ENV".to_string()));
                    }

                    if lower == "createobject" {
                        let prog_id = if let Some(first) = args.first() {
                            self.eval_expr(first)?.to_display_string()
                        } else {
                            String::new()
                        };
                        let p_lower = prog_id.to_ascii_lowercase();
                        let cap = if p_lower.contains("shell") {
                            Capability::SystemCommand
                        } else if p_lower.contains("http") || p_lower.contains("xmlhttp") {
                            Capability::Network
                        } else if p_lower.contains("filesystemobject") || p_lower.contains("stream")
                        {
                            Capability::FileIo
                        } else {
                            Capability::SystemCommand
                        };
                        self.sandbox
                            .check_capability(cap, &format!("CreateObject({prog_id})"))?;
                        return Ok(VbaValue::String(format!("COMObject:{prog_id}")));
                    }

                    if lower == "cells" && args.len() == 2 {
                        let r = self.eval_expr(&args[0])?.as_i64().unwrap_or(1) as u32;
                        let c = self.eval_expr(&args[1])?.as_i64().unwrap_or(1) as u32;
                        let sheet = self.dom.active_sheet()?;
                        return Ok(sheet.get_value(r, c));
                    }

                    if lower == "len" && !args.is_empty() {
                        let s = self.eval_expr(&args[0])?.to_display_string();
                        return Ok(VbaValue::Integer(s.len() as i64));
                    }

                    if lower == "ucase" && !args.is_empty() {
                        let s = self.eval_expr(&args[0])?.to_display_string();
                        return Ok(VbaValue::String(s.to_uppercase()));
                    }

                    if lower == "lcase" && !args.is_empty() {
                        let s = self.eval_expr(&args[0])?.to_display_string();
                        return Ok(VbaValue::String(s.to_lowercase()));
                    }

                    if lower == "abs" && !args.is_empty() {
                        let f = self.eval_expr(&args[0])?.as_f64().unwrap_or(0.0);
                        return Ok(VbaValue::Double(f.abs()));
                    }
                }

                // If callee is a range object with .Offset(r, c)
                if let VbaExpr::MemberAccess { object, member } = callee.as_ref() {
                    if member.eq_ignore_ascii_case("offset") && args.len() == 2 {
                        let r_off = self.eval_expr(&args[0])?.as_i64().unwrap_or(0) as i32;
                        let c_off = self.eval_expr(&args[1])?.as_i64().unwrap_or(0) as i32;

                        if let VbaExpr::IndexOrCall {
                            callee: inner_callee,
                            args: r_args,
                        } = object.as_ref()
                        {
                            if let VbaExpr::Variable(name) = inner_callee.as_ref() {
                                if name.eq_ignore_ascii_case("range") && !r_args.is_empty() {
                                    let a1 = self.eval_expr(&r_args[0])?.to_display_string();
                                    let base_range = RangeAddress::from_a1(&a1)?;
                                    let offset_range = base_range.offset(r_off, c_off)?;
                                    let sheet = self.dom.active_sheet()?;
                                    return Ok(sheet.get_value(
                                        offset_range.start_row,
                                        offset_range.start_col,
                                    ));
                                }
                            }
                        }
                    }
                }

                Err(MacroError::EvalError(format!(
                    "Unknown function or indexing call: {callee:?}"
                )))
            }
        }
    }
}
