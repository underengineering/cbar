use std::env;

fn main() {
    println!("cargo:rerun-if-env-changed=PROFILE");
    if env::var("PROFILE") == Ok("debug".to_owned()) {
        println!("cargo:rustc-link-arg=-fuse-ld=mold");
    }
}
