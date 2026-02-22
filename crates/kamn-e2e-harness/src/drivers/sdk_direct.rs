use crate::drivers::{DriverExecutionResult, HarnessDriver};
use crate::ExecutionMode;

/// SDK-direct driver scaffold.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SdkDirectDriver;

impl HarnessDriver for SdkDirectDriver {
    fn mode(&self) -> ExecutionMode {
        ExecutionMode::SdkDirect
    }

    fn execute(&self, scenario_id: &'static str) -> DriverExecutionResult {
        DriverExecutionResult {
            scenario_id,
            status: "pass",
        }
    }
}
