//! Fast In-Memory Multi-Dimensional OLAP Aggregation Cube Engine.
//! Supports arbitrary hierarchical row & column groupings, page report filters,
//! comprehensive aggregation metrics (SUM, COUNT, AVERAGE, MIN, MAX, PRODUCT, STDDEV,
//! VARIANCE, DISTINCT COUNT, % OF TOTAL), automatic date hierarchy grouping,
//! and direct rendering into `excel_engine::grid::CompressedSparsePageGrid`.

use crate::error::ConnectorResult;
use crate::schema::Schema;
use crate::value::DataValue;
use excel_engine::grid::CompressedSparsePageGrid;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Converts civil day offset (from 1970-01-01) back into `(year, month, day)`.
pub fn civil_from_days(days: i32) -> (i32, u32, u32) {
    let z = days + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i32) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

/// OLAP Aggregation function kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AggregationType {
    Sum,
    Count,
    CountNumbers,
    Average,
    Min,
    Max,
    Product,
    Variance,
    StdDev,
    DistinctCount,
    PercentOfTotal,
    PercentOfRow,
    PercentOfCol,
}

impl AggregationType {
    pub fn default_label(&self, source_field: &str) -> String {
        match self {
            Self::Sum => format!("Sum of {source_field}"),
            Self::Count => format!("Count of {source_field}"),
            Self::CountNumbers => format!("Count Numbers of {source_field}"),
            Self::Average => format!("Average of {source_field}"),
            Self::Min => format!("Min of {source_field}"),
            Self::Max => format!("Max of {source_field}"),
            Self::Product => format!("Product of {source_field}"),
            Self::Variance => format!("Var of {source_field}"),
            Self::StdDev => format!("StdDev of {source_field}"),
            Self::DistinctCount => format!("Distinct Count of {source_field}"),
            Self::PercentOfTotal => format!("% Total of {source_field}"),
            Self::PercentOfRow => format!("% Row of {source_field}"),
            Self::PercentOfCol => format!("% Col of {source_field}"),
        }
    }
}

/// Date grouping levels for automatic date hierarchy (Year -> Quarter -> Month -> Day).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DateGroupingLevel {
    Year,
    Quarter,
    Month,
    Day,
}

impl DateGroupingLevel {
    pub fn extract(&self, val: &DataValue) -> DataValue {
        let (y, m, d) = match val {
            DataValue::Date(days) => civil_from_days(*days),
            DataValue::DateTime(ms) => {
                let days = (*ms / 86400000) as i32;
                civil_from_days(days)
            }
            DataValue::Text(s) => {
                let trimmed = s.trim();
                if trimmed.len() >= 10 {
                    let y = trimmed[0..4].parse::<i32>().unwrap_or(1970);
                    let m = trimmed[5..7].parse::<u32>().unwrap_or(1);
                    let d = trimmed[8..10].parse::<u32>().unwrap_or(1);
                    (y, m, d)
                } else {
                    return val.clone();
                }
            }
            _ => return val.clone(),
        };

        match self {
            Self::Year => DataValue::Int64(y as i64),
            Self::Quarter => {
                let q = (m.saturating_sub(1)) / 3 + 1;
                DataValue::Text(format!("Q{q}"))
            }
            Self::Month => {
                let names = [
                    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov",
                    "Dec",
                ];
                let idx = (m.saturating_sub(1)) as usize;
                let name = names.get(idx).copied().unwrap_or("Jan");
                DataValue::Text(name.to_string())
            }
            Self::Day => DataValue::Int64(d as i64),
        }
    }
}

/// A grouping dimension field on Rows or Columns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PivotField {
    pub source_column: String,
    pub date_grouping: Option<DateGroupingLevel>,
    pub sort_ascending: bool,
}

impl PivotField {
    pub fn new(source_column: impl Into<String>) -> Self {
        Self {
            source_column: source_column.into(),
            date_grouping: None,
            sort_ascending: true,
        }
    }

    pub fn with_date_grouping(mut self, level: DateGroupingLevel) -> Self {
        self.date_grouping = Some(level);
        self
    }
}

/// A value measure field to aggregate in the cube.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeasureField {
    pub source_column: String,
    pub aggregation: AggregationType,
    pub custom_label: Option<String>,
}

impl MeasureField {
    pub fn new(source_column: impl Into<String>, aggregation: AggregationType) -> Self {
        Self {
            source_column: source_column.into(),
            aggregation,
            custom_label: None,
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.custom_label = Some(label.into());
        self
    }

    pub fn label(&self) -> String {
        self.custom_label
            .clone()
            .unwrap_or_else(|| self.aggregation.default_label(&self.source_column))
    }
}

/// Page report filter for filtering datasets before aggregation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterField {
    pub source_column: String,
    pub included_values: Option<Vec<DataValue>>,
    pub excluded_values: Option<Vec<DataValue>>,
}

impl FilterField {
    pub fn include(source_column: impl Into<String>, values: Vec<DataValue>) -> Self {
        Self {
            source_column: source_column.into(),
            included_values: Some(values),
            excluded_values: None,
        }
    }

    pub fn exclude(source_column: impl Into<String>, values: Vec<DataValue>) -> Self {
        Self {
            source_column: source_column.into(),
            included_values: None,
            excluded_values: Some(values),
        }
    }

    pub fn passes(&self, val: &DataValue) -> bool {
        if let Some(ref inc) = self.included_values {
            if !inc.contains(val) {
                return false;
            }
        }
        if let Some(ref exc) = self.excluded_values {
            if exc.contains(val) {
                return false;
            }
        }
        true
    }
}

/// Configuration options for Grand Totals display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GrandTotalMode {
    #[default]
    Both,
    RowsOnly,
    ColumnsOnly,
    None,
}

/// Full configuration definition of a PivotTable / OLAP cube.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PivotConfig {
    pub row_fields: Vec<PivotField>,
    pub col_fields: Vec<PivotField>,
    pub value_fields: Vec<MeasureField>,
    pub filter_fields: Vec<FilterField>,
    pub grand_totals: GrandTotalMode,
    pub show_subtotals: bool,
}

/// In-memory metric accumulator for a single cell or subtotal node.
#[derive(Debug, Clone, Default)]
pub struct MeasureAccumulator {
    pub sum: f64,
    pub count: usize,
    pub count_numbers: usize,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub product: f64,
    pub values: Vec<f64>,
    pub distinct_values: BTreeSet<String>,
}

impl MeasureAccumulator {
    pub fn new() -> Self {
        Self {
            sum: 0.0,
            count: 0,
            count_numbers: 0,
            min: None,
            max: None,
            product: 1.0,
            values: Vec::new(),
            distinct_values: BTreeSet::new(),
        }
    }

    pub fn add(&mut self, val: &DataValue) {
        if val.is_null() {
            return;
        }
        self.count += 1;
        self.distinct_values.insert(val.to_string_lossy());

        if let Some(n) = val.as_f64() {
            self.count_numbers += 1;
            self.sum += n;
            self.values.push(n);

            if self.min.is_none() || Some(n) < self.min {
                self.min = Some(n);
            }
            if self.max.is_none() || Some(n) > self.max {
                self.max = Some(n);
            }
            self.product *= n;
        }
    }

    pub fn compute(&self, agg: AggregationType, grand_total: Option<f64>) -> f64 {
        match agg {
            AggregationType::Sum => self.sum,
            AggregationType::Count => self.count as f64,
            AggregationType::CountNumbers => self.count_numbers as f64,
            AggregationType::Average => {
                if self.count_numbers > 0 {
                    self.sum / (self.count_numbers as f64)
                } else {
                    0.0
                }
            }
            AggregationType::Min => self.min.unwrap_or(0.0),
            AggregationType::Max => self.max.unwrap_or(0.0),
            AggregationType::Product => {
                if self.count_numbers > 0 {
                    self.product
                } else {
                    0.0
                }
            }
            AggregationType::Variance => {
                if self.count_numbers > 1 {
                    let mean = self.sum / (self.count_numbers as f64);
                    let var_sum: f64 = self.values.iter().map(|v| (v - mean).powi(2)).sum();
                    var_sum / ((self.count_numbers - 1) as f64)
                } else {
                    0.0
                }
            }
            AggregationType::StdDev => {
                let var = self.compute(AggregationType::Variance, None);
                var.sqrt()
            }
            AggregationType::DistinctCount => self.distinct_values.len() as f64,
            AggregationType::PercentOfTotal
            | AggregationType::PercentOfRow
            | AggregationType::PercentOfCol => {
                if let Some(gt) = grand_total {
                    if gt != 0.0 {
                        (self.sum / gt) * 100.0
                    } else {
                        0.0
                    }
                } else {
                    100.0
                }
            }
        }
    }
}

/// Coordinate tuple for multidimensional rows/columns: `["North", "Q1"]`.
pub type CoordTuple = Vec<String>;

/// In-Memory Multi-Dimensional OLAP Aggregation Cube.
#[derive(Debug, Clone)]
pub struct OlapCube {
    pub config: PivotConfig,
    pub row_coords: Vec<CoordTuple>,
    pub col_coords: Vec<CoordTuple>,
    /// Dense/sparse cell accumulators: `(row_idx, col_idx, measure_idx) -> MeasureAccumulator`
    pub cells: BTreeMap<(usize, usize, usize), MeasureAccumulator>,
    /// Row total accumulators: `(row_idx, measure_idx) -> MeasureAccumulator`
    pub row_totals: BTreeMap<(usize, usize), MeasureAccumulator>,
    /// Column total accumulators: `(col_idx, measure_idx) -> MeasureAccumulator`
    pub col_totals: BTreeMap<(usize, usize), MeasureAccumulator>,
    /// Grand total accumulators: `measure_idx -> MeasureAccumulator`
    pub grand_totals: BTreeMap<usize, MeasureAccumulator>,
}

impl OlapCube {
    /// Builds and executes the multi-dimensional cube over a tabular dataset.
    pub fn build(
        schema: &Schema,
        records: &[Vec<DataValue>],
        config: PivotConfig,
    ) -> ConnectorResult<Self> {
        let col_map: BTreeMap<String, usize> = schema
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| (f.name.clone(), i))
            .collect();

        // Resolve field indices
        let row_col_indices: Vec<(usize, Option<DateGroupingLevel>)> = config
            .row_fields
            .iter()
            .filter_map(|rf| {
                col_map
                    .get(&rf.source_column)
                    .map(|&idx| (idx, rf.date_grouping))
            })
            .collect();

        let col_col_indices: Vec<(usize, Option<DateGroupingLevel>)> = config
            .col_fields
            .iter()
            .filter_map(|cf| {
                col_map
                    .get(&cf.source_column)
                    .map(|&idx| (idx, cf.date_grouping))
            })
            .collect();

        let measure_indices: Vec<(usize, AggregationType)> = config
            .value_fields
            .iter()
            .filter_map(|vf| {
                col_map
                    .get(&vf.source_column)
                    .map(|&idx| (idx, vf.aggregation))
            })
            .collect();

        let filter_indices: Vec<(usize, FilterField)> = config
            .filter_fields
            .iter()
            .filter_map(|ff| col_map.get(&ff.source_column).map(|&idx| (idx, ff.clone())))
            .collect();

        // Filter and collect unique row and col tuples
        let mut row_set = BTreeSet::new();
        let mut col_set = BTreeSet::new();

        let mut filtered_records = Vec::with_capacity(records.len());

        for row in records {
            // Apply page report filters
            let passes_filters = filter_indices.iter().all(|(col_idx, filter)| {
                let val = row.get(*col_idx).unwrap_or(&DataValue::Null);
                filter.passes(val)
            });

            if !passes_filters {
                continue;
            }

            // Extract row tuple
            let mut r_tuple = Vec::with_capacity(row_col_indices.len());
            for (idx, date_grp) in &row_col_indices {
                let raw_val = row.get(*idx).unwrap_or(&DataValue::Null);
                let grouped_val = if let Some(dg) = date_grp {
                    dg.extract(raw_val)
                } else {
                    raw_val.clone()
                };
                r_tuple.push(grouped_val.to_string_lossy());
            }

            // Extract col tuple
            let mut c_tuple = Vec::with_capacity(col_col_indices.len());
            for (idx, date_grp) in &col_col_indices {
                let raw_val = row.get(*idx).unwrap_or(&DataValue::Null);
                let grouped_val = if let Some(dg) = date_grp {
                    dg.extract(raw_val)
                } else {
                    raw_val.clone()
                };
                c_tuple.push(grouped_val.to_string_lossy());
            }

            if !r_tuple.is_empty() {
                row_set.insert(r_tuple.clone());
            }
            if !c_tuple.is_empty() {
                col_set.insert(c_tuple.clone());
            }

            filtered_records.push((r_tuple, c_tuple, row));
        }

        // Deterministic sorted list of coordinates
        let mut row_coords: Vec<CoordTuple> = row_set.into_iter().collect();
        let mut col_coords: Vec<CoordTuple> = col_set.into_iter().collect();

        if row_coords.is_empty() {
            row_coords.push(vec!["Total".to_string()]);
        }
        if col_coords.is_empty() {
            col_coords.push(vec!["Total".to_string()]);
        }

        let row_idx_map: BTreeMap<CoordTuple, usize> = row_coords
            .iter()
            .enumerate()
            .map(|(i, t)| (t.clone(), i))
            .collect();

        let col_idx_map: BTreeMap<CoordTuple, usize> = col_coords
            .iter()
            .enumerate()
            .map(|(i, t)| (t.clone(), i))
            .collect();

        let mut cells: BTreeMap<(usize, usize, usize), MeasureAccumulator> = BTreeMap::new();
        let mut row_totals: BTreeMap<(usize, usize), MeasureAccumulator> = BTreeMap::new();
        let mut col_totals: BTreeMap<(usize, usize), MeasureAccumulator> = BTreeMap::new();
        let mut grand_totals: BTreeMap<usize, MeasureAccumulator> = BTreeMap::new();

        // Populate aggregations
        for (r_tuple, c_tuple, row) in filtered_records {
            let r_idx = row_idx_map.get(&r_tuple).copied().unwrap_or(0);
            let c_idx = col_idx_map.get(&c_tuple).copied().unwrap_or(0);

            for (m_idx, (col_idx, _agg)) in measure_indices.iter().enumerate() {
                let val = row.get(*col_idx).unwrap_or(&DataValue::Null);

                // Cell accumulator
                cells.entry((r_idx, c_idx, m_idx)).or_default().add(val);

                // Row total
                row_totals.entry((r_idx, m_idx)).or_default().add(val);

                // Col total
                col_totals.entry((c_idx, m_idx)).or_default().add(val);

                // Grand total
                grand_totals.entry(m_idx).or_default().add(val);
            }
        }

        Ok(Self {
            config,
            row_coords,
            col_coords,
            cells,
            row_totals,
            col_totals,
            grand_totals,
        })
    }

    /// Evaluates value for cell `(row_idx, col_idx)` of `measure_idx`.
    pub fn get_value(&self, row_idx: usize, col_idx: usize, measure_idx: usize) -> f64 {
        let agg = self
            .config
            .value_fields
            .get(measure_idx)
            .map(|m| m.aggregation)
            .unwrap_or(AggregationType::Sum);

        let grand_total_val = self
            .grand_totals
            .get(&measure_idx)
            .map(|gt| gt.compute(AggregationType::Sum, None));

        if let Some(acc) = self.cells.get(&(row_idx, col_idx, measure_idx)) {
            acc.compute(agg, grand_total_val)
        } else {
            0.0
        }
    }

    /// Evaluates row subtotal for `row_idx` of `measure_idx`.
    pub fn get_row_total(&self, row_idx: usize, measure_idx: usize) -> f64 {
        let agg = self
            .config
            .value_fields
            .get(measure_idx)
            .map(|m| m.aggregation)
            .unwrap_or(AggregationType::Sum);

        let grand_total_val = self
            .grand_totals
            .get(&measure_idx)
            .map(|gt| gt.compute(AggregationType::Sum, None));

        if let Some(acc) = self.row_totals.get(&(row_idx, measure_idx)) {
            acc.compute(agg, grand_total_val)
        } else {
            0.0
        }
    }

    /// Evaluates column subtotal for `col_idx` of `measure_idx`.
    pub fn get_col_total(&self, col_idx: usize, measure_idx: usize) -> f64 {
        let agg = self
            .config
            .value_fields
            .get(measure_idx)
            .map(|m| m.aggregation)
            .unwrap_or(AggregationType::Sum);

        let grand_total_val = self
            .grand_totals
            .get(&measure_idx)
            .map(|gt| gt.compute(AggregationType::Sum, None));

        if let Some(acc) = self.col_totals.get(&(col_idx, measure_idx)) {
            acc.compute(agg, grand_total_val)
        } else {
            0.0
        }
    }

    /// Evaluates grand total for `measure_idx`.
    pub fn get_grand_total(&self, measure_idx: usize) -> f64 {
        let agg = self
            .config
            .value_fields
            .get(measure_idx)
            .map(|m| m.aggregation)
            .unwrap_or(AggregationType::Sum);

        if let Some(acc) = self.grand_totals.get(&measure_idx) {
            acc.compute(agg, None)
        } else {
            0.0
        }
    }

    /// Simulates Excel's `GETPIVOTDATA` function to retrieve aggregated measure values
    /// using field filter pairs (e.g. `[("Region", "North"), ("Year", "2026")]`).
    pub fn get_pivot_data(&self, data_field: &str, filters: &[(&str, &str)]) -> Option<f64> {
        let (measure_idx, _measure) = self
            .config
            .value_fields
            .iter()
            .enumerate()
            .find(|(_, vf)| vf.source_column == data_field || vf.label() == data_field)?;

        if filters.is_empty() {
            return Some(self.get_grand_total(measure_idx));
        }

        // Check if any filter targets row fields
        let has_row_filter = filters.iter().any(|(f_name, _)| {
            self.config
                .row_fields
                .iter()
                .any(|rf| &rf.source_column == f_name)
        });

        let mut matching_row_idx = None;
        if has_row_filter {
            for (r_idx, r_tuple) in self.row_coords.iter().enumerate() {
                let mut matches_all = true;
                for (f_name, f_val) in filters {
                    if let Some(pos) = self
                        .config
                        .row_fields
                        .iter()
                        .position(|rf| &rf.source_column == f_name)
                    {
                        if let Some(val) = r_tuple.get(pos) {
                            if val != f_val {
                                matches_all = false;
                                break;
                            }
                        }
                    }
                }
                if matches_all {
                    matching_row_idx = Some(r_idx);
                    break;
                }
            }
        }

        // Check if any filter targets col fields
        let has_col_filter = filters.iter().any(|(f_name, _)| {
            self.config
                .col_fields
                .iter()
                .any(|cf| &cf.source_column == f_name)
        });

        let mut matching_col_idx = None;
        if has_col_filter {
            for (c_idx, c_tuple) in self.col_coords.iter().enumerate() {
                let mut matches_all = true;
                for (f_name, f_val) in filters {
                    if let Some(pos) = self
                        .config
                        .col_fields
                        .iter()
                        .position(|cf| &cf.source_column == f_name)
                    {
                        if let Some(val) = c_tuple.get(pos) {
                            if val != f_val {
                                matches_all = false;
                                break;
                            }
                        }
                    }
                }
                if matches_all {
                    matching_col_idx = Some(c_idx);
                    break;
                }
            }
        }

        match (matching_row_idx, matching_col_idx) {
            (Some(r), Some(c)) => Some(self.get_value(r, c, measure_idx)),
            (Some(r), None) => Some(self.get_row_total(r, measure_idx)),
            (None, Some(c)) => Some(self.get_col_total(c, measure_idx)),
            (None, None) => Some(self.get_grand_total(measure_idx)),
        }
    }
}

/// Report detailing the layout boundaries of a rendered PivotTable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PivotTableLayoutReport {
    pub start_row: u32,
    pub start_col: u32,
    pub end_row: u32,
    pub end_col: u32,
    pub data_range_start: (u32, u32),
    pub data_range_end: (u32, u32),
    pub row_labels_count: usize,
    pub col_labels_count: usize,
    pub measures_count: usize,
}

/// Bridge rendering an `OlapCube` directly into a Calc sheet grid.
pub struct PivotTableGridBridge;

impl PivotTableGridBridge {
    /// Renders the cube as a tabular PivotTable into `CompressedSparsePageGrid`.
    pub fn render_to_grid(
        cube: &OlapCube,
        grid: &mut CompressedSparsePageGrid,
        start_row: u32,
        start_col: u32,
    ) -> ConnectorResult<PivotTableLayoutReport> {
        let mut cur_row = start_row;
        let start_c = start_col;

        // 1. Render Page Report Filters (if any)
        for filter in &cube.config.filter_fields {
            grid.set_inline_string(
                cur_row,
                start_c,
                &format!("Filter: {}", filter.source_column),
            )?;
            let filter_val = if let Some(ref inc) = filter.included_values {
                let items: Vec<String> = inc.iter().map(|v| v.to_string_lossy()).collect();
                items.join(", ")
            } else {
                "(All)".to_string()
            };
            grid.set_inline_string(cur_row, start_c + 1, &filter_val)?;
            cur_row += 1;
        }

        if !cube.config.filter_fields.is_empty() {
            cur_row += 1; // Blank separator
        }

        let header_row = cur_row;
        let num_row_fields = cube.config.row_fields.len().max(1);
        let num_measures = cube.config.value_fields.len().max(1);

        // 2. Render Row Header Titles
        for (i, rf) in cube.config.row_fields.iter().enumerate() {
            let label = if let Some(dg) = rf.date_grouping {
                format!("{} ({:?})", rf.source_column, dg)
            } else {
                rf.source_column.clone()
            };
            grid.set_inline_string(header_row, start_c + i as u32, &label)?;
        }

        // 3. Render Column Group Titles
        let col_data_start = start_c + num_row_fields as u32;
        let mut cur_col_offset = col_data_start;

        for c_tuple in &cube.col_coords {
            let col_label = c_tuple.join(" - ");
            for (m_idx, vf) in cube.config.value_fields.iter().enumerate() {
                let title = if cube.col_coords.len() > 1
                    || c_tuple.first().map(|s| s.as_str()) != Some("Total")
                {
                    format!("{} | {}", col_label, vf.label())
                } else {
                    vf.label()
                };
                grid.set_inline_string(header_row, cur_col_offset + m_idx as u32, &title)?;
            }
            cur_col_offset += num_measures as u32;
        }

        // Render Grand Total Column Header (if enabled)
        let include_col_totals = matches!(
            cube.config.grand_totals,
            GrandTotalMode::Both | GrandTotalMode::ColumnsOnly
        );

        let grand_total_col_start = cur_col_offset;
        if include_col_totals {
            for (m_idx, vf) in cube.config.value_fields.iter().enumerate() {
                let title = format!("Total {}", vf.label());
                grid.set_inline_string(header_row, cur_col_offset + m_idx as u32, &title)?;
            }
            cur_col_offset += num_measures as u32;
        }

        let end_col = cur_col_offset;
        cur_row += 1;

        let data_start_row = cur_row;

        // 4. Render Data Rows
        for (r_idx, r_tuple) in cube.row_coords.iter().enumerate() {
            // Write row dimension labels
            for (f_idx, text) in r_tuple.iter().enumerate() {
                grid.set_inline_string(cur_row, start_c + f_idx as u32, text)?;
            }

            // Write measure cell values
            let mut c_offset = col_data_start;
            for (c_idx, _) in cube.col_coords.iter().enumerate() {
                for m_idx in 0..num_measures {
                    let val = cube.get_value(r_idx, c_idx, m_idx);
                    grid.set_number(cur_row, c_offset + m_idx as u32, val)?;
                }
                c_offset += num_measures as u32;
            }

            // Write row grand totals
            if include_col_totals {
                for m_idx in 0..num_measures {
                    let total = cube.get_row_total(r_idx, m_idx);
                    grid.set_number(cur_row, grand_total_col_start + m_idx as u32, total)?;
                }
            }

            cur_row += 1;
        }

        let data_end_row = cur_row;

        // 5. Render Grand Total Row (if enabled)
        let include_row_totals = matches!(
            cube.config.grand_totals,
            GrandTotalMode::Both | GrandTotalMode::RowsOnly
        );

        if include_row_totals {
            grid.set_inline_string(cur_row, start_c, "Grand Total")?;

            let mut c_offset = col_data_start;
            for (c_idx, _) in cube.col_coords.iter().enumerate() {
                for m_idx in 0..num_measures {
                    let total = cube.get_col_total(c_idx, m_idx);
                    grid.set_number(cur_row, c_offset + m_idx as u32, total)?;
                }
                c_offset += num_measures as u32;
            }

            if include_col_totals {
                for m_idx in 0..num_measures {
                    let total = cube.get_grand_total(m_idx);
                    grid.set_number(cur_row, grand_total_col_start + m_idx as u32, total)?;
                }
            }

            cur_row += 1;
        }

        let final_end_row = cur_row;

        Ok(PivotTableLayoutReport {
            start_row,
            start_col,
            end_row: final_end_row,
            end_col,
            data_range_start: (data_start_row, col_data_start),
            data_range_end: (data_end_row, end_col),
            row_labels_count: cube.row_coords.len(),
            col_labels_count: cube.col_coords.len(),
            measures_count: num_measures,
        })
    }
}
