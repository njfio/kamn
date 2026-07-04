use crate::support::loading::load_waiver;
use crate::support::models::{Baseline, CurrentSurface, Evaluation, Thresholds};
use crate::support::paths::fail;

pub(crate) fn evaluate_policy(
    baseline: &Baseline,
    thresholds: &Thresholds,
    current: &CurrentSurface,
) -> Evaluation {
    let (shell_delta, ratio_delta) = policy_deltas(baseline, current);
    let mut fail_reasons = fail_reasons(thresholds, shell_delta, ratio_delta);
    if fail_reasons.is_empty() {
        return within_evaluation();
    }
    if let Some(evaluation) = waiver_evaluation(thresholds, shell_delta, ratio_delta) {
        return evaluation;
    }
    fail_reasons.push("ratio_fail_threshold_exceeded_unwaived");
    no_go_evaluation(fail_reasons)
}

fn policy_deltas(baseline: &Baseline, current: &CurrentSurface) -> (i64, f64) {
    (
        current.shell_test_file_count - baseline.shell_test_file_count,
        current.shell_to_rust_ratio - baseline.shell_to_rust_ratio,
    )
}

fn fail_reasons(thresholds: &Thresholds, shell_delta: i64, ratio_delta: f64) -> Vec<&'static str> {
    let mut reasons = Vec::new();
    push_if_exceeded(
        &mut reasons,
        shell_delta > thresholds.allowed_shell_test_file_delta_max,
        "shell_test_file_delta_exceeded",
    );
    push_if_exceeded(
        &mut reasons,
        ratio_delta > thresholds.allowed_ratio_delta_max,
        "ratio_delta_exceeded",
    );
    reasons
}

fn within_evaluation() -> Evaluation {
    Evaluation {
        policy_status: "within",
        final_decision: "GO",
        reason_codes: vec!["none"],
    }
}

fn no_go_evaluation(reason_codes: Vec<&'static str>) -> Evaluation {
    Evaluation {
        policy_status: "fail",
        final_decision: "NO-GO",
        reason_codes,
    }
}

fn push_if_exceeded(reasons: &mut Vec<&'static str>, exceeded: bool, reason: &'static str) {
    if exceeded {
        reasons.push(reason);
    }
}

fn waiver_evaluation(
    thresholds: &Thresholds,
    shell_delta: i64,
    ratio_delta: f64,
) -> Option<Evaluation> {
    let waiver_file = thresholds.waiver_file.as_ref()?;
    if !waiver_file.is_file() {
        return None;
    }
    let waiver = load_waiver(waiver_file);
    if within_waiver_cap(&waiver, shell_delta, ratio_delta) {
        let _ = &waiver.mitigation_issue;
        return Some(waiver_applied_evaluation());
    }
    waiver_cap_exceeded(shell_delta, ratio_delta, waiver_file)
}

fn within_waiver_cap(
    waiver: &crate::support::models::Waiver,
    shell_delta: i64,
    ratio_delta: f64,
) -> bool {
    shell_delta <= waiver.max_shell_test_file_delta && ratio_delta <= waiver.max_ratio_delta
}

fn waiver_applied_evaluation() -> Evaluation {
    Evaluation {
        policy_status: "waiver-applied",
        final_decision: "GO",
        reason_codes: vec!["ratio_fail_threshold_waiver_applied"],
    }
}

fn waiver_cap_exceeded(shell_delta: i64, ratio_delta: f64, waiver_file: &std::path::Path) -> ! {
    let waiver_display = waiver_file.display();
    fail(
        "waiver_cap_exceeded",
        &format!(
            "shell_delta={shell_delta} ratio_delta={ratio_delta:.6} exceeds waiver cap in {waiver_display}"
        ),
    )
}
