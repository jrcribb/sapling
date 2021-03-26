/*
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This software may be used and distributed according to the terms of the
 * GNU General Public License version 2.
 */

use anyhow::{Context, Result};
use sql::queries;
use sql_ext::SqlConnections;

use stats::prelude::*;

use context::{CoreContext, PerfCounterType};
use mononoke_types::RepositoryId;

use crate::logging::log_new_segmented_changelog_version;
use crate::types::{IdDagVersion, IdMapVersion, SegmentedChangelogVersion};

define_stats! {
    prefix = "mononoke.segmented_changelog.sql_version_store";
    set: timeseries(Sum),
    get: timeseries(Sum),
}

/// Specifies the versions for the latest SegmentedChangelogVersion. The version contains IdDag and
/// IdMap versions.  The IdDag version can be loaded directly from the blobstore and the IdMap
/// version ties the IdDag back to the bonsai changesets.
pub struct SegmentedChangelogVersionStore {
    connections: SqlConnections,
    repo_id: RepositoryId,
}

impl SegmentedChangelogVersionStore {
    pub fn new(connections: SqlConnections, repo_id: RepositoryId) -> Self {
        Self {
            connections,
            repo_id,
        }
    }

    pub async fn set(&self, ctx: &CoreContext, version: SegmentedChangelogVersion) -> Result<()> {
        STATS::set.add_value(1);
        ctx.perf_counters()
            .increment_counter(PerfCounterType::SqlWrites);
        Insertversion::query(
            &self.connections.write_connection,
            &[(
                &self.repo_id,
                &version.iddag_version,
                &version.idmap_version,
            )],
        )
        .await
        .context("inserting segmented changelog version")?;
        log_new_segmented_changelog_version(ctx, self.repo_id, version);
        Ok(())
    }

    pub async fn get(&self, ctx: &CoreContext) -> Result<Option<SegmentedChangelogVersion>> {
        STATS::get.add_value(1);
        ctx.perf_counters()
            .increment_counter(PerfCounterType::SqlReadsReplica);
        let rows = Selectversion::query(&self.connections.read_connection, &self.repo_id).await?;
        Ok(rows.into_iter().next().map(|r| r.into()))
    }
}

queries! {
    write Insertversion(
        values: (repo_id: RepositoryId, iddag_version: IdDagVersion, idmap_version: IdMapVersion)
    ) {
        none,
        "
        REPLACE INTO segmented_changelog_version (repo_id, iddag_version, idmap_version)
        VALUES {values}
        "
    }

    read Selectversion(repo_id: RepositoryId) -> (IdDagVersion, IdMapVersion) {
        "
        SELECT iddag_version, idmap_version
        FROM segmented_changelog_version
        WHERE repo_id = {repo_id}
        "
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fbinit::FacebookInit;

    use sql_construct::SqlConstruct;

    use crate::builder::SegmentedChangelogSqlConnections;

    #[fbinit::test]
    async fn test_more_than_one_repo(fb: FacebookInit) -> Result<()> {
        let ctx = CoreContext::test_mock(fb);
        let conns = SegmentedChangelogSqlConnections::with_sqlite_in_memory()?;
        let version_repo1 =
            SegmentedChangelogVersionStore::new(conns.0.clone(), RepositoryId::new(1));
        let version_repo2 =
            SegmentedChangelogVersionStore::new(conns.0.clone(), RepositoryId::new(2));

        assert_eq!(version_repo1.get(&ctx).await?, None);
        assert_eq!(version_repo2.get(&ctx).await?, None);
        let version11 = SegmentedChangelogVersion::new(
            IdDagVersion::from_serialized_bytes(b"1"),
            IdMapVersion(1),
        );
        let version23 = SegmentedChangelogVersion::new(
            IdDagVersion::from_serialized_bytes(b"2"),
            IdMapVersion(3),
        );
        version_repo1.set(&ctx, version11).await?;
        assert_eq!(version_repo1.get(&ctx).await?, Some(version11));
        assert_eq!(version_repo2.get(&ctx).await?, None);
        version_repo2.set(&ctx, version23).await?;
        assert_eq!(version_repo1.get(&ctx).await?, Some(version11));
        assert_eq!(version_repo2.get(&ctx).await?, Some(version23));

        Ok(())
    }
}
