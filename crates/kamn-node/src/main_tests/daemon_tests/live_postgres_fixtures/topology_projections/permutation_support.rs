use super::super::models::*;
fn rotate_left_one(
    mut lanes: Vec<LivePostgresRolePairProfile>,
) -> Vec<LivePostgresRolePairProfile> {
    if !lanes.is_empty() {
        lanes.rotate_left(1);
    }
    lanes
}

fn interleave_even_then_odd(
    lanes: Vec<LivePostgresRolePairProfile>,
) -> Vec<LivePostgresRolePairProfile> {
    let mut even = Vec::with_capacity(lanes.len());
    let mut odd = Vec::with_capacity(lanes.len());
    for (idx, lane) in lanes.into_iter().enumerate() {
        if idx % 2 == 0 {
            even.push(lane);
        } else {
            odd.push(lane);
        }
    }
    even.extend(odd);
    even
}

pub(crate) fn permute_role_pair_lanes(
    mut lanes: Vec<LivePostgresRolePairProfile>,
    permutation_id: &str,
) -> Vec<LivePostgresRolePairProfile> {
    match permutation_id {
        "baseline" => lanes,
        "reverse" => {
            lanes.reverse();
            lanes
        }
        "rotate_left_1" => rotate_left_one(lanes),
        "interleaved_even_then_odd" => interleave_even_then_odd(lanes),
        _ => panic!("unknown lane permutation: {permutation_id}"),
    }
}
