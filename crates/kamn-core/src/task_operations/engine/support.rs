use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VisitState {
    Visiting,
    Visited,
}

pub(super) fn detect_cycle_task_id(graph: &BTreeMap<String, BTreeSet<String>>) -> Option<String> {
    let mut states = BTreeMap::new();
    for task_id in graph.keys() {
        if let Some(cycle_task_id) = visit_cycle_task(task_id, graph, &mut states) {
            return Some(cycle_task_id);
        }
    }
    None
}

fn visit_cycle_task(
    task_id: &str,
    graph: &BTreeMap<String, BTreeSet<String>>,
    states: &mut BTreeMap<String, VisitState>,
) -> Option<String> {
    if states.contains_key(task_id) {
        return None;
    }
    states.insert(task_id.to_owned(), VisitState::Visiting);
    let cycle = visit_dependencies(task_id, graph, states);
    states.insert(task_id.to_owned(), VisitState::Visited);
    cycle
}

fn visit_dependencies(
    task_id: &str,
    graph: &BTreeMap<String, BTreeSet<String>>,
    states: &mut BTreeMap<String, VisitState>,
) -> Option<String> {
    for dependency_id in graph.get(task_id).into_iter().flatten() {
        if let Some(cycle) = inspect_dependency(dependency_id, graph, states) {
            return Some(cycle);
        }
    }
    None
}

fn inspect_dependency(
    dependency_id: &str,
    graph: &BTreeMap<String, BTreeSet<String>>,
    states: &mut BTreeMap<String, VisitState>,
) -> Option<String> {
    if !graph.contains_key(dependency_id) {
        return None;
    }
    match states.get(dependency_id) {
        Some(VisitState::Visiting) => Some(dependency_id.to_owned()),
        Some(VisitState::Visited) => None,
        None => visit_cycle_task(dependency_id, graph, states),
    }
}

pub(super) fn requires_completed_dependencies(state: TaskState) -> bool {
    matches!(
        state,
        TaskState::InProgress
            | TaskState::InputRequired
            | TaskState::Blocked
            | TaskState::Completed
            | TaskState::Failed
    )
}
