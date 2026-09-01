fn main() {
    println!("cargo:rustc-link-arg-cdylib=--no-merge-data-segments");
}
