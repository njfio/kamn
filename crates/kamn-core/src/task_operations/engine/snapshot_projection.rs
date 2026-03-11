#![allow(missing_docs)]

use super::restore_support::{restored_task_state, validate_schema_version, RestoredMaps};
use super::*;

impl TaskOperationEngine {
    pub fn export_snapshot(&self) -> TaskOperationSnapshot {
        let tasks = self
            .tasks
            .iter()
            .map(|(task_id, record)| TaskOperationRecordSnapshot {
                task_id: task_id.clone(),
                requester: record.requester.clone(),
                assignee: record.assignee.clone(),
                description: record.description.clone(),
                lifecycle_history: record.lifecycle.history(),
                dependencies: self
                    .dependencies_by_task
                    .get(task_id)
                    .map(|values| values.iter().cloned().collect())
                    .unwrap_or_default(),
                notices: self.notices(task_id),
            })
            .collect();
        TaskOperationSnapshot {
            schema_version: TASK_OPERATION_SNAPSHOT_SCHEMA_VERSION,
            tasks,
        }
    }

    pub fn restore_snapshot(
        &mut self,
        snapshot: TaskOperationSnapshot,
    ) -> Result<(), TaskOperationError> {
        validate_schema_version(snapshot.schema_version)?;
        let (restored_tasks, restored_notices, restored_dependencies) =
            restore_snapshot_maps(snapshot.tasks)?;
        validate_restored_dependencies(&restored_tasks, &restored_dependencies)?;
        validate_dependency_completion(&restored_tasks, &restored_dependencies)?;
        self.tasks = restored_tasks;
        self.notices_by_task = restored_notices;
        self.dependencies_by_task = restored_dependencies;
        Ok(())
    }
}

fn restore_snapshot_maps(tasks: Vec<TaskOperationRecordSnapshot>) -> Result<RestoredMaps, TaskOperationError> {
    let mut restored_tasks = BTreeMap::new();
    let mut restored_notices = BTreeMap::new();
    let mut restored_dependencies = BTreeMap::new();
    for task in tasks {
        let task_id = task.task_id.clone();
        if restored_tasks.contains_key(&task_id) {
            return Err(TaskOperationError::DuplicateTaskId(task_id));
        }
        let (record, dependency_set, notices) = restore_snapshot_record(task)?;
        let task_id = record.task_id.clone();
        restored_notices.insert(task_id.clone(), notices);
        restored_dependencies.insert(task_id.clone(), dependency_set);
        restored_tasks.insert(task_id, record);
    }
    Ok((restored_tasks, restored_notices, restored_dependencies))
}

fn restore_snapshot_record(
    task: TaskOperationRecordSnapshot,
) -> Result<(TaskOperationRecord, BTreeSet<String>, Vec<TaskOperationNoticeKind>), TaskOperationError> {
    validate_record_metadata(&task)?;
    let lifecycle = restore_record_lifecycle(&task)?;
    let dependency_set = restore_dependency_set(&task)?;
    Ok((
        TaskOperationRecord {
            task_id: task.task_id,
            requester: task.requester,
            assignee: task.assignee,
            description: task.description,
            lifecycle,
        },
        dependency_set,
        task.notices,
    ))
}

fn validate_record_metadata(task: &TaskOperationRecordSnapshot) -> Result<(), TaskOperationError> {
    super::validate_did(&task.requester)?;
    if let Some(assignee) = &task.assignee {
        super::validate_did(assignee)?;
    }
    if task.description.trim().is_empty() {
        return Err(TaskOperationError::InvalidSnapshot(format!(
            "task {} has empty description",
            task.task_id
        )));
    }
    Ok(())
}

fn restore_record_lifecycle(task: &TaskOperationRecordSnapshot) -> Result<TaskLifecycle, TaskOperationError> {
    TaskLifecycle::restore(&task.task_id, task.lifecycle_history.clone()).map_err(|error| {
        TaskOperationError::InvalidSnapshot(format!(
            "task {} has invalid lifecycle history: {error}",
            task.task_id
        ))
    })
}

fn restore_dependency_set(
    task: &TaskOperationRecordSnapshot,
) -> Result<BTreeSet<String>, TaskOperationError> {
    let mut dependency_set = BTreeSet::new();
    for dependency_id in &task.dependencies {
        if !dependency_set.insert(dependency_id.clone()) {
            return Err(TaskOperationError::DuplicateDependency {
                task_id: task.task_id.clone(),
                dependency_id: dependency_id.clone(),
            });
        }
    }
    Ok(dependency_set)
}

fn validate_restored_dependencies(
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

fn validate_dependency_completion(
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
    restored_task_state(restored_tasks, dependency_id).map_err(|_| TaskOperationError::UnknownDependency {
        task_id: task_id.to_owned(),
        dependency_id: dependency_id.to_owned(),
    })
}
