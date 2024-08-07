/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This software may be used and distributed according to the terms of the
 * GNU General Public License version 2.
 */

#![deny(warnings)]

use anyhow::anyhow;
use anyhow::Error;
use anyhow::Result;
use cached_config::ConfigStore;
use clap::Parser;
use context::CoreContext;
use fbinit::FacebookInit;
use futures::prelude::*;
use git_push_redirect::GitPushRedirectConfig;
use git_push_redirect::GitPushRedirectConfigEntry;
use git_push_redirect::SqlGitPushRedirectConfigBuilder;
use metaconfig_parser::RepoConfigs;
use mysql_client::ConnectionOptionsBuilder;
use mysql_client::ConnectionPoolOptionsBuilder;
use repository::Repository;
use slog::Logger;
use sql_construct::SqlConstruct;
use storage::Destination;
use storage::Xdb;
use storage::XdbFactory;
use tokio::time::Duration;

const MONONOKE_PRODUCTION_SHARD_NAME: &str = "xdb.mononoke_production";
const METAGIT_SHARD_NAME: &str = "xdb.metagit";

#[derive(Debug, Parser)]
pub struct Args {
    /// Seconds between checking for new updates to Mononoke Git repositories.
    #[arg(long = "mononoke-polling-interval", default_value = "5")]
    mononoke_polling_interval: u64,
    /// Path to the Mononoke configs.
    #[arg(
        long = "mononoke-config-path",
        default_value = "configerator://scm/mononoke/repos/tiers/scs"
    )]
    mononoke_config_path: String,
    /// Maximum concurrency of operations during one iteration of polling.
    #[arg(long = "concurrency", default_value = "10")]
    concurrency: usize,
}

pub fn create_config_store(fb: FacebookInit, logger: Logger) -> Result<ConfigStore> {
    const CRYPTO_PROJECT: &str = "SCM";
    const CONFIGERATOR_POLL_INTERVAL: Duration = Duration::from_secs(1);
    const CONFIGERATOR_REFRESH_TIMEOUT: Duration = Duration::from_secs(1);

    let crypto_regex_paths = vec!["scm/mononoke/repos/.*".to_string()];
    let crypto_regex = crypto_regex_paths
        .into_iter()
        .map(|path| (path, CRYPTO_PROJECT.to_string()))
        .collect();
    ConfigStore::regex_signed_configerator(
        fb,
        logger,
        crypto_regex,
        CONFIGERATOR_POLL_INTERVAL,
        CONFIGERATOR_REFRESH_TIMEOUT,
    )
}

fn create_prod_xdb_factory(fb: FacebookInit) -> Result<XdbFactory> {
    let pool_options = ConnectionPoolOptionsBuilder::default()
        .build()
        .map_err(Error::msg)?;
    let conn_options = ConnectionOptionsBuilder::default()
        .build()
        .map_err(Error::msg)?;
    let destination = Destination::Prod;
    XdbFactory::new(fb, destination, pool_options, conn_options)
}

async fn current_mononoke_git_repositories<'a>(
    ctx: &'a CoreContext,
    mononoke_production_xdb: &'a Xdb,
    metagit_xdb: &'a Xdb,
    repo_configs: &'a RepoConfigs,
) -> Result<Vec<Repository<'a>>> {
    let connections = mononoke_production_xdb.read_conns().await?;
    let git_push_redirect_config: &dyn GitPushRedirectConfig =
        &SqlGitPushRedirectConfigBuilder::from_sql_connections(connections).build();

    let entries: Vec<GitPushRedirectConfigEntry> = git_push_redirect_config
        .get_redirected_to_mononoke(ctx)
        .await?;
    let mut repositories: Vec<Repository> = vec![];

    for entry in entries {
        let id = entry.repo_id;
        repositories.push(Repository::new(
            id,
            repo_configs
                .get_repo_config(id)
                .map(|(name, _)| name.to_string())
                .ok_or_else(|| anyhow!("Could not find repository name for repository id {}", id))?
                .into(),
            metagit_xdb,
        ))
    }

    Ok(repositories)
}

async fn update_fingerprints(
    ctx: &CoreContext,
    mononoke_production_xdb: &Xdb,
    metagit_xdb: &Xdb,
    repo_configs: &RepoConfigs,
    concurrency: usize,
) -> Result<()> {
    let repositories =
        current_mononoke_git_repositories(ctx, mononoke_production_xdb, metagit_xdb, repo_configs)
            .await?;
    futures::stream::iter(repositories.into_iter().map(|repository| async move {
        repository.update_metagit_fingerprint().await?;
        Ok(repository)
    }))
    .buffer_unordered(concurrency)
    .for_each(|repository: Result<Repository>| async move {
        match repository {
            Ok(repository) => {
                logging::info!("Successfully processed repository `{}`", repository.name())
            }
            Err(e) => logging::warn!("Failed to process a repository with error `{}`", e),
        }
    })
    .await;

    Ok(())
}

pub async fn poll(fb: FacebookInit, args: Args) -> Result<()> {
    let logger = logging::get();
    let ctx = CoreContext::new_with_logger(fb, logger.clone());
    let config_path = args.mononoke_config_path;
    let config_store = create_config_store(fb, logger.clone())?;
    let repo_configs = metaconfig_parser::load_repo_configs(&config_path, &config_store)?;
    let xdb_factory = create_prod_xdb_factory(fb)?;
    let mononoke_production_xdb = xdb_factory
        .create_or_get_shard(MONONOKE_PRODUCTION_SHARD_NAME)
        .await?;
    let metagit_xdb = xdb_factory.create_or_get_shard(METAGIT_SHARD_NAME).await?;

    let mut interval = tokio::time::interval(Duration::from_secs(args.mononoke_polling_interval));
    loop {
        interval.tick().await;
        if let Err(e) = update_fingerprints(
            &ctx,
            &mononoke_production_xdb,
            &metagit_xdb,
            &repo_configs,
            args.concurrency,
        )
        .await
        {
            logging::warn!(
                "Encounted error `{}` while updating fingerprints in iteration",
                e
            );
        }
    }
}
