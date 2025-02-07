// @generated by autocargo

use std::env;
use std::fs;
use std::path::Path;
use thrift_compiler::Config;
use thrift_compiler::GenContext;
const CRATEMAP: &str = "\
eden/mononoke/git/packfile/if/packfile_thrift.thrift crate //eden/mononoke/git/packfile/if:packfile-thrift-rust
thrift/annotation/rust.thrift rust //thrift/annotation:rust-rust
thrift/annotation/scope.thrift rust->scope //thrift/annotation:scope-rust
";
#[rustfmt::skip]
fn main() {
    println!("cargo:rerun-if-changed=thrift_build.rs");
    let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR env not provided");
    let cratemap_path = Path::new(&out_dir).join("cratemap");
    fs::write(cratemap_path, CRATEMAP).expect("Failed to write cratemap");
    Config::from_env(GenContext::Mocks)
        .expect("Failed to instantiate thrift_compiler::Config")
        .base_path("../../../../../..")
        .types_crate("packfile-thrift__types")
        .clients_crate("packfile-thrift__clients")
        .options("deprecated_default_enum_min_i32")
        .run(["../packfile_thrift.thrift"])
        .expect("Failed while running thrift compilation");
}
