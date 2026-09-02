use std::env::var;

fn main() {
    if let Ok(arch) = var("CARGO_CFG_TARGET_ARCH")
        && arch.strip_prefix("wasm").is_some()
    {
        println!("cargo:rustc-link-arg-cdylib=--no-merge-data-segments");
    }
}
