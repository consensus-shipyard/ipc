// Copyright 2026 Recall Contributors
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::execution::{
    CompleteJobParams, CreateJobParams, ExecutionJob, FailJobParams, JobStatus, ListJobsParams,
    ListJobsReturn,
};
use fil_actors_runtime::ActorError;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::{address::Address, clock::ChainEpoch};

use crate::State;

#[derive(Debug, Default, Serialize_tuple, Deserialize_tuple)]
pub struct ExecutionState {
    pub next_job_id: u64,
    pub jobs: Vec<ExecutionJob>,
}

impl State {
    pub fn create_job(
        &mut self,
        creator: Address,
        params: CreateJobParams,
        epoch: ChainEpoch,
    ) -> Result<ExecutionJob, ActorError> {
        if params.binary_ref.is_empty() {
            return Err(ActorError::illegal_argument("binary_ref cannot be empty".into()));
        }
        if params.timeout_secs == 0 {
            return Err(ActorError::illegal_argument("timeout_secs must be > 0".into()));
        }

        let id = self.execution.next_job_id;
        self.execution.next_job_id += 1;

        let job = ExecutionJob {
            id,
            creator,
            claimed_by: None,
            status: JobStatus::Pending,
            binary_ref: params.binary_ref,
            input_refs: params.input_refs,
            args: params.args,
            env: params.env,
            timeout_secs: params.timeout_secs,
            created_epoch: epoch,
            started_epoch: None,
            completed_epoch: None,
            output_refs: Vec::new(),
            output_commitment: None,
            exit_code: None,
            error: None,
        };

        self.execution.jobs.push(job.clone());
        Ok(job)
    }

    pub fn claim_job(&mut self, worker: Address, id: u64, epoch: ChainEpoch) -> Result<ExecutionJob, ActorError> {
        let job = self
            .execution
            .jobs
            .iter_mut()
            .find(|j| j.id == id)
            .ok_or_else(|| ActorError::not_found(format!("job {} not found", id)))?;

        if job.status != JobStatus::Pending {
            return Err(ActorError::illegal_state(format!(
                "job {} is not pending (status: {:?})",
                id, job.status
            )));
        }

        job.status = JobStatus::Claimed;
        job.claimed_by = Some(worker);
        job.started_epoch = Some(epoch);
        Ok(job.clone())
    }

    pub fn complete_job(
        &mut self,
        worker: Address,
        params: CompleteJobParams,
        epoch: ChainEpoch,
    ) -> Result<ExecutionJob, ActorError> {
        let job = self
            .execution
            .jobs
            .iter_mut()
            .find(|j| j.id == params.id)
            .ok_or_else(|| ActorError::not_found(format!("job {} not found", params.id)))?;

        if job.claimed_by != Some(worker) {
            return Err(ActorError::forbidden("only claiming worker can complete job".into()));
        }
        if !(job.status == JobStatus::Claimed || job.status == JobStatus::Running) {
            return Err(ActorError::illegal_state(format!(
                "job {} is not claim/running (status: {:?})",
                params.id, job.status
            )));
        }

        job.status = JobStatus::Succeeded;
        job.completed_epoch = Some(epoch);
        job.output_refs = params.output_refs;
        job.output_commitment = Some(params.output_commitment);
        job.exit_code = Some(params.exit_code);
        job.error = None;
        Ok(job.clone())
    }

    pub fn fail_job(
        &mut self,
        worker: Address,
        params: FailJobParams,
        epoch: ChainEpoch,
    ) -> Result<ExecutionJob, ActorError> {
        let job = self
            .execution
            .jobs
            .iter_mut()
            .find(|j| j.id == params.id)
            .ok_or_else(|| ActorError::not_found(format!("job {} not found", params.id)))?;

        if job.claimed_by != Some(worker) {
            return Err(ActorError::forbidden("only claiming worker can fail job".into()));
        }
        if !(job.status == JobStatus::Claimed || job.status == JobStatus::Running) {
            return Err(ActorError::illegal_state(format!(
                "job {} is not claim/running (status: {:?})",
                params.id, job.status
            )));
        }

        job.status = JobStatus::Failed;
        job.completed_epoch = Some(epoch);
        job.exit_code = Some(params.exit_code);
        job.error = Some(params.reason);
        Ok(job.clone())
    }

    pub fn get_job(&self, id: u64) -> Option<ExecutionJob> {
        self.execution.jobs.iter().find(|j| j.id == id).cloned()
    }

    pub fn list_jobs(&self, params: ListJobsParams) -> ListJobsReturn {
        let limit = if params.limit == 0 { 100 } else { params.limit as usize };
        let mut jobs: Vec<ExecutionJob> = self
            .execution
            .jobs
            .iter()
            .filter(|job| {
                params
                    .status
                    .as_ref()
                    .is_none_or(|status| &job.status == status)
            })
            .take(limit)
            .cloned()
            .collect();
        jobs.shrink_to_fit();
        ListJobsReturn { jobs }
    }
}
