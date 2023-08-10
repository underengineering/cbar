use std::env;

fn main() {
    println!("cargo:rerun-if-env-changed=PROFILE");
    if env::var("PROFILE") == Ok("release".to_owned()) {
        // Use ld linker in release build as thin lto with mold breaks
        // linking to luajit
        println!("cargo:rustc-link-arg=-fuse-ld=ld");
    }
}
