#![allow(missing_docs)]

use super::restore_support::{validate_schema_version, RestoredMaps};
use super::snapshot_restore_validation::{
    validate_dependency_completion, validate_restored_dependencies,
};
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

fn restore_snapshot_maps(
    tasks: Vec<TaskOperationRecordSnapshot>,
) -> Result<RestoredMaps, TaskOperationError> {
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
) -> Result<
    (
        TaskOperationRecord,
        BTreeSet<String>,
        Vec<TaskOperationNoticeKind>,
    ),
    TaskOperationError,
> {
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

fn restore_record_lifecycle(
    task: &TaskOperationRecordSnapshot,
) -> Result<TaskLifecycle, TaskOperationError> {
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
