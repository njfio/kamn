#[path = "evaluation/contract_decision.rs"]
mod contract_decision;
#[path = "evaluation/contract_logic.rs"]
mod contract_logic;
#[path = "evaluation/section_logic.rs"]
mod section_logic;

pub(crate) use contract_logic::evaluate_contract;
