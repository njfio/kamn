#![allow(missing_docs)]

use super::transition_support::{
    apply_transition, ensure_assignable, ensure_reason_present, ensure_requester_or_assignee,
    ready_task_id,
};
use super::*;

impl TaskOperationEngine {
    pub fn accept(&mut self, task_id: &str, actor: &str) -> Result<(), TaskOperationError> {
        validate_did(actor)?;
        let record = self.task_mut(task_id)?;
        ensure_assignable(record, actor)?;
        apply_transition(record, TaskTransition::Accept)?;
        record.assignee = Some(actor.to_owned());
        self.push_notice(task_id, TaskOperationNoticeKind::Accepted);
        Ok(())
    }

    pub fn delegate(
        &mut self,
        task_id: &str,
        actor: &str,
        delegatee: &str,
    ) -> Result<(), TaskOperationError> {
        validate_did(actor)?;
        validate_did(delegatee)?;
        let record = self.task_mut(task_id)?;
        Self::require_assignee(record, actor)?;
        apply_transition(record, TaskTransition::Delegate)?;
        record.assignee = Some(delegatee.to_owned());
        self.push_notice(task_id, TaskOperationNoticeKind::Delegated);
        Ok(())
    }

    pub fn start_work(&mut self, task_id: &str, actor: &str) -> Result<(), TaskOperationError> {
        validate_did(actor)?;
        if let Some(dependency_id) = self.unsatisfied_dependency(task_id)? {
            return Err(TaskOperationError::DependencyNotSatisfied {
                task_id: task_id.to_owned(),
                dependency_id,
            });
        }
        let record = self.task_mut(task_id)?;
        Self::require_assignee(record, actor)?;
        apply_transition(record, TaskTransition::StartWork)?;
        self.push_notice(task_id, TaskOperationNoticeKind::Started);
        Ok(())
    }

    pub fn block(
        &mut self,
        task_id: &str,
        actor: &str,
        reason: &str,
    ) -> Result<(), TaskOperationError> {
        self.transition_with_reason(task_id, actor, reason, "block", TaskTransition::Block)?;
        self.push_notice(task_id, TaskOperationNoticeKind::Blocked);
        Ok(())
    }

    pub fn request_input(
        &mut self,
        task_id: &str,
        actor: &str,
        reason: &str,
    ) -> Result<(), TaskOperationError> {
        self.transition_with_reason(
            task_id,
            actor,
            reason,
            "request_input",
            TaskTransition::RequestInput,
        )?;
        self.push_notice(task_id, TaskOperationNoticeKind::InputRequired);
        Ok(())
    }

    pub fn complete(&mut self, task_id: &str, actor: &str) -> Result<(), TaskOperationError> {
        validate_did(actor)?;
        let record = self.task_mut(task_id)?;
        Self::require_assignee(record, actor)?;
        apply_transition(record, TaskTransition::Complete)?;
        self.push_notice(task_id, TaskOperationNoticeKind::Completed);
        Ok(())
    }

    pub fn fail(
        &mut self,
        task_id: &str,
        actor: &str,
        reason: &str,
    ) -> Result<(), TaskOperationError> {
        self.transition_with_reason(task_id, actor, reason, "fail", TaskTransition::Fail)?;
        self.push_notice(task_id, TaskOperationNoticeKind::Failed);
        Ok(())
    }

    pub fn cancel(&mut self, task_id: &str, actor: &str) -> Result<(), TaskOperationError> {
        validate_did(actor)?;
        let record = self.task_mut(task_id)?;
        ensure_requester_or_assignee(record, actor)?;
        apply_transition(record, TaskTransition::Cancel)?;
        self.push_notice(task_id, TaskOperationNoticeKind::Cancelled);
        Ok(())
    }

    pub fn task(&self, task_id: &str) -> Result<&TaskOperationRecord, TaskOperationError> {
        self.tasks
            .get(task_id)
            .ok_or_else(|| TaskOperationError::NotFound(task_id.to_owned()))
    }

    pub fn notices(&self, task_id: &str) -> Vec<TaskOperationNoticeKind> {
        self.notices_by_task.get(task_id).cloned().unwrap_or_default()
    }

    pub fn ready_tasks(&self) -> Vec<String> {
        self.tasks
            .iter()
            .filter_map(|(task_id, record)| ready_task_id(self, task_id, record))
            .collect()
    }

    fn task_mut(&mut self, task_id: &str) -> Result<&mut TaskOperationRecord, TaskOperationError> {
        self.tasks
            .get_mut(task_id)
            .ok_or_else(|| TaskOperationError::NotFound(task_id.to_owned()))
    }

    fn require_assignee(
        record: &TaskOperationRecord,
        actor: &str,
    ) -> Result<(), TaskOperationError> {
        if record.assignee.as_deref() != Some(actor) {
            return Err(TaskOperationError::UnauthorizedActor {
                actor: actor.to_owned(),
                required: "assignee",
            });
        }
        Ok(())
    }

    pub(super) fn push_notice(&mut self, task_id: &str, notice: TaskOperationNoticeKind) {
        self.notices_by_task
            .entry(task_id.to_owned())
            .or_default()
            .push(notice);
    }

    pub(super) fn unsatisfied_dependency(&self, task_id: &str) -> Result<Option<String>, TaskOperationError> {
        if !self.tasks.contains_key(task_id) {
            return Err(TaskOperationError::NotFound(task_id.to_owned()));
        }
        let Some(dependencies) = self.dependencies_by_task.get(task_id) else {
            return Ok(None);
        };
        for dependency_id in dependencies {
            let dependency = self.tasks.get(dependency_id).ok_or_else(|| {
                TaskOperationError::UnknownDependency {
                    task_id: task_id.to_owned(),
                    dependency_id: dependency_id.clone(),
                }
            })?;
            if dependency.lifecycle.state() != TaskState::Completed {
                return Ok(Some(dependency_id.clone()));
            }
        }
        Ok(None)
    }

    fn transition_with_reason(
        &mut self,
        task_id: &str,
        actor: &str,
        reason: &str,
        action: &'static str,
        transition: TaskTransition,
    ) -> Result<(), TaskOperationError> {
        validate_did(actor)?;
        ensure_reason_present(reason, action)?;
        let record = self.task_mut(task_id)?;
        Self::require_assignee(record, actor)?;
        apply_transition(record, transition)
    }
}
