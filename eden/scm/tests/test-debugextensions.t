#modern-config-incompatible

#require no-fsmonitor no-eden


  $ disable copytrace
  $ hg debugextensions --excludedefault
  remotenames (untested!)
  treemanifest (untested!)

  $ enable histedit rebase
  $ newext ext1 <<EOF
  > EOF
  $ newext ext2 <<EOF
  > testedwith = '3.0 3.1 3.2.1'
  > buglink = 'https://example.org/bts'
  > EOF

  $ setconfig extensions.hotfix1=python-base64:Cgo=

  $ hg debugextensions --excludedefault
  ext1 (untested!)
  ext2 (3.2.1!)
  histedit
  hotfix1 (untested!)
  rebase
  remotenames (untested!)
  treemanifest (untested!)

  $ hg debugextensions -v --excludedefault
  ext1
    location: *ext1.py* (glob)
    bundled: no
  ext2
    location: *ext2.py* (glob)
    bundled: no
    tested with: 3.0 3.1 3.2.1
    bug reporting: https://example.org/bts
  histedit
    location: * (glob)
    bundled: yes
  hotfix1
    location: <sapling_ext_hotfix1>
    bundled: no
  rebase
    location: *rebase* (glob)
    bundled: yes
  remotenames
    location: *remotenames* (glob)
    bundled: no
  treemanifest
    location: *treemanifest* (glob)
    bundled: no

  $ hg debugextensions --excludedefault -Tjson | sed 's|\\\\|/|g'
  [
   {
    "buglink": "",
    "bundled": false,
    "name": "ext1",
    "source": "*/ext1.py*", (glob)
    "testedwith": []
   },
   {
    "buglink": "https://example.org/bts",
    "bundled": false,
    "name": "ext2",
    "source": "*/ext2.py*", (glob)
    "testedwith": ["3.0", "3.1", "3.2.1"]
   },
   {
    "buglink": "",
    "bundled": true,
    "name": "histedit",
    "source": "*histedit*", (glob)
    "testedwith": []
   },
   {
    "buglink": "",
    "bundled": false,
    "name": "hotfix1",
    "source": "<sapling_ext_hotfix1>",
    "testedwith": []
   },
   {
    "buglink": "",
    "bundled": true,
    "name": "rebase",
    "source": "*rebase*", (glob)
    "testedwith": []
   },
   {
    "buglink": "",
    "bundled": false,
    "name": "remotenames",
    "source": "*remotenames*", (glob)
    "testedwith": []
   },
   {
    "buglink": "",
    "bundled": false,
    "name": "treemanifest",
    "source": "*treemanifest*", (glob)
    "testedwith": []
   }
  ]

  $ hg debugextensions -T '{ifcontains("3.1", testedwith, "{name}\n")}'
  ext2
  $ hg debugextensions \
  > -T '{ifcontains("3.2", testedwith, "no substring match: {name}\n")}'
