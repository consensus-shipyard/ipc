// Copyright 2026 Recall Contributors
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::execution::{
    ClaimJobParams, CompleteJobParams, CreateJobParams, ExecutionJob, FailJobParams, GetJobParams,
    ListJobsParams, ListJobsReturn,
};
use fil_actors_runtime::{runtime::Runtime, ActorError};

use crate::{actor::BlobsActor, State};

impl BlobsActor {
    pub fn create_job(
        rt: &impl Runtime,
        params: CreateJobParams,
    ) -> Result<ExecutionJob, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let creator = rt.message().caller();
        rt.transaction(|st: &mut State, rt| st.create_job(creator, params.clone(), rt.curr_epoch()))
    }

    pub fn claim_job(
        rt: &impl Runtime,
        params: ClaimJobParams,
    ) -> Result<ExecutionJob, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let worker = rt.message().caller();

        // For MVP, execution permission is tied to active storage operators.
        let is_active_operator = {
            let state = rt.state::<State>()?;
            state.operators.get_index(&worker).is_some()
        };
        if !is_active_operator {
            return Err(ActorError::forbidden(
                "caller is not an active storage operator".into(),
            ));
        }

        rt.transaction(|st: &mut State, rt| st.claim_job(worker, params.id, rt.curr_epoch()))
    }

    pub fn complete_job(
        rt: &impl Runtime,
        params: CompleteJobParams,
    ) -> Result<ExecutionJob, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let worker = rt.message().caller();
        rt.transaction(|st: &mut State, rt| st.complete_job(worker, params.clone(), rt.curr_epoch()))
    }

    pub fn fail_job(rt: &impl Runtime, params: FailJobParams) -> Result<ExecutionJob, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let worker = rt.message().caller();
        rt.transaction(|st: &mut State, rt| st.fail_job(worker, params.clone(), rt.curr_epoch()))
    }

    pub fn get_job(
        rt: &impl Runtime,
        params: GetJobParams,
    ) -> Result<Option<ExecutionJob>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let state = rt.state::<State>()?;
        Ok(state.get_job(params.id))
    }

    pub fn list_jobs(rt: &impl Runtime, params: ListJobsParams) -> Result<ListJobsReturn, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let state = rt.state::<State>()?;
        Ok(state.list_jobs(params))
    }
}
