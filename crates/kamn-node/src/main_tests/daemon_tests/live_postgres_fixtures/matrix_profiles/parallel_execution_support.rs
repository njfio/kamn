use super::super::models::*;
use super::super::*;
pub(crate) fn run_parallel_phase6_projections(
    leg_a_args: Vec<String>,
    leg_b_args: Vec<String>,
) -> (LivePostgresPhase6Projection, LivePostgresPhase6Projection) {
    let leg_a_handle = std::thread::spawn(move || run_daemon_for_phase6_projection(leg_a_args));
    let leg_b_handle = std::thread::spawn(move || run_daemon_for_phase6_projection(leg_b_args));
    let leg_a_projection = leg_a_handle
        .join()
        .expect("parallel role-pair lane leg A should complete");
    let leg_b_projection = leg_b_handle
        .join()
        .expect("parallel role-pair lane leg B should complete");
    (leg_a_projection, leg_b_projection)
}
