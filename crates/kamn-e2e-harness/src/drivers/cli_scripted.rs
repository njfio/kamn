use crate::drivers::{DriverExecutionResult, HarnessDriver};
use crate::ExecutionMode;

/// CLI-scripted driver scaffold.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CliScriptedDriver;

impl HarnessDriver for CliScriptedDriver {
    fn mode(&self) -> ExecutionMode {
        ExecutionMode::CliScripted
    }

    fn execute(&self, scenario_id: &'static str) -> DriverExecutionResult {
        DriverExecutionResult {
            scenario_id,
            status: "pass",
        }
    }
}
