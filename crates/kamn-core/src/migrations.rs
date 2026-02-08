use std::fmt;

use crate::state::StateVersion;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationStep {
    pub id: &'static str,
    pub from: StateVersion,
    pub to: StateVersion,
    pub description: &'static str,
    pub namespaces: &'static [&'static str],
}

impl MigrationStep {
    pub const fn new(
        id: &'static str,
        from: StateVersion,
        to: StateVersion,
        description: &'static str,
        namespaces: &'static [&'static str],
    ) -> Self {
        Self {
            id,
            from,
            to,
            description,
            namespaces,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationPlan {
    pub from: StateVersion,
    pub to: StateVersion,
    pub steps: Vec<MigrationStep>,
}

impl MigrationPlan {
    pub fn validate(&self) -> Result<(), MigrationError> {
        if self.from == self.to {
            if self.steps.is_empty() {
                return Ok(());
            }
            return Err(MigrationError::UnexpectedStepsForSameVersion);
        }

        if self.steps.is_empty() {
            return Err(MigrationError::MissingStep {
                from: self.from,
                to: self.to,
            });
        }

        let mut current = self.from;
        for step in &self.steps {
            if step.from != current {
                return Err(MigrationError::NonContiguousStep {
                    expected_from: current,
                    found_from: step.from,
                });
            }
            current = step.to;
        }

        if current != self.to {
            return Err(MigrationError::MissingStep {
                from: current,
                to: self.to,
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct MigrationRegistry {
    steps: Vec<MigrationStep>,
}

impl MigrationRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, step: MigrationStep) -> Result<(), MigrationError> {
        if step.from >= step.to {
            return Err(MigrationError::InvalidStepRange {
                id: step.id,
                from: step.from,
                to: step.to,
            });
        }

        if self.steps.iter().any(|existing| existing.id == step.id) {
            return Err(MigrationError::DuplicateStepId(step.id.to_owned()));
        }

        self.steps.push(step);
        self.steps.sort_by_key(|item| item.from);
        Ok(())
    }

    pub fn build_plan(
        &self,
        from: StateVersion,
        to: StateVersion,
    ) -> Result<MigrationPlan, MigrationError> {
        if from > to {
            return Err(MigrationError::InvalidPlanRange { from, to });
        }

        if from == to {
            return Ok(MigrationPlan {
                from,
                to,
                steps: Vec::new(),
            });
        }

        let mut steps = Vec::new();
        let mut current = from;

        while current < to {
            let step = self
                .steps
                .iter()
                .find(|candidate| candidate.from == current)
                .ok_or(MigrationError::MissingStep { from: current, to })?
                .clone();

            if step.to > to {
                return Err(MigrationError::StepOvershootsTarget {
                    id: step.id.to_owned(),
                    target: to,
                    step_to: step.to,
                });
            }

            current = step.to;
            steps.push(step);
        }

        let plan = MigrationPlan { from, to, steps };
        plan.validate()?;
        Ok(plan)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationError {
    DuplicateStepId(String),
    InvalidStepRange {
        id: &'static str,
        from: StateVersion,
        to: StateVersion,
    },
    InvalidPlanRange {
        from: StateVersion,
        to: StateVersion,
    },
    MissingStep {
        from: StateVersion,
        to: StateVersion,
    },
    NonContiguousStep {
        expected_from: StateVersion,
        found_from: StateVersion,
    },
    StepOvershootsTarget {
        id: String,
        target: StateVersion,
        step_to: StateVersion,
    },
    UnexpectedStepsForSameVersion,
}

impl fmt::Display for MigrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateStepId(id) => write!(f, "duplicate migration step id: {id}"),
            Self::InvalidStepRange { id, from, to } => {
                write!(
                    f,
                    "invalid migration step range for {id}: {:?} -> {:?}",
                    from, to
                )
            }
            Self::InvalidPlanRange { from, to } => {
                write!(f, "invalid migration plan range: {:?} -> {:?}", from, to)
            }
            Self::MissingStep { from, to } => {
                write!(
                    f,
                    "missing migration step to progress from {:?} toward {:?}",
                    from, to
                )
            }
            Self::NonContiguousStep {
                expected_from,
                found_from,
            } => write!(
                f,
                "non-contiguous migration step: expected {:?}, found {:?}",
                expected_from, found_from
            ),
            Self::StepOvershootsTarget {
                id,
                target,
                step_to,
            } => write!(
                f,
                "migration step {id} overshoots target {:?} with {:?}",
                target, step_to
            ),
            Self::UnexpectedStepsForSameVersion => {
                write!(f, "migration plan for same version must not contain steps")
            }
        }
    }
}

impl std::error::Error for MigrationError {}

#[cfg(test)]
mod tests {
    use super::{MigrationError, MigrationRegistry, MigrationStep};
    use crate::state::StateVersion;

    const NAMESPACES: &[&str] = &["kamn.tasks.state"];

    #[test]
    fn registry_rejects_duplicate_ids() {
        let mut registry = MigrationRegistry::new();
        let step = MigrationStep::new(
            "tasks-v1-v2",
            StateVersion(1),
            StateVersion(2),
            "Upgrade task schema",
            NAMESPACES,
        );

        registry.register(step.clone()).expect("first insert works");
        assert_eq!(
            registry.register(step),
            Err(MigrationError::DuplicateStepId("tasks-v1-v2".to_owned()))
        );
    }

    #[test]
    fn registry_builds_contiguous_plan() {
        let mut registry = MigrationRegistry::new();
        registry
            .register(MigrationStep::new(
                "tasks-v1-v2",
                StateVersion(1),
                StateVersion(2),
                "Upgrade task schema",
                NAMESPACES,
            ))
            .expect("register first step");
        registry
            .register(MigrationStep::new(
                "tasks-v2-v3",
                StateVersion(2),
                StateVersion(3),
                "Second task schema update",
                NAMESPACES,
            ))
            .expect("register second step");

        let plan = registry
            .build_plan(StateVersion(1), StateVersion(3))
            .expect("build contiguous plan");

        assert_eq!(plan.steps.len(), 2);
        assert!(plan.validate().is_ok());
    }

    #[test]
    fn registry_rejects_missing_step() {
        let mut registry = MigrationRegistry::new();
        registry
            .register(MigrationStep::new(
                "tasks-v1-v2",
                StateVersion(1),
                StateVersion(2),
                "Upgrade task schema",
                NAMESPACES,
            ))
            .expect("register step");

        // Regression: #17
        assert_eq!(
            registry.build_plan(StateVersion(1), StateVersion(3)),
            Err(MigrationError::MissingStep {
                from: StateVersion(2),
                to: StateVersion(3)
            })
        );
    }

    #[test]
    fn plan_for_same_version_is_empty() {
        let registry = MigrationRegistry::new();
        let plan = registry
            .build_plan(StateVersion(2), StateVersion(2))
            .expect("same version requires no steps");
        assert!(plan.steps.is_empty());
        assert!(plan.validate().is_ok());
    }
}
