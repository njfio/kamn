use crate::DataLayerM8ComplianceRegistry;

use super::super::{
    DataLayerM10PartitionLifecycleError, DataLayerM10PartitionLifecycleRegistry,
    DataLayerM10Phase6SchedulerRuntime, DataLayerM10Phase6SchedulerRuntimeState,
    DataLayerM10Phase6ExecutionTickBudget, DataLayerM10Phase6ExecutionTickRequest,
    DataLayerM10Phase6SchedulerCycleReport, DataLayerM10Phase6SchedulerCycleRequest,
    DataLayerM10Phase6SchedulerPolicy,
    DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_APPLIED_REASON_CODE,
    DATA_LAYER_M10_PHASE6_SCHEDULER_RUNTIME_INITIALIZED_REASON_CODE,
};
use super::scheduler::orchestration::data_layer_m10_execute_phase6_scheduler_cycle;
use super::scheduler::budget::validate_phase6_execution_tick_budget;
use super::scheduler::trigger::{
    phase6_scheduler_error_reason_code, validate_phase6_scheduler_policy,
    validate_phase6_scheduler_runtime_clock,
};

impl DataLayerM10Phase6SchedulerPolicy {
    /// Creates a scheduler policy and validates threshold/interval values.
    pub fn new(
        due_candidate_trigger_threshold: usize,
        max_tick_interval_seconds: u64,
    ) -> Result<Self, DataLayerM10PartitionLifecycleError> {
        let policy = Self {
            due_candidate_trigger_threshold,
            max_tick_interval_seconds,
        };
        validate_phase6_scheduler_policy(policy)?;
        Ok(policy)
    }
}

impl DataLayerM10Phase6SchedulerRuntime {
    /// Creates a stateful scheduler runtime with deterministic zeroed counters.
    pub fn new(
        scheduler_policy: DataLayerM10Phase6SchedulerPolicy,
        budget: DataLayerM10Phase6ExecutionTickBudget,
    ) -> Result<Self, DataLayerM10PartitionLifecycleError> {
        validate_phase6_scheduler_policy(scheduler_policy)?;
        validate_phase6_execution_tick_budget(budget)?;
        Ok(Self {
            scheduler_policy,
            budget,
            state: DataLayerM10Phase6SchedulerRuntimeState {
                last_successful_tick_epoch_seconds: None,
                last_observed_now_epoch_seconds: None,
                total_cycles: 0,
                executed_cycles: 0,
                deferred_cycles: 0,
                fail_closed_cycles: 0,
                last_reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_RUNTIME_INITIALIZED_REASON_CODE,
            },
        })
    }

    /// Returns an immutable snapshot of runtime scheduler state.
    pub fn state(&self) -> &DataLayerM10Phase6SchedulerRuntimeState {
        &self.state
    }

    /// Runs one stateful Phase-6 scheduler cycle and updates runtime checkpoint/counters.
    pub fn run_cycle(
        &mut self,
        compliance_registry: &mut DataLayerM8ComplianceRegistry,
        partition_registry: &mut DataLayerM10PartitionLifecycleRegistry,
        execution_request: DataLayerM10Phase6ExecutionTickRequest,
    ) -> Result<DataLayerM10Phase6SchedulerCycleReport, DataLayerM10PartitionLifecycleError> {
        self.state.total_cycles = self.state.total_cycles.saturating_add(1);
        self.validate_runtime_clock(execution_request.now_epoch_seconds)?;
        let cycle_request = self.build_cycle_request(execution_request);
        self.run_cycle_request(compliance_registry, partition_registry, cycle_request)
    }

    fn validate_runtime_clock(
        &mut self,
        now_epoch_seconds: u64,
    ) -> Result<(), DataLayerM10PartitionLifecycleError> {
        validate_phase6_scheduler_runtime_clock(
            now_epoch_seconds,
            self.state.last_observed_now_epoch_seconds,
        )
        .inspect_err(|error| self.record_cycle_failure(error))
        .map(|()| {
            self.state.last_observed_now_epoch_seconds = Some(now_epoch_seconds);
        })
    }

    fn build_cycle_request(
        &self,
        execution_request: DataLayerM10Phase6ExecutionTickRequest,
    ) -> DataLayerM10Phase6SchedulerCycleRequest {
        DataLayerM10Phase6SchedulerCycleRequest {
            scheduler_policy: self.scheduler_policy,
            last_tick_epoch_seconds: self.state.last_successful_tick_epoch_seconds,
            budget: self.budget,
            execution_request,
        }
    }

    fn run_cycle_request(
        &mut self,
        compliance_registry: &mut DataLayerM8ComplianceRegistry,
        partition_registry: &mut DataLayerM10PartitionLifecycleRegistry,
        cycle_request: DataLayerM10Phase6SchedulerCycleRequest,
    ) -> Result<DataLayerM10Phase6SchedulerCycleReport, DataLayerM10PartitionLifecycleError> {
        data_layer_m10_execute_phase6_scheduler_cycle(
            compliance_registry,
            partition_registry,
            cycle_request,
        )
        .inspect(|report| self.apply_cycle_report(report))
        .inspect_err(|error| self.record_cycle_failure(error))
    }

    fn apply_cycle_report(&mut self, report: &DataLayerM10Phase6SchedulerCycleReport) {
        if report.reason_code == DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_APPLIED_REASON_CODE {
            self.state.executed_cycles = self.state.executed_cycles.saturating_add(1);
            self.state.last_successful_tick_epoch_seconds =
                self.state.last_observed_now_epoch_seconds;
        } else {
            self.state.deferred_cycles = self.state.deferred_cycles.saturating_add(1);
        }
        self.state.last_reason_code = report.reason_code;
    }

    fn record_cycle_failure(&mut self, error: &DataLayerM10PartitionLifecycleError) {
        self.state.fail_closed_cycles = self.state.fail_closed_cycles.saturating_add(1);
        self.state.last_reason_code = phase6_scheduler_error_reason_code(error);
    }
}
