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

fn evaluate_decision(
    hook: &mut impl ForkChoiceHook,
    candidate: &CanonicalCommitRecord,
    context: &str,
) -> ForkChoiceDecision {
    hook.evaluate_candidate(candidate).expect(context)
}

fn assert_canonical_head(
    hook: &DeterministicCompetingBranchForkChoiceHook,
    expected: &CanonicalCommitRecord,
) {
    assert_eq!(hook.canonical_head(), Some(expected));
}

fn assert_rejected(decision: ForkChoiceDecision, reason_code: &str) {
    assert_eq!(
        decision,
        ForkChoiceDecision::Reject {
            reason_code: reason_code.to_owned(),
        }
    );
}

#[test]
fn unit_fork_choice_empty_head_accepts_and_seeds_canonical_head() {
    let candidate = sample_canonical_record(5, "digest-b");
    let mut hook = DeterministicCompetingBranchForkChoiceHook::new();

    let decision = evaluate_decision(
        &mut hook,
        &candidate,
        "empty-head candidate should evaluate",
    );

    assert_eq!(decision, ForkChoiceDecision::Accept);
    assert_canonical_head(&hook, &candidate);
}

#[test]
fn unit_fork_choice_higher_block_height_replaces_canonical_head() {
    let existing = sample_canonical_record(5, "digest-c");
    let candidate = sample_canonical_record(6, "digest-z");
    let mut hook = DeterministicCompetingBranchForkChoiceHook::with_canonical_head(existing);

    let decision = evaluate_decision(
        &mut hook,
        &candidate,
        "higher-height candidate should evaluate",
    );

    assert_eq!(decision, ForkChoiceDecision::Accept);
    assert_canonical_head(&hook, &candidate);
}

#[test]
fn unit_fork_choice_stale_block_height_rejects_and_preserves_head() {
    let head = sample_canonical_record(9, "digest-c");
    let candidate = sample_canonical_record(8, "digest-a");
    let mut hook = DeterministicCompetingBranchForkChoiceHook::with_canonical_head(head.clone());

    let decision = evaluate_decision(
        &mut hook,
        &candidate,
        "stale-height candidate should evaluate",
    );

    assert_rejected(decision, "fork_choice_stale_block_height");
    assert_canonical_head(&hook, &head);
}

#[test]
fn unit_fork_choice_duplicate_candidate_rejects_and_preserves_head() {
    let head = sample_canonical_record(7, "digest-b");
    let mut hook = DeterministicCompetingBranchForkChoiceHook::with_canonical_head(head.clone());

    let decision = evaluate_decision(&mut hook, &head, "duplicate candidate should evaluate");

    assert_rejected(decision, "fork_choice_duplicate_candidate");
    assert_canonical_head(&hook, &head);
}

#[test]
fn unit_fork_choice_lower_digest_at_same_height_replaces_canonical_head() {
    let head = sample_canonical_record(11, "digest-z");
    let candidate = sample_canonical_record(11, "digest-a");
    let mut hook = DeterministicCompetingBranchForkChoiceHook::with_canonical_head(head);

    let decision = evaluate_decision(
        &mut hook,
        &candidate,
        "same-height lower digest candidate should evaluate",
    );

    assert_eq!(decision, ForkChoiceDecision::Accept);
    assert_canonical_head(&hook, &candidate);
}

#[test]
fn unit_fork_choice_higher_digest_at_same_height_rejects_and_preserves_head() {
    let head = sample_canonical_record(11, "digest-a");
    let candidate = sample_canonical_record(11, "digest-z");
    let mut hook = DeterministicCompetingBranchForkChoiceHook::with_canonical_head(head.clone());

    let decision = evaluate_decision(
        &mut hook,
        &candidate,
        "same-height higher digest candidate should evaluate",
    );

    assert_rejected(decision, "fork_choice_tie_break_loser");
    assert_canonical_head(&hook, &head);
}

#[test]
fn unit_accept_all_fork_choice_hook_accepts_without_state() {
    let candidate = sample_canonical_record(3, "digest-accept-all");
    let mut hook = AcceptAllForkChoiceHook;

    let decision = evaluate_decision(&mut hook, &candidate, "accept-all hook should evaluate");

    assert_eq!(decision, ForkChoiceDecision::Accept);
}
