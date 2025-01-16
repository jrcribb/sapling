# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This software may be used and distributed according to the terms of the
# GNU General Public License found in the LICENSE file in the root
# directory of this source tree.

  $ . "${TEST_FIXTURES}/library.sh"
  $ GIT_REPO_ORIGIN="${TESTTMP}/origin/repo-git"
  $ GIT_REPO="${TESTTMP}/repo-git"
  $ HG_REPO="${TESTTMP}/repo"
  $ setup_common_config blob_files

# Setup git repository
  $ mkdir -p "$GIT_REPO_ORIGIN"
  $ cd "$GIT_REPO_ORIGIN"
  $ git init -q
  $ echo "this is file1" > file1
  $ git add file1
  $ git commit -qam "Add file1"
  $ git tag -a -m "new tag" first_tag
# Capture the root tree hash
  $ root_tree_hash=$(git cat-file commit $(git rev-list --max-parents=0 HEAD) | grep '^tree' | awk '{print $2}')
# Capture the first blob hash
  $ first_blob_hash=$(git ls-tree $(git rev-list --max-parents=0 HEAD) | awk '{print $3}' | head -n 1)
# Create an annotated tag pointing to the root tree of the repo
  $ git tag -a tag_to_tree $root_tree_hash -m "Tag pointing to root tree"  
# Create a branch pointing to the root tree of the repo
  $ echo $root_tree_hash > .git/refs/heads/branch_to_root_tree
# Create a simple tag pointing to the root tree of the repo
  $ git tag simple_tag_to_tree $root_tree_hash
# Create a branch pointing to a blob in the repo
  $ echo $first_blob_hash > .git/refs/heads/branch_to_blob
# Create a recursive tag to check if it gets imported
  $ git config advice.nestedTag false
  $ git tag -a recursive_tag -m "this recursive tag points to tag_to_tree" $(git rev-parse tag_to_tree)
  $ cd "$TESTTMP"
  $ git clone --mirror "$GIT_REPO_ORIGIN" repo-git
  Cloning into bare repository 'repo-git'...
  done.


# Import it into Mononoke
  $ cd "$TESTTMP"
  $ with_stripped_logs gitimport "$GIT_REPO" --concurrency 100 --generate-bookmarks --allow-content-refs full-repo
  using repo "repo" repoid RepositoryId(0)
  GitRepo:$TESTTMP/repo-git commit 1 of 1 - Oid:8ce3eae4 => Bid:032cd4dc, repo: $TESTTMP/repo-git
  Ref: "refs/heads/branch_to_blob": None
  Ref: "refs/heads/branch_to_root_tree": None
  Ref: "refs/heads/master_bookmark": Some(ChangesetId(Blake2(032cd4dce0406f1c1dd1362b6c3c9f9bdfa82f2fc5615e237a890be4fe08b044)))
  Ref: "refs/tags/first_tag": Some(ChangesetId(Blake2(032cd4dce0406f1c1dd1362b6c3c9f9bdfa82f2fc5615e237a890be4fe08b044)))
  Ref: "refs/tags/recursive_tag": None
  Ref: "refs/tags/simple_tag_to_tree": None
  Ref: "refs/tags/tag_to_tree": None
  Initializing repo: repo
  Initialized repo: repo
  All repos initialized. It took: * seconds (glob)
  Bookmark: "heads/master_bookmark": ChangesetId(Blake2(032cd4dce0406f1c1dd1362b6c3c9f9bdfa82f2fc5615e237a890be4fe08b044)) (created)
  Bookmark: "tags/first_tag": ChangesetId(Blake2(032cd4dce0406f1c1dd1362b6c3c9f9bdfa82f2fc5615e237a890be4fe08b044)) (created)

# Ensure that the refs pointing to trees and blobs are recorded as expected. Note that currently we record the refs with the "refs/" prefix which is incorrect
  $ sqlite3 "$TESTTMP/monsql/sqlite_dbs" "SELECT ref_name, hex(git_hash) as git_hash, is_tree FROM git_ref_content_mapping ORDER BY ref_name"
  refs/heads/branch_to_blob|433EB172726BC7B6D60E8D68EFB0F0EF4E67A667|0
  refs/heads/branch_to_root_tree|CB2EF838EB24E4667FEE3A8B89C930234AE6E4BB|1
  refs/tags/simple_tag_to_tree|CB2EF838EB24E4667FEE3A8B89C930234AE6E4BB|1
  refs/tags/tag_to_tree|CB2EF838EB24E4667FEE3A8B89C930234AE6E4BB|1
