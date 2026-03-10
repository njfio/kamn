use super::*;
use super::shared_support::*;

#[path = "compatibility_resilience_contract_tests/compatibility_schema_contract_tests.rs"]
mod compatibility_schema_contract_tests;
#[path = "compatibility_resilience_contract_tests/resilience_local_heavy_contract_tests.rs"]
mod resilience_local_heavy_contract_tests;
#[path = "compatibility_resilience_contract_tests/lifecycle_presence_contract_tests.rs"]
mod lifecycle_presence_contract_tests;
