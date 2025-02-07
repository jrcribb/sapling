// @generated by autocargo

use std::env;
use std::fs;
use std::path::Path;
use thrift_compiler::Config;
use thrift_compiler::GenContext;
const CRATEMAP: &str = "\
eden/mononoke/filenodes/if/filenodes.thrift crate //eden/mononoke/filenodes/if:filenodes-if-rust
eden/mononoke/mercurial/types/if/mercurial_thrift.thrift mercurial_thrift //eden/mononoke/mercurial/types/if:mercurial-thrift-rust
eden/mononoke/mononoke_types/serialization/blame.thrift mononoke_types_serialization //eden/mononoke/mononoke_types/serialization:mononoke_types_serialization-rust
eden/mononoke/mononoke_types/serialization/bonsai.thrift mononoke_types_serialization //eden/mononoke/mononoke_types/serialization:mononoke_types_serialization-rust
eden/mononoke/mononoke_types/serialization/bssm.thrift mononoke_types_serialization //eden/mononoke/mononoke_types/serialization:mononoke_types_serialization-rust
eden/mononoke/mononoke_types/serialization/ccsm.thrift mononoke_types_serialization //eden/mononoke/mononoke_types/serialization:mononoke_types_serialization-rust
eden/mononoke/mononoke_types/serialization/changeset_info.thrift mononoke_types_serialization //eden/mononoke/mononoke_types/serialization:mononoke_types_serialization-rust
eden/mononoke/mononoke_types/serialization/content.thrift mononoke_types_serialization //eden/mononoke/mononoke_types/serialization:mononoke_types_serialization-rust
eden/mononoke/mononoke_types/serialization/content_manifest.thrift mononoke_types_serialization //eden/mononoke/mononoke_types/serialization:mononoke_types_serialization-rust
eden/mononoke/mononoke_types/serialization/data.thrift mononoke_types_serialization //eden/mononoke/mononoke_types/serialization:mononoke_types_serialization-rust
eden/mononoke/mononoke_types/serialization/deleted_manifest.thrift mononoke_types_serialization //eden/mononoke/mononoke_types/serialization:mononoke_types_serialization-rust
eden/mononoke/mononoke_types/serialization/fastlog.thrift mononoke_types_serialization //eden/mononoke/mononoke_types/serialization:mononoke_types_serialization-rust
eden/mononoke/mononoke_types/serialization/fsnodes.thrift mononoke_types_serialization //eden/mononoke/mononoke_types/serialization:mononoke_types_serialization-rust
eden/mononoke/mononoke_types/serialization/id.thrift mononoke_types_serialization //eden/mononoke/mononoke_types/serialization:mononoke_types_serialization-rust
eden/mononoke/mononoke_types/serialization/path.thrift mononoke_types_serialization //eden/mononoke/mononoke_types/serialization:mononoke_types_serialization-rust
eden/mononoke/mononoke_types/serialization/raw_bundle2.thrift mononoke_types_serialization //eden/mononoke/mononoke_types/serialization:mononoke_types_serialization-rust
eden/mononoke/mononoke_types/serialization/redaction.thrift mononoke_types_serialization //eden/mononoke/mononoke_types/serialization:mononoke_types_serialization-rust
eden/mononoke/mononoke_types/serialization/sharded_map.thrift mononoke_types_serialization //eden/mononoke/mononoke_types/serialization:mononoke_types_serialization-rust
eden/mononoke/mononoke_types/serialization/skeleton_manifest.thrift mononoke_types_serialization //eden/mononoke/mononoke_types/serialization:mononoke_types_serialization-rust
eden/mononoke/mononoke_types/serialization/test_manifest.thrift mononoke_types_serialization //eden/mononoke/mononoke_types/serialization:mononoke_types_serialization-rust
eden/mononoke/mononoke_types/serialization/time.thrift mononoke_types_serialization //eden/mononoke/mononoke_types/serialization:mononoke_types_serialization-rust
eden/mononoke/mononoke_types/serialization/unodes.thrift mononoke_types_serialization //eden/mononoke/mononoke_types/serialization:mononoke_types_serialization-rust
thrift/annotation/rust.thrift mercurial_thrift->rust //thrift/annotation:rust-rust
thrift/annotation/scope.thrift mercurial_thrift->rust->scope //thrift/annotation:scope-rust
";
#[rustfmt::skip]
fn main() {
    println!("cargo:rerun-if-changed=thrift_build.rs");
    let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR env not provided");
    let cratemap_path = Path::new(&out_dir).join("cratemap");
    fs::write(cratemap_path, CRATEMAP).expect("Failed to write cratemap");
    Config::from_env(GenContext::Mocks)
        .expect("Failed to instantiate thrift_compiler::Config")
        .base_path("../../../../..")
        .types_crate("filenodes-if__types")
        .clients_crate("filenodes-if__clients")
        .run(["../filenodes.thrift"])
        .expect("Failed while running thrift compilation");
}
