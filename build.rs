use copy_to_output::copy_to_output;
use std::env;

fn main() {
    println!("cargo:rerun-if-changed=static/*");
    copy_to_output("static", &env::var("PROFILE").unwrap()).expect("Could not copy");
}
