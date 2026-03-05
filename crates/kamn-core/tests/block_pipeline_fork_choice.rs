use kamn_core::{
    AcceptAllForkChoiceHook, CanonicalCommitRecord, DeterministicCompetingBranchForkChoiceHook,
    ForkChoiceDecision, ForkChoiceHook, NodeRole,
};

fn sample_canonical_record(height: u64, digest: &str) -> CanonicalCommitRecord {
    CanonicalCommitRecord {
        block_height: height,
        producer_role: NodeRole::Processor,
        payload_digest: digest.to_owned(),
        transaction_ids: vec![format!("tx-{height}-{digest}")],
    }
}

#[test]
fn unit_fork_choice_empty_head_accepts_and_seeds_canonical_head() {
    let candidate = sample_canonical_record(5, "digest-b");
    let mut hook = DeterministicCompetingBranchForkChoiceHook::new();

    let decision = hook
        .evaluate_candidate(&candidate)
        .expect("empty-head candidate should evaluate");

    assert_eq!(decision, ForkChoiceDecision::Accept);
    assert_eq!(hook.canonical_head(), Some(&candidate));
}

#[test]
fn unit_fork_choice_higher_block_height_replaces_canonical_head() {
    let existing = sample_canonical_record(5, "digest-c");
    let candidate = sample_canonical_record(6, "digest-z");
    let mut hook = DeterministicCompetingBranchForkChoiceHook::with_canonical_head(existing);

    let decision = hook
        .evaluate_candidate(&candidate)
        .expect("higher-height candidate should evaluate");

    assert_eq!(decision, ForkChoiceDecision::Accept);
    assert_eq!(hook.canonical_head(), Some(&candidate));
}

#[test]
fn unit_fork_choice_stale_block_height_rejects_and_preserves_head() {
    let head = sample_canonical_record(9, "digest-c");
    let candidate = sample_canonical_record(8, "digest-a");
    let mut hook = DeterministicCompetingBranchForkChoiceHook::with_canonical_head(head.clone());

    let decision = hook
        .evaluate_candidate(&candidate)
        .expect("stale-height candidate should evaluate");

    assert_eq!(
        decision,
        ForkChoiceDecision::Reject {
            reason_code: "fork_choice_stale_block_height".to_owned(),
        }
    );
    assert_eq!(hook.canonical_head(), Some(&head));
}

#[test]
fn unit_fork_choice_duplicate_candidate_rejects_and_preserves_head() {
    let head = sample_canonical_record(7, "digest-b");
    let mut hook = DeterministicCompetingBranchForkChoiceHook::with_canonical_head(head.clone());

    let decision = hook
        .evaluate_candidate(&head)
        .expect("duplicate candidate should evaluate");

    assert_eq!(
        decision,
        ForkChoiceDecision::Reject {
            reason_code: "fork_choice_duplicate_candidate".to_owned(),
        }
    );
    assert_eq!(hook.canonical_head(), Some(&head));
}

#[test]
fn unit_fork_choice_lower_digest_at_same_height_replaces_canonical_head() {
    let head = sample_canonical_record(11, "digest-z");
    let candidate = sample_canonical_record(11, "digest-a");
    let mut hook = DeterministicCompetingBranchForkChoiceHook::with_canonical_head(head);

    let decision = hook
        .evaluate_candidate(&candidate)
        .expect("same-height lower digest candidate should evaluate");

    assert_eq!(decision, ForkChoiceDecision::Accept);
    assert_eq!(hook.canonical_head(), Some(&candidate));
}

#[test]
fn unit_fork_choice_higher_digest_at_same_height_rejects_and_preserves_head() {
    let head = sample_canonical_record(11, "digest-a");
    let candidate = sample_canonical_record(11, "digest-z");
    let mut hook = DeterministicCompetingBranchForkChoiceHook::with_canonical_head(head.clone());

    let decision = hook
        .evaluate_candidate(&candidate)
        .expect("same-height higher digest candidate should evaluate");

    assert_eq!(
        decision,
        ForkChoiceDecision::Reject {
            reason_code: "fork_choice_tie_break_loser".to_owned(),
        }
    );
    assert_eq!(hook.canonical_head(), Some(&head));
}

#[test]
fn unit_accept_all_fork_choice_hook_accepts_without_state() {
    let candidate = sample_canonical_record(3, "digest-accept-all");
    let mut hook = AcceptAllForkChoiceHook;

    let decision = hook
        .evaluate_candidate(&candidate)
        .expect("accept-all hook should evaluate");

    assert_eq!(decision, ForkChoiceDecision::Accept);
}
