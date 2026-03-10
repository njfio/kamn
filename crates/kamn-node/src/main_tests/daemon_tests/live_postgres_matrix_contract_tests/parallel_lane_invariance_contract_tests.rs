// daemon live-postgres matrix invariance contracts decomposition: keep order and permutation
// invariance assertions split into bounded include modules while preserving canonical test names.
include!("parallel_lane_invariance_contract_tests/order_invariance_contract_tests.rs");
include!("parallel_lane_invariance_contract_tests/permutation_invariance_contract_tests.rs");
