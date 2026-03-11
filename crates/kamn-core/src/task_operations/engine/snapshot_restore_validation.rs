#![allow(missing_docs)]

use super::restore_support::restored_task_state;
use super::*;

pub(super) fn validate_restored_dependencies(
    restored_tasks: &BTreeMap<String, TaskOperationRecord>,
    restored_dependencies: &BTreeMap<String, BTreeSet<String>>,
) -> Result<(), TaskOperationError> {
    for (task_id, dependencies) in restored_dependencies {
        for dependency_id in dependencies {
            if !restored_tasks.contains_key(dependency_id) {
                return Err(TaskOperationError::UnknownDependency {
                    task_id: task_id.clone(),
                    dependency_id: dependency_id.clone(),
                });
            }
        }
    }
    if let Some(task_id) = super::support::detect_cycle_task_id(restored_dependencies) {
        return Err(TaskOperationError::CyclicDependency { task_id });
    }
    Ok(())
}

pub(super) fn validate_dependency_completion(
    restored_tasks: &BTreeMap<String, TaskOperationRecord>,
    restored_dependencies: &BTreeMap<String, BTreeSet<String>>,
) -> Result<(), TaskOperationError> {
    for (task_id, dependencies) in restored_dependencies {
        validate_task_dependencies(restored_tasks, task_id, dependencies)?;
    }
    Ok(())
}

fn validate_task_dependencies(
    restored_tasks: &BTreeMap<String, TaskOperationRecord>,
    task_id: &str,
    dependencies: &BTreeSet<String>,
) -> Result<(), TaskOperationError> {
    let task_state = restored_task_state(restored_tasks, task_id)?;
    if !super::support::requires_completed_dependencies(task_state) {
        return Ok(());
    }
    for dependency_id in dependencies {
        validate_completed_dependency(restored_tasks, task_id, dependency_id)?;
    }
    Ok(())
}

fn validate_completed_dependency(
    restored_tasks: &BTreeMap<String, TaskOperationRecord>,
    task_id: &str,
    dependency_id: &str,
) -> Result<(), TaskOperationError> {
    let dependency_state = restored_dependency_state(restored_tasks, task_id, dependency_id)?;
    if dependency_state == TaskState::Completed {
        return Ok(());
    }
    Err(TaskOperationError::SnapshotDependencyNotCompleted {
        task_id: task_id.to_owned(),
        dependency_id: dependency_id.to_owned(),
        dependency_state,
    })
}

fn restored_dependency_state(
    restored_tasks: &BTreeMap<String, TaskOperationRecord>,
    task_id: &str,
    dependency_id: &str,
) -> Result<TaskState, TaskOperationError> {
    restored_task_state(restored_tasks, dependency_id).map_err(|_| {
        TaskOperationError::UnknownDependency {
            task_id: task_id.to_owned(),
            dependency_id: dependency_id.to_owned(),
        }
    })
}
