
  $ . "$TESTDIR/library.sh"

  $ newserver master
  $ clone master client1
  $ cd client1

  $ drawdag <<'EOS'
  > A1 # A1/A = A42
  > |  # A1/A1 = (removed)
  > |
  > B
  > |
  > A
  > |
  > C
  > EOS

  $ hg rebase -s $B -d $C
  rebasing 2:c84328973e26 "B"
  rebasing 3:2f1af6263db7 "A1" (tip)
  other [source] changed A which local [dest] deleted
  use (c)hanged version, leave (d)eleted, or leave (u)nresolved? u
  unresolved conflicts (see hg resolve, then hg rebase --continue)
  [1]

  $ hg log -Gr 'all()'
  @  changeset:   4:27652fba03b2
  |  tag:         tip
  |  parent:      0:96cc3511f894
  |  user:        test
  |  date:        Thu Jan 01 00:00:00 1970 +0000
  |  summary:     B
  |
  | @  changeset:   3:2f1af6263db7
  | |  user:        test
  | |  date:        Thu Jan 01 00:00:00 1970 +0000
  | |  summary:     A1
  | |
  | o  changeset:   2:c84328973e26
  | |  user:        test
  | |  date:        Thu Jan 01 00:00:00 1970 +0000
  | |  summary:     B
  | |
  | o  changeset:   1:9cfaa5b6d3e1
  |/   user:        test
  |    date:        Thu Jan 01 00:00:00 1970 +0000
  |    summary:     A
  |
  o  changeset:   0:96cc3511f894
     user:        test
     date:        Thu Jan 01 00:00:00 1970 +0000
     summary:     C
  

  $ hg rm -f A

  $ hg resolve -m A
  (no more unresolved files)
  continue: hg rebase --continue

  $ hg rebase --continue
  already rebased 2:c84328973e26 "B" as 27652fba03b2
  rebasing 3:2f1af6263db7 "A1"

  $ hg log -Gr 'all()'
  o  changeset:   5:8bbb642d1454
  |  tag:         tip
  |  user:        test
  |  date:        Thu Jan 01 00:00:00 1970 +0000
  |  summary:     A1
  |
  o  changeset:   4:27652fba03b2
  |  parent:      0:96cc3511f894
  |  user:        test
  |  date:        Thu Jan 01 00:00:00 1970 +0000
  |  summary:     B
  |
  | o  changeset:   1:9cfaa5b6d3e1
  |/   user:        test
  |    date:        Thu Jan 01 00:00:00 1970 +0000
  |    summary:     A
  |
  o  changeset:   0:96cc3511f894
     user:        test
     date:        Thu Jan 01 00:00:00 1970 +0000
     summary:     C
  
