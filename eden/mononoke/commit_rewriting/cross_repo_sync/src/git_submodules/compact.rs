/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This software may be used and distributed according to the terms of the
 * GNU General Public License version 2.
 */

use std::clone::Clone;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use bytes::Bytes;
use cloned::cloned;
use commit_transformation::rewrite_commit;
use commit_transformation::RewriteOpts;
use context::CoreContext;
use filestore::StoreRequest;
use fsnodes::RootFsnodeId;
use futures::future;
use futures::stream;
use futures::stream::StreamExt;
use futures::stream::TryStreamExt;
use futures_stats::TimedFutureExt;
use itertools::Itertools;
use manifest::ManifestOps;
use mononoke_types::BonsaiChangesetMut;
use mononoke_types::ChangesetId;
use mononoke_types::ContentId;
use mononoke_types::FileChange;
use mononoke_types::FileType;
use mononoke_types::FsnodeId;
use mononoke_types::GitLfs;
use mononoke_types::NonRootMPath;
use movers::Mover;
use scuba_ext::FutureStatsScubaExt;

use crate::commit_syncers_lib::mover_to_multi_mover;
use crate::git_submodules::expand::SubmoduleExpansionData;
use crate::git_submodules::git_hash_from_submodule_metadata_file;
use crate::git_submodules::utils::get_x_repo_submodule_metadata_file_path;
use crate::git_submodules::validation::SubmoduleExpansionValidationToken;
use crate::git_submodules::validation::ValidSubmoduleExpansionBonsai;
use crate::git_submodules::SubmodulePath;
use crate::reporting::log_debug;
use crate::reporting::log_error;
use crate::reporting::log_trace;
use crate::types::Repo;

/// Wrapper type to ensure that the bonsai generated by compacting the submodule
/// expansion **can only be used to be rewritten to a small repo**.
///
/// This is needed because the large repo should never have file changes of type
/// GitSubmodule, but the process of compacting/backsyncing submodule expansion
/// changes involves creating these file changes.
///
/// To make it harder for these bonsais to be saved to the large repo, this type
/// only exposes one method: which is to rewrite itself for a small repo.
pub(crate) struct CompactedSubmoduleBonsai(BonsaiChangesetMut);

impl CompactedSubmoduleBonsai {
    pub(crate) async fn rewrite_to_small_repo<'a, R: Repo>(
        self,
        ctx: &'a CoreContext,
        remapped_parents: &'a HashMap<ChangesetId, ChangesetId>,
        mover: Mover,
        large_repo: &'a R,
        rewrite_opts: RewriteOpts,
    ) -> Result<Option<BonsaiChangesetMut>> {
        rewrite_commit(
            ctx,
            self.0,
            remapped_parents,
            mover_to_multi_mover(mover),
            large_repo,
            None,
            rewrite_opts,
        )
        .await
        .context("Failed to rewrite compacted submodule bonsai for small repo")
    }
}

/// Given a large repo bonsai that might contain submodule expansions, validate
/// that all submodule expansions are valid and return a new bonsai without
/// the submodule expansion file changes and with the proper file change
/// of type GitSubmodule that can be backsynced to the small repo.
pub(crate) async fn compact_all_submodule_expansion_file_changes<'a, R: Repo>(
    ctx: &'a CoreContext,
    // Large repo bonsai
    bonsai_mut: BonsaiChangesetMut,
    sm_exp_data: SubmoduleExpansionData<'a, R>,
    large_repo: &'a R,
    // Forward sync mover is needed to convert paths from submodule configs,
    // which are all relative small repo root, to their large repo counter-parts
    forward_sync_mover: Mover,
) -> Result<CompactedSubmoduleBonsai> {
    let bonsai = bonsai_mut.freeze()?;

    log_trace(
        ctx,
        format!(
            "Compacting all submodule expansions of bonsai: {0:#?}",
            bonsai.message()
        ),
    );

    let valid_bonsai = ValidSubmoduleExpansionBonsai::validate_all_submodule_expansions(
        ctx,
        sm_exp_data.clone(),
        bonsai,
        forward_sync_mover.clone(),
    )
    .timed()
    .await
    .log_future_stats(
        ctx.scuba().clone(),
        "Validating all submodule expansions",
        None,
    )
    .context("Validation of submodule expansion failed")?;

    compact_all_submodule_expansion_file_changes_impl(
        ctx,
        valid_bonsai,
        sm_exp_data,
        large_repo,
        forward_sync_mover,
    )
    .await
    .map(CompactedSubmoduleBonsai)
}

/// Given a bonsai that has passed submodule expansion validation, compact
/// changes made to all submodule expansions by replacing them with file changes
/// of type GitSubmodule.
async fn compact_all_submodule_expansion_file_changes_impl<'a, R: Repo>(
    ctx: &CoreContext,
    valid_bonsai: ValidSubmoduleExpansionBonsai,
    sm_exp_data: SubmoduleExpansionData<'a, R>,
    large_repo: &'a R,
    forward_sync_mover: Mover,
) -> Result<BonsaiChangesetMut> {
    let (bonsai, validation_token) = valid_bonsai.into_inner_with_token();
    let bonsai_mut = bonsai.into_mut();

    // Remove all recursive submodules from the submodule deps, because any
    // change to them will also change a top-level submodule expansion.
    // Since all expansion changes are removed, we only need to generate a
    // GitSubmodule file change to the top-level submodule.
    let top_level_submodule_deps: HashMap<NonRootMPath, Arc<R>> = sm_exp_data
        .submodule_deps
        .clone()
        .into_iter()
        // Submodule paths need to be sorted for the filtering below to work
        .sorted_by_key(|(sm_path, _repo)| sm_path.clone())
        .fold(HashMap::new(), |mut filtered_paths, (sm_path, sm_repo)| {
            for path in filtered_paths.keys() {
                if path.is_prefix_of(&sm_path) && *path != sm_path {
                    // Recursive submodule path, so it can be ignored
                    return filtered_paths;
                }
            }
            filtered_paths.insert(sm_path, sm_repo);
            filtered_paths
        });

    let compacted_bonsai_mut = stream::iter(top_level_submodule_deps)
        .map(anyhow::Ok)
        .try_fold(bonsai_mut, |bonsai_mut, (sm_path, _)| {
            cloned!(forward_sync_mover);
            let x_repo_submodule_metadata_file_prefix =
                sm_exp_data.x_repo_submodule_metadata_file_prefix;

            async move {
                compact_submodule_expansion_file_changes(
                    ctx,
                    bonsai_mut,
                    large_repo,
                    x_repo_submodule_metadata_file_prefix,
                    forward_sync_mover,
                    &sm_path,
                    validation_token,
                )
                .await
            }
        })
        .await?;

    Ok(compacted_bonsai_mut)
}

/// If there are any changes made to a submodule's expansion, compact them, which
/// means:
///
/// 1. Getting the submodule commit being expanded from the submodule expansion
/// metadata file.
/// 2. Removing all the file changes from the expansion and metadata file from
/// the bonsai.
/// 3. Adding a single file change of type GitSubmodule to the bonsai, pointing
/// to the submodule commit.
///
/// IMPORTANT: this function assumes that the provided bonsai **has a valid
/// submodule expansion**. So **never call this without validating the expansion
/// first!**
async fn compact_submodule_expansion_file_changes<'a, R: Repo>(
    ctx: &'a CoreContext,
    // Large repo bonsai
    mut bonsai_mut: BonsaiChangesetMut,
    large_repo: &'a R,
    x_repo_submodule_metadata_file_prefix: &'a str,
    forward_sync_mover: Mover,
    sm_path: &'a NonRootMPath,
    // Token that ensures that this function can't be called without performing
    // submodule expansion validation on the provided bonsai.
    _validation_token: SubmoduleExpansionValidationToken,
) -> Result<BonsaiChangesetMut> {
    let x_repo_sm_metadata_file_path = get_x_repo_submodule_metadata_file_path(
        &SubmodulePath(sm_path.clone()),
        x_repo_submodule_metadata_file_prefix,
    )?;

    log_trace(
        ctx,
        format!(
            "Compacting submodule {sm_path}. Metadata file path: {x_repo_sm_metadata_file_path}"
        ),
    );

    let synced_sm_metadata_file_path = forward_sync_mover(&x_repo_sm_metadata_file_path)
        .with_context(|| anyhow!("Mover failed on path {x_repo_sm_metadata_file_path}"))?
        .ok_or_else(|| {
            anyhow!("Mover didn't return any path for path {x_repo_sm_metadata_file_path}")
        })?;

    let large_repo_sm_path = forward_sync_mover(sm_path)
        .with_context(|| anyhow!("Forward sync mover failed on submodule path: {sm_path}"))?
        .ok_or(anyhow!(
            "Forward sync mover didn't provide large repo path for submodule path: {sm_path}"
        ))?;

    log_trace(
        ctx,
        format!("synced_sm_metadata_file_path is {synced_sm_metadata_file_path}"),
    );

    // Consindering that the provided bonsai is valid, any change affecting the
    // expansion will be affecting the expansion's metadata file.
    match bonsai_mut
        .file_changes
        .remove(&synced_sm_metadata_file_path)
    {
        Some(sm_metadata_file_fc) => {
            log_trace(
                ctx,
                format!("Submodule metadata file {synced_sm_metadata_file_path} was modified"),
            );
            match sm_metadata_file_fc {
                FileChange::Change(tfc) => {
                    compact_submodule_expansion_update(
                        ctx,
                        bonsai_mut,
                        large_repo,
                        large_repo_sm_path,
                        tfc.content_id(),
                    )
                    .await
                }
                FileChange::Deletion => {
                    compact_submodule_expansion_deletion(
                        ctx,
                        bonsai_mut,
                        large_repo,
                        large_repo_sm_path,
                    )
                    .await
                }
                _ => bail!("Unsupported change to submodule metadata file"),
            }
        }
        None => {
            log_trace(
                ctx,
                format!("Submodule metadata file {synced_sm_metadata_file_path} was NOT modified"),
            );
            Ok(bonsai_mut)
        }
    }
}

/// Handle updates to the submodulen pointer, i.e. where the metadata file was
/// updated with a git commit and the expansion working copy was updated to
/// match the working copy of that commit.
async fn compact_submodule_expansion_update<'a, R: Repo>(
    ctx: &'a CoreContext,
    mut bonsai_mut: BonsaiChangesetMut,
    large_repo: &'a R,
    large_repo_sm_path: NonRootMPath,
    sm_metadata_file_content_id: ContentId,
) -> Result<BonsaiChangesetMut> {
    // If the submodule metadata file was changed, remove all changes from the
    // expansion.
    bonsai_mut
        .file_changes
        .retain(|path, _fc| !large_repo_sm_path.is_prefix_of(path));

    let git_submodule_sha1 =
        git_hash_from_submodule_metadata_file(ctx, large_repo, sm_metadata_file_content_id).await?;
    let oid = git_submodule_sha1
        .to_object_id()
        .context("Object id from GitSha1")?;

    let oid_bytes = Bytes::copy_from_slice(oid.as_slice());

    let submodule_commit_content_id = filestore::store(
        large_repo.repo_blobstore(),
        *large_repo.filestore_config(),
        ctx,
        &StoreRequest::new(oid_bytes.len() as u64),
        stream::once(async move { Ok(oid_bytes) }),
    )
    .await
    .context("filestore: upload GitSubmodule file change")?;

    let sm_file_change = FileChange::tracked(
        submodule_commit_content_id.content_id,
        FileType::GitSubmodule,
        submodule_commit_content_id.total_size,
        None,
        GitLfs::FullContent,
    );

    bonsai_mut
        .file_changes
        .insert(large_repo_sm_path, sm_file_change);

    Ok(bonsai_mut)
}

/// Handle deletion of the submodule expansion.
///
///
/// Even though deleting only the submodule metadata file would be a valid,
/// "back-syncable" change, this change would add the entire expansion working
/// copy to the small repo.
/// Because users would likely shoot themselves in the foot when doing this,
/// we'll only allow the deletion of the metadata file if the **entire
/// submodule expansion is also deleted**.
///
/// NOTE: when this function is called, the caller has already removed the
/// submodule metadata file change from the bonsai.
async fn compact_submodule_expansion_deletion<'a, R: Repo>(
    ctx: &'a CoreContext,
    bonsai_mut: BonsaiChangesetMut,
    large_repo: &'a R,
    large_repo_sm_path: NonRootMPath,
) -> Result<BonsaiChangesetMut> {
    let parents = bonsai_mut.parents.clone();
    let parent_cs_id = match parents[..] {
        [cs_id] => cs_id,
        [] => bail!("Can't compact expansion in bonsai without parents"),
        _ => bail!("Can't compact expansion in bonsai with multiple parents"),
    };

    let parent_fsnode_id: FsnodeId = large_repo
        .repo_derived_data()
        .derive::<RootFsnodeId>(ctx, parent_cs_id)
        .await
        .context("Failed to derive parent root fsnode id")?
        .into_fsnode_id();

    let expansion_files_stream = parent_fsnode_id.list_leaf_entries_under(
        ctx.clone(),
        large_repo.repo_blobstore_arc(),
        [large_repo_sm_path.clone()],
    );

    // Iterate over all the files in the submodule expansion working copy to
    // ensure that they're all being deleted in this changeset.
    //
    // Keep track of any file in the expansion that is not being deleted, to
    // throw an error and log.
    let (mut bonsai_mut, missing_paths) = expansion_files_stream
        .try_fold(
            (bonsai_mut, HashSet::new()),
            |(mut bonsai_mut, mut missing_paths), (file_path, _)| {
                let fc = bonsai_mut.file_changes.remove(&file_path);
                match fc {
                    // File in expansion is being deleted, as expected
                    Some(FileChange::Deletion) => (),
                    Some(fc) => {
                        log_trace(
                            ctx,
                            format!("File {file_path} is being modified when it should be deleted. Change: {fc:#?}"),
                        );
                        missing_paths.insert(file_path);
                    }
                    None => {
                        log_trace(ctx, format!("File {file_path} was not deleted"));
                        missing_paths.insert(file_path);
                    }
                };
                future::ok((bonsai_mut, missing_paths))
            },
        )
        .await?;

    if !missing_paths.is_empty() {
        let msg = format!(
            "Submodule metadata file was deleted but {} files in the submodule expansion were not.",
            missing_paths.len()
        );
        log_error(ctx, msg.clone());

        let examples = missing_paths.into_iter().take(10).collect::<Vec<_>>();
        log_debug(
            ctx,
            format!("Example paths that should be deleted but weren't: {examples:#?}"),
        );

        return Err(anyhow!(msg));
    }

    // All paths in submodule expansion were removed. The submodule metadata
    // file change was removed by the caller, so now we just need to insert a
    // deletion for the submodule.
    bonsai_mut
        .file_changes
        .insert(large_repo_sm_path, FileChange::Deletion);

    Ok(bonsai_mut)
}
