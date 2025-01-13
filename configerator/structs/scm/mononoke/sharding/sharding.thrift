// @generated SignedSource<<c42bd69c5f94046a9181744435b4f91e>>
// DO NOT EDIT THIS FILE MANUALLY!
// This file is a mechanical copy of the version in the configerator repo. To
// modify it, edit the copy in the configerator repo instead and copy it over by
// running the following in your fbcode directory:
//
// configerator-thrift-updater scm/mononoke/sharding/sharding.thrift
/*
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This software may be used and distributed according to the terms of the
 * GNU General Public License version 2.
 */

namespace py configerator.mononoke.sharding
namespace py3 mononoke.sharding
namespace cpp2 mononoke.sharding

// NOTICE:
// Don't use 'defaults' for any of these values (e.g. 'bool enabled = true')
// because these structs will be deserialized by serde in rust. The following
// rules apply upon deserialization:
//   1) specified default values are ignored, default values will always be
//      the 'Default::default()' value for a given type. For example, even
//      if you specify:
//          1: bool enabled = true,
//
//       upon decoding, if the field enabled isn't present, the default value
//       will be false.
//
//   2) not specifying optional won't actually make your field required,
//      neither will specifying required make any field required. Upon decoding
//      with serde, all values will be Default::default() and no error will be
//      given.
//
//   3) the only way to detect wether a field was specified in the structure
//      being deserialized is by making a field optional. This will result in
//      a 'None' value for a Option<T> in rust. So the way we can give default
//      values other then 'Default::default()' is by making a field optional,
//      and then explicitly handle 'None' after deserialization.

/// Struct representing the sharding configuration for a repo
/// independent of the type, job or region.
struct RawRepoShard {
  // The number of replicas this repo-shard should have. e.g. If we need 10 instances of
  // fbsource running for a job or request-serving process, then the replica count
  // should be 10.
  1: optional i64 replica_count;
  // The weight of each replica of the repo-shard expressed as a fractional value. e.g. If
  // a replica of aros repo-shard requires 15% of a hosts resources, then the repo-weight is
  // expressed as 0.15. Based on the actual host capacity (say weight = 100,000), this is then
  // converted to an integer value (100,000 * 0.15 = 15,000)
  2: optional float weight;
  // The list of shards that should NOT be co-located with the current repo-shard. e.g. If
  // fbsource should not be co-located with www & configerator, then the conflicting_repo_shards
  // should have the value ['www', 'configerator']
  // NOTE: This feature is not currently supported by SM and will be supported in the future.
  3: optional list<string> conflicting_repo_shards;
  // The list of repo shards that are associated with the given shard. This property can be used
  // for processes involving dual repo processing (e.g. fbsource to ovrsource backsyncer).
  4: optional list<string> associated_repo_shards;
  // The number of chunks to be created for the given repo shard. e.g. When chunks = 10 for repo
  // fbsource, the shards would be created as: fbsource_CHUNK_1-10, fbsource_CHUNK_2-10 and so on.
  // NOTE: If you want multiple tasks to run fbsource without any difference in state, then use
  // replica_count to get miltiple replicas. If you want multiple tasks running fbsource but each
  // to have a different state context (e.g. run X for the first 100K commit of fbsource), then
  // use chunks.
  5: optional i64 chunks;
  // If chunking is enabled, this option controls the size of each chunk to be provided to the job/service.
  // If the chunk size is dynamic and will be determined at runtime (e.g. AliasVerify), then this option
  // should not be set. e.g. When chunks = 10, repo = fbsource and chunk_size = 10000 then the shards would
  // be created as fbsource_CHUNK_1-10_SIZE_10000, fbsource_CHUNK_2-10_SIZE_10000
  6: optional i64 chunk_size;
  // If the repo needs to be chunked but the chunking strategy can be arbitrary then instead of
  // just providing the total number of chunks, we can provide chunks + chunk_list. e.g. When
  // chunks = 10 and chunk_list = [2, 3, 6] then the shards for repo fbsource would be created
  // as fbsource_CHUNK_2-10, fbsource_CHUNK_3-10 and fbsource_CHUNK_6-10
  7: optional list<i64> chunk_list;
} (rust.exhaustive)

/// The regions in which repo-shards can be deployed.
enum RawRegion {
  UNKNOWN = 0,
  GLOBAL = 1,
  ATN = 2,
  PRN = 3,
  LLA = 4,
  FTW = 5,
  FRC = 6,
  CLN = 7,
  ASH = 9,
  VLL = 10,
  PNB = 11,
  ODN = 12,
  NAO = 13,
  RVA = 14,
  LDC = 15,
  EAG = 16,
  NCG = 17,
  NHA = 18,
  SNC = 19,
  CCO = 20,
  GTN = 21,
} (rust.exhaustive)

/// The names of the processes for which a repo can be executed in a sharded setting.
enum RawProcessName {
  UNKNOWN = 0,
  WALKER_SCRUB_ALL = 1,
  WALKER_SCRUB_DERIVED = 2,
  WALKER_SCRUB_HG_ALL = 3,
  WALKER_SCRUB_UNODE_ALL = 4,
  WALKER_SHALLOW_HG_SCRUB = 5,
  WALKER_VALIDATE_ALL = 6,
  HG_SYNC_BACKUP = 7,
  HG_SYNC = 8,
  DERIVED_DATA_TAILER = 9,
  DERIVED_DATA_TAILER_BACKUP = 10,
  X_REPO_BACKSYNCER = 11, // deprecated in favour of X_REPO_BACKSYNC
  X_REPO_BOOKMARKS_VALIDATOR = 12,
  EDEN_API = 13,
  SCS = 14,
  LFS = 15,
  SHARDMANAGER_TEST = 16, // deprecated
  DERIVATION_WORKER = 17,
  ALIAS_VERIFY = 18, // deprecated
  ALIAS_VERIFY_BACKUP = 19, // deprecated
  DRAFT_COMMIT_DELETION = 20,
  STATISTICS_COLLECTOR = 21,
  X_REPO_BACKSYNC = 22,
  MONONOKE_GIT_SERVER = 23,
  REPO_METADATA_LOGGER = 24,
  CAS_SYNC = 25,
  X_REPO_SYNC = 26,
  ASYNC_REQUESTS_WORKER = 27,
  DERIVED_DATA_SERVICE = 28,
  MODERN_SYNC = 29,
} (rust.exhaustive)

/// Struct representing the sharding configuration for a repo
/// in the context of a specific job/process with defined region and type.
struct RawRepoProcessShard {
  // Flag determining if the current RepoProcessShard is for background process
  // like hg sync or for request serving process like SCS service.
  1: optional bool is_background;
  // Mapping of repo-shard configuration for each distinct region in which the
  // shards can be deployed.
  2: optional map<RawRegion, RawRepoShard> region_repo_shards;
} (rust.exhaustive)

/// Struct representing the overall sharding configuration for a repo.
struct RawRepoShardingConfig {
  // Mapping of process names to the corresponding process shard configuration
  // for the current repo.
  1: optional map<RawProcessName, RawRepoProcessShard> process_shards;
} (rust.exhaustive)

/// Struct representing the mapping of repo-names to their corresponding
/// RawRepoShardingConfig.
struct RawShardingConfigs {
  // Mapping of the repo-names to their corresponding RawRepoShardingConfig.
  1: optional map<string, RawRepoShardingConfig> repo_sharding_configs;
  // Mapping of the process names to the overall capacity for that process.
  2: optional map<RawProcessName, i64> process_capacity;
} (rust.exhaustive)
