//! State-version migration planning and validation contracts.

use std::fmt;

use crate::state::StateVersion;

/// Single migration step between two adjacent state versions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationStep {
    /// Stable step identifier used in audit and policy checks.
    pub id: &'static str,
    /// Source state version for this step.
    pub from: StateVersion,
    /// Destination state version reached by this step.
    pub to: StateVersion,
    /// Human-readable description of the migration behavior.
    pub description: &'static str,
    /// Namespaces affected by this migration step.
    pub namespaces: &'static [&'static str],
}

impl MigrationStep {
    /// Builds a static migration step descriptor.
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

/// Ordered migration plan from one version to another.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationPlan {
    /// Requested source version.
    pub from: StateVersion,
    /// Requested target version.
    pub to: StateVersion,
    /// Ordered steps required to reach the target version.
    pub steps: Vec<MigrationStep>,
}

impl MigrationPlan {
    /// Validates continuity and target coverage of the migration plan.
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

/// Registry of known migration steps keyed by their source version.
#[derive(Debug, Clone, Default)]
pub struct MigrationRegistry {
    steps: Vec<MigrationStep>,
}

impl MigrationRegistry {
    /// Creates an empty migration registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a new migration step after range and ID validation.
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

    /// Builds a contiguous migration plan from source to target version.
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

/// Error returned when migration steps or plans are invalid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationError {
    /// Step ID already exists in the registry.
    DuplicateStepId(String),
    /// Step range is invalid because source is not lower than destination.
    InvalidStepRange {
        /// Step identifier with invalid range.
        id: &'static str,
        /// Source version supplied for the step.
        from: StateVersion,
        /// Destination version supplied for the step.
        to: StateVersion,
    },
    /// Requested plan range is invalid.
    InvalidPlanRange {
        /// Source version requested by the caller.
        from: StateVersion,
        /// Target version requested by the caller.
        to: StateVersion,
    },
    /// Registry lacks a required step to continue toward the target.
    MissingStep {
        /// Current version that requires a next step.
        from: StateVersion,
        /// Final target version requested by the caller.
        to: StateVersion,
    },
    /// Step sequence is not contiguous with prior step output.
    NonContiguousStep {
        /// Expected source version based on previous step.
        expected_from: StateVersion,
        /// Actual source version found on the next step.
        found_from: StateVersion,
    },
    /// Step destination exceeds requested migration target.
    StepOvershootsTarget {
        /// Step ID that overshoots the target.
        id: String,
        /// Requested target version.
        target: StateVersion,
        /// Step destination that exceeded the target.
        step_to: StateVersion,
    },
    /// Plan includes steps even though source and target are equal.
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
