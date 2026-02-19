use kamn_core::{EscrowLifecycle, EscrowStatus, TaskState};

#[path = "../property_invariant_helpers.rs"]
pub mod property_invariant_helpers;

pub const TASK_CASES: u32 = 192;
pub const ESCROW_CASES: u32 = 192;
pub const MAX_SEQUENCE_LEN: usize = 32;
pub const TASK_SEED: u64 = 0x3532_0000_0000_0001;
pub const ESCROW_SEED: u64 = 0x3532_0000_0000_0002;
pub const TASK_SEED_ENV_KEY: &str = "KAMN_PROPTEST_TASK_ESCROW_SEED";
pub const ESCROW_SEED_ENV_KEY: &str = "KAMN_PROPTEST_ESCROW_SEED";
pub const TASK_EVIDENCE_SEED_SALT: u64 = 0x0aa0_55ff;
pub const TASK_RESTORE_SEED_SALT: u64 = 0x0f0f_0f0f;
pub const ESCROW_EVIDENCE_SEED_SALT: u64 = 0x00ff_aacc;
pub const SUITE_SOURCE_PATH: &str = "crates/kamn-core/tests/task_escrow_proptest_invariants.rs";

pub fn task_seed() -> u64 {
    property_invariant_helpers::resolve_seed_from_env(TASK_SEED_ENV_KEY, TASK_SEED)
}

pub fn escrow_seed() -> u64 {
    property_invariant_helpers::resolve_seed_from_env(ESCROW_SEED_ENV_KEY, ESCROW_SEED)
}

pub fn deterministic_config(cases: u32, seed: u64) -> proptest::test_runner::Config {
    property_invariant_helpers::deterministic_proptest_config(cases, seed, SUITE_SOURCE_PATH)
}

pub fn escrow_invariant_violation(escrow: &EscrowLifecycle, total: u128) -> Option<String> {
    let released = escrow.released_amount();
    let refunded = escrow.refunded_amount();
    let remaining = escrow.remaining_amount();
    let total_projection = released
        .checked_add(refunded)
        .and_then(|value| value.checked_add(remaining));
    if total_projection != Some(total) {
        return Some(format!(
            "amount conservation failed: released={released}, refunded={refunded}, remaining={remaining}, total={total}"
        ));
    }

    match escrow.status() {
        EscrowStatus::Funded => {
            if released != 0 || refunded != 0 || remaining != total {
                return Some(format!(
                    "funded projection mismatch: released={released}, refunded={refunded}, remaining={remaining}, total={total}"
                ));
            }
        }
        EscrowStatus::PartiallyReleased {
            released: status_released,
            remaining: status_remaining,
        } => {
            if status_released != released || status_remaining != remaining || remaining == 0 {
                return Some(format!(
                    "partial projection mismatch: status_released={status_released}, released={released}, status_remaining={status_remaining}, remaining={remaining}"
                ));
            }
        }
        EscrowStatus::Released => {
            if released != total || refunded != 0 || remaining != 0 {
                return Some(format!(
                    "released projection mismatch: released={released}, refunded={refunded}, remaining={remaining}, total={total}"
                ));
            }
        }
        EscrowStatus::Refunded => {
            if refunded == 0 || remaining != 0 {
                return Some(format!(
                    "refunded projection mismatch: released={released}, refunded={refunded}, remaining={remaining}, total={total}"
                ));
            }
        }
        EscrowStatus::Disputed => {
            if remaining == 0 {
                return Some("disputed projection must keep non-zero remaining balance".to_owned());
            }
        }
        EscrowStatus::Resolved {
            released_total,
            refunded_total,
        } => {
            if released_total != released || refunded_total != refunded || remaining != 0 {
                return Some(format!(
                    "resolved projection mismatch: status_released={released_total}, released={released}, status_refunded={refunded_total}, refunded={refunded}, remaining={remaining}"
                ));
            }
        }
    }
    None
}

pub fn history_starts_submitted(history: &[TaskState]) -> bool {
    history.first().copied() == Some(TaskState::Submitted)
}
