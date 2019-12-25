use std::env;

fn main() {
    if env::var("TARGET").unwrap().contains("-apple-") {
        println!("cargo:rustc-link-lib=framework=WebKit");
    }
}
