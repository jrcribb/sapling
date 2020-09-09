/*
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This software may be used and distributed according to the terms of the
 * GNU General Public License version 2.
 */

use std::collections::BTreeMap;
use std::sync::Arc;

use anyhow::Result;
use bookmarks::{BookmarkName, BookmarkUpdateLog, BookmarkUpdateReason, Freshness};
use context::CoreContext;
use fbinit::FacebookInit;
use futures::stream::TryStreamExt;
use mononoke_types::ChangesetId;
use tests_utils::drawdag::create_from_dag;

use crate::repo::{BookmarkFreshness, Repo, RepoContext};

async fn init_repo(ctx: &CoreContext) -> Result<(RepoContext, BTreeMap<String, ChangesetId>)> {
    let blob_repo = blobrepo_factory::new_memblob_empty(None)?;
    let changesets = create_from_dag(
        ctx,
        &blob_repo,
        r##"
            A-B-C-D-E
               \
                F-G
        "##,
    )
    .await?;
    let mut txn = blob_repo.update_bookmark_transaction(ctx.clone());
    txn.force_set(
        &BookmarkName::new("trunk")?,
        changesets["C"],
        BookmarkUpdateReason::TestMove,
        None,
    )?;
    txn.commit().await?;

    let repo = Repo::new_test(ctx.clone(), blob_repo).await?;
    let repo_ctx = RepoContext::new(ctx.clone(), Arc::new(repo)).await?;
    Ok((repo_ctx, changesets))
}

#[fbinit::compat_test]
async fn create_bookmark(fb: FacebookInit) -> Result<()> {
    let ctx = CoreContext::test_mock(fb);
    let (repo, changesets) = init_repo(&ctx).await?;
    let repo = repo.write().await?;

    // Can create public bookmarks on existing changesets (ancestors of trunk).
    repo.create_bookmark("bookmark1", changesets["A"]).await?;
    let bookmark1 = repo
        .resolve_bookmark("bookmark1", BookmarkFreshness::MostRecent)
        .await?
        .expect("bookmark should be set");
    assert_eq!(bookmark1.id(), changesets["A"]);

    // Can create public bookmarks on other changesets (not ancestors of trunk).
    repo.create_bookmark("bookmark2", changesets["F"]).await?;
    let bookmark2 = repo
        .resolve_bookmark("bookmark2", BookmarkFreshness::MostRecent)
        .await?
        .expect("bookmark should be set");
    assert_eq!(bookmark2.id(), changesets["F"]);

    // Can create scratch bookmarks.
    repo.create_bookmark("scratch/bookmark3", changesets["G"])
        .await?;
    let bookmark3 = repo
        .resolve_bookmark("scratch/bookmark3", BookmarkFreshness::MostRecent)
        .await?
        .expect("bookmark should be set");
    assert_eq!(bookmark3.id(), changesets["G"]);

    // F is now public.  G is not.
    let stack = repo.stack(vec![changesets["G"]], 10).await?;
    assert_eq!(stack.draft, vec![changesets["G"]]);
    assert_eq!(stack.public, vec![changesets["F"]]);

    Ok(())
}

#[fbinit::compat_test]
async fn move_bookmark(fb: FacebookInit) -> Result<()> {
    let ctx = CoreContext::test_mock(fb);
    let (repo, changesets) = init_repo(&ctx).await?;
    let repo = repo.write().await?;

    repo.move_bookmark("trunk", changesets["E"], None, false)
        .await?;
    let trunk = repo
        .resolve_bookmark("trunk", BookmarkFreshness::MostRecent)
        .await?
        .expect("bookmark should be set");
    assert_eq!(trunk.id(), changesets["E"]);

    // Attempt to move to a non-descendant commit without allowing
    // non-fast-forward moves should fail.
    assert!(
        repo.move_bookmark("trunk", changesets["G"], None, false)
            .await
            .is_err()
    );
    repo.move_bookmark("trunk", changesets["G"], None, true)
        .await?;
    let trunk = repo
        .resolve_bookmark("trunk", BookmarkFreshness::MostRecent)
        .await?
        .expect("bookmark should be set");
    assert_eq!(trunk.id(), changesets["G"]);

    // Check the bookmark moves created BookmarkLogUpdate entries
    let entries = repo
        .blob_repo()
        .attribute_expected::<dyn BookmarkUpdateLog>()
        .list_bookmark_log_entries(
            ctx.clone(),
            BookmarkName::new("trunk")?,
            3,
            None,
            Freshness::MostRecent,
        )
        .map_ok(|(cs, rs, _ts)| (cs, rs)) // dropping timestamps
        .try_collect::<Vec<_>>()
        .await?;
    assert_eq!(
        entries,
        vec![
            (Some(changesets["G"]), BookmarkUpdateReason::ApiRequest),
            (Some(changesets["E"]), BookmarkUpdateReason::ApiRequest),
            (Some(changesets["C"]), BookmarkUpdateReason::TestMove),
        ]
    );

    Ok(())
}

#[fbinit::compat_test]
async fn delete_bookmark(fb: FacebookInit) -> Result<()> {
    let ctx = CoreContext::test_mock(fb);
    let (repo, changesets) = init_repo(&ctx).await?;
    let repo = repo.write().await?;

    repo.create_bookmark("bookmark1", changesets["A"]).await?;
    repo.create_bookmark("bookmark2", changesets["F"]).await?;
    repo.create_bookmark("scratch/bookmark3", changesets["G"])
        .await?;

    // Can delete public bookmarks.
    repo.delete_bookmark("bookmark1", None).await?;
    assert!(
        repo.resolve_bookmark("bookmark1", BookmarkFreshness::MostRecent)
            .await?
            .is_none()
    );

    // Deleting a bookmark with the wrong old-target fails.
    assert!(
        repo.delete_bookmark("bookmark2", Some(changesets["E"]))
            .await
            .is_err()
    );
    let bookmark2 = repo
        .resolve_bookmark("bookmark2", BookmarkFreshness::MostRecent)
        .await?
        .expect("bookmark should be set");
    assert_eq!(bookmark2.id(), changesets["F"]);

    // But with the right old-target succeeds.
    repo.delete_bookmark("bookmark2", Some(changesets["F"]))
        .await?;
    assert!(
        repo.resolve_bookmark("bookmark1", BookmarkFreshness::MostRecent)
            .await?
            .is_none()
    );

    // Can't delete scratch bookmarks.
    assert!(
        repo.delete_bookmark("scratch/bookmark3", None)
            .await
            .is_err()
    );

    Ok(())
}
