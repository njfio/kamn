use super::*;
use crate::daemon_test_env_lock;
#[cfg(unix)]
use crate::{configure_os_signal_test_triggers, OsSignalTestKind, OsSignalTestTrigger};

mod live_postgres_fixtures;
use live_postgres_fixtures::*;

// daemon_tests structural budget shell phase3; route runtime/matrix/topology contracts through src/main_tests/daemon_tests/*.rs includes
include!("daemon_tests/runtime_contract_tests.rs");
include!("daemon_tests/live_postgres_matrix_contract_tests.rs");
include!("daemon_tests/live_postgres_topology_contract_tests.rs");
include!("daemon_tests/live_postgres_distributed_execution_contract_tests.rs");
