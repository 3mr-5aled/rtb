fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if let Ok(target) = std::env::var("TARGET") {
        println!("cargo:rustc-env=TARGET={target}");
    }
}
