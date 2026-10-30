use crate::error::MacroError;
use crate::security::{Capability, PermissionStatus, SecurityAuditLog};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Sandbox configuration and runtime isolation boundary for macro execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroSandbox {
    pub gas_limit: u64,
    pub gas_remaining: u64,
    pub permissions: HashMap<Capability, PermissionStatus>,
    pub audit_log: SecurityAuditLog,
}

impl Default for MacroSandbox {
    fn default() -> Self {
        Self::new(100_000)
    }
}

impl MacroSandbox {
    pub fn new(gas_limit: u64) -> Self {
        let mut permissions = HashMap::new();
        // By default under GEC-v2.0, all dangerous capabilities require explicit user grant/prompt
        permissions.insert(Capability::FileIo, PermissionStatus::Denied);
        permissions.insert(Capability::Network, PermissionStatus::Denied);
        permissions.insert(Capability::SystemCommand, PermissionStatus::Denied);
        permissions.insert(Capability::Environment, PermissionStatus::Denied);
        permissions.insert(Capability::Registry, PermissionStatus::Denied);

        Self {
            gas_limit,
            gas_remaining: gas_limit,
            permissions,
            audit_log: SecurityAuditLog::new(),
        }
    }

    pub fn with_unlimited_gas() -> Self {
        Self::new(u64::MAX)
    }

    pub fn grant(&mut self, cap: Capability) {
        self.permissions.insert(cap, PermissionStatus::Granted);
        self.audit_log.record(
            cap,
            "GRANT_CAPABILITY",
            true,
            "User explicitly granted capability",
        );
    }

    pub fn deny(&mut self, cap: Capability) {
        self.permissions.insert(cap, PermissionStatus::Denied);
        self.audit_log
            .record(cap, "DENY_CAPABILITY", false, "Capability revoked / denied");
    }

    pub fn set_status(&mut self, cap: Capability, status: PermissionStatus) {
        self.permissions.insert(cap, status);
    }

    pub fn check_capability(&mut self, cap: Capability, action: &str) -> Result<(), MacroError> {
        let status = self
            .permissions
            .get(&cap)
            .copied()
            .unwrap_or(PermissionStatus::Denied);

        match status {
            PermissionStatus::Granted => {
                self.audit_log.record(
                    cap,
                    action,
                    true,
                    "Execution permitted under granted capability",
                );
                Ok(())
            }
            PermissionStatus::Denied | PermissionStatus::PromptRequired => {
                self.audit_log.record(
                    cap,
                    action,
                    false,
                    "Execution blocked by sandbox capability gatekeeper",
                );
                Err(MacroError::PermissionDenied(format!(
                    "Sandbox blocked access to {cap:?} for action '{action}'"
                )))
            }
        }
    }

    pub fn consume_gas(&mut self, amount: u64) -> Result<(), MacroError> {
        if self.gas_remaining < amount {
            self.gas_remaining = 0;
            return Err(MacroError::GasExhausted(format!(
                "Execution gas limit reached ({})",
                self.gas_limit
            )));
        }
        self.gas_remaining -= amount;
        Ok(())
    }

    pub fn reset_gas(&mut self) {
        self.gas_remaining = self.gas_limit;
    }

    pub fn remaining_gas(&self) -> u64 {
        self.gas_remaining
    }

    pub fn audit_log(&self) -> &SecurityAuditLog {
        &self.audit_log
    }

    pub fn audit_log_mut(&mut self) -> &mut SecurityAuditLog {
        &mut self.audit_log
    }
}
