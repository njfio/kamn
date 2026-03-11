use super::*;

impl TaskOperationEngine {
    /// Submit a single task with requester and description metadata.
    pub fn submit(
        &mut self,
        task_id: &str,
        requester: &str,
        description: &str,
    ) -> Result<(), TaskOperationError> {
        validate_submission_input(self, task_id, requester, description)?;
        let record = build_submitted_record(task_id, requester, description)?;
        self.insert_submitted_task(task_id, record);
        self.push_notice(task_id, TaskOperationNoticeKind::Submitted);
        Ok(())
    }

    /// Submit a batch of tasks and validate dependency graph integrity.
    pub fn submit_swarm_tasks(
        &mut self,
        drafts: Vec<SwarmTaskDraft>,
    ) -> Result<(), TaskOperationError> {
        if drafts.is_empty() {
            return Err(TaskOperationError::EmptySwarmTaskSet);
        }
        let (graph, draft_ids) = build_validated_draft_graph(self, &drafts)?;
        if let Some(task_id) = super::support::detect_cycle_task_id(&graph) {
            return Err(TaskOperationError::CyclicDependency { task_id });
        }
        let mut dependencies_by_task = build_dependency_map(&drafts, &graph);
        for draft in drafts {
            self.submit(&draft.task_id, &draft.requester, &draft.description)?;
            let dependencies = dependencies_by_task
                .remove(&draft.task_id)
                .unwrap_or_default();
            self.dependencies_by_task
                .insert(draft.task_id, dependencies);
        }
        let _ = draft_ids;
        Ok(())
    }
}

fn validate_submission_input(
    engine: &TaskOperationEngine,
    task_id: &str,
    requester: &str,
    description: &str,
) -> Result<(), TaskOperationError> {
    if engine.tasks.contains_key(task_id) {
        return Err(TaskOperationError::DuplicateTaskId(task_id.to_owned()));
    }
    validate_did(requester)?;
    if description.trim().is_empty() {
        return Err(TaskOperationError::EmptyDescription);
    }
    Ok(())
}

fn build_submitted_record(
    task_id: &str,
    requester: &str,
    description: &str,
) -> Result<TaskOperationRecord, TaskOperationError> {
    let lifecycle = TaskLifecycle::new(task_id).map_err(lifecycle_error)?;
    Ok(TaskOperationRecord {
        task_id: task_id.to_owned(),
        requester: requester.to_owned(),
        assignee: None,
        description: description.to_owned(),
        lifecycle,
    })
}

fn build_validated_draft_graph(
    engine: &TaskOperationEngine,
    drafts: &[SwarmTaskDraft],
) -> Result<(BTreeMap<String, BTreeSet<String>>, BTreeSet<String>), TaskOperationError> {
    let mut graph = BTreeMap::new();
    let mut draft_ids = BTreeSet::new();
    for draft in drafts {
        validate_unique_draft(engine, draft, &mut draft_ids)?;
        graph.insert(draft.task_id.clone(), validate_unique_dependencies(draft)?);
    }
    validate_known_dependencies(engine, &graph, &draft_ids)?;
    Ok((graph, draft_ids))
}

fn validate_unique_draft(
    engine: &TaskOperationEngine,
    draft: &SwarmTaskDraft,
    draft_ids: &mut BTreeSet<String>,
) -> Result<(), TaskOperationError> {
    if engine.tasks.contains_key(&draft.task_id) || !draft_ids.insert(draft.task_id.clone()) {
        return Err(TaskOperationError::DuplicateTaskId(draft.task_id.clone()));
    }
    validate_did(&draft.requester)?;
    if draft.description.trim().is_empty() {
        return Err(TaskOperationError::EmptyDescription);
    }
    Ok(())
}

fn validate_unique_dependencies(
    draft: &SwarmTaskDraft,
) -> Result<BTreeSet<String>, TaskOperationError> {
    let mut unique_dependencies = BTreeSet::new();
    for dependency_id in &draft.dependencies {
        if !unique_dependencies.insert(dependency_id.clone()) {
            return Err(TaskOperationError::DuplicateDependency {
                task_id: draft.task_id.clone(),
                dependency_id: dependency_id.clone(),
            });
        }
    }
    Ok(unique_dependencies)
}

fn validate_known_dependencies(
    engine: &TaskOperationEngine,
    graph: &BTreeMap<String, BTreeSet<String>>,
    draft_ids: &BTreeSet<String>,
) -> Result<(), TaskOperationError> {
    for (task_id, dependencies) in graph {
        for dependency_id in dependencies {
            if !draft_ids.contains(dependency_id) && !engine.tasks.contains_key(dependency_id) {
                return Err(TaskOperationError::UnknownDependency {
                    task_id: task_id.clone(),
                    dependency_id: dependency_id.clone(),
                });
            }
        }
    }
    Ok(())
}

fn build_dependency_map(
    drafts: &[SwarmTaskDraft],
    graph: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut dependencies_by_task = BTreeMap::new();
    for draft in drafts {
        dependencies_by_task.insert(
            draft.task_id.clone(),
            graph.get(&draft.task_id).cloned().unwrap_or_default(),
        );
    }
    dependencies_by_task
}

impl TaskOperationEngine {
    fn insert_submitted_task(&mut self, task_id: &str, record: TaskOperationRecord) {
        self.tasks.insert(task_id.to_owned(), record);
        self.dependencies_by_task
            .entry(task_id.to_owned())
            .or_default();
    }
}
