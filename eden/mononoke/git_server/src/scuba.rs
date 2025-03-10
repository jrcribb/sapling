/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This software may be used and distributed according to the terms of the
 * GNU General Public License version 2.
 */

use context::CoreContext;
use gotham::state::State;
use gotham_ext::middleware::request_context::RequestContext;
use gotham_ext::middleware::MetadataState;
use gotham_ext::middleware::PostResponseInfo;
use gotham_ext::middleware::ScubaHandler;
use permission_checker::MononokeIdentitySetExt;
use scuba_ext::MononokeScubaSampleBuilder;

use crate::model::GitMethodInfo;
use crate::model::PushValidationErrors;

#[derive(Copy, Clone, Debug)]
pub enum MononokeGitScubaKey {
    Repo,
    Method,
    MethodVariants,
    User,
    Error,
    ErrorCount,
    PushValidationErrors,
    PackfileReadError,
    PackfileSize,
}

impl AsRef<str> for MononokeGitScubaKey {
    fn as_ref(&self) -> &'static str {
        match self {
            Self::Repo => "repo",
            Self::Method => "method",
            Self::MethodVariants => "method_variants",
            Self::User => "user",
            Self::Error => "error",
            Self::ErrorCount => "error_count",
            Self::PushValidationErrors => "push_validation_errors",
            Self::PackfileReadError => "packfile_read_error",
            Self::PackfileSize => "packfile_size",
        }
    }
}

impl From<MononokeGitScubaKey> for String {
    fn from(key: MononokeGitScubaKey) -> Self {
        key.as_ref().to_string()
    }
}

#[derive(Clone)]
pub struct MononokeGitScubaHandler {
    request_context: Option<RequestContext>,
    method_info: Option<GitMethodInfo>,
    push_validation_errors: Option<PushValidationErrors>,
    client_username: Option<String>,
}

pub(crate) fn scuba_from_state(ctx: &CoreContext, state: &State) -> MononokeScubaSampleBuilder {
    let scuba = ctx.scuba().clone();
    let user = state
        .try_borrow::<MetadataState>()
        .and_then(|metadata_state| metadata_state.metadata().identities().username())
        .map(ToString::to_string);
    scuba_with_basic_info(user, state.try_borrow::<GitMethodInfo>().cloned(), scuba)
}

fn scuba_with_basic_info(
    user: Option<String>,
    info: Option<GitMethodInfo>,
    mut scuba: MononokeScubaSampleBuilder,
) -> MononokeScubaSampleBuilder {
    scuba.add_opt(MononokeGitScubaKey::User, user);
    if let Some(info) = info {
        scuba.add(MononokeGitScubaKey::Repo, info.repo.clone());
        scuba.add(MononokeGitScubaKey::Method, info.method.to_string());
        scuba.add(
            MononokeGitScubaKey::MethodVariants,
            info.variants_to_string(),
        );
        scuba.add(
            MononokeGitScubaKey::MethodVariants,
            info.variants_to_string_vector(),
        );
    }
    scuba
}

impl MononokeGitScubaHandler {
    pub fn from_state(state: &State) -> Self {
        Self {
            request_context: state.try_borrow::<RequestContext>().cloned(),
            method_info: state.try_borrow::<GitMethodInfo>().cloned(),
            push_validation_errors: state.try_borrow::<PushValidationErrors>().cloned(),
            client_username: state
                .try_borrow::<MetadataState>()
                .and_then(|metadata_state| metadata_state.metadata().identities().username())
                .map(ToString::to_string),
        }
    }

    pub(crate) fn to_scuba(&self, ctx: &CoreContext) -> MononokeScubaSampleBuilder {
        let scuba = ctx.scuba().clone();
        scuba_with_basic_info(
            self.client_username.clone(),
            self.method_info.clone(),
            scuba,
        )
    }

    fn log_processed(self, info: &PostResponseInfo, mut scuba: MononokeScubaSampleBuilder) {
        scuba = scuba_with_basic_info(self.client_username, self.method_info, scuba);
        if let Some(ctx) = self.request_context {
            ctx.ctx.perf_counters().insert_perf_counters(&mut scuba);
        }

        if let Some(push_validation_errors) = self.push_validation_errors {
            scuba.add(
                MononokeGitScubaKey::PushValidationErrors,
                push_validation_errors.to_string(),
            );
        }
        if let Some(err) = info.first_error() {
            scuba.add(MononokeGitScubaKey::Error, format!("{:?}", err));
        }
        scuba.add(MononokeGitScubaKey::ErrorCount, info.error_count());
        scuba.add("log_tag", "MononokeGit Request Processed");
        scuba.log();
    }

    fn log_cancelled(mut scuba: MononokeScubaSampleBuilder) {
        scuba.add("log_tag", "MononokeGit Request Cancelled");
        scuba.log();
    }
}

impl ScubaHandler for MononokeGitScubaHandler {
    fn from_state(state: &State) -> Self {
        Self::from_state(state)
    }

    fn log_processed(self, info: &PostResponseInfo, scuba: MononokeScubaSampleBuilder) {
        Self::log_processed(self, info, scuba)
    }

    fn log_cancelled(scuba: MononokeScubaSampleBuilder) {
        Self::log_cancelled(scuba)
    }
}
