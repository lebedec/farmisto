use std::{env, path::Path};

fn main() {
    if env::var("TARGET")
        .expect("target")
        .ends_with("windows-msvc")
    {
        let manifest_path = "farmisto.exe.manifest";
        let manifest = Path::new(manifest_path).canonicalize().unwrap();
        println!("cargo:rustc-link-arg-bins=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg-bins=/MANIFESTINPUT:{}",
            manifest.display()
        );
        println!("cargo:rerun-if-changed={manifest_path}");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
