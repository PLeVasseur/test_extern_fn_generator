use std::{env, fs};
use std::path::{Path, PathBuf};

fn main() -> miette::Result<()> {

    let project_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let runtime_wrapper_dir = project_root.join("src/wrappers/include");

    let mut b = autocxx_build::Builder::new("src/main.rs", &[&runtime_wrapper_dir])
        .extra_clang_args(&[
            "-I/usr/include/c++/11",
            "-I/usr/include/x86_64-linux-gnu/c++/11",
        ])
        .build()?;
    b.flag_if_supported("-std=c++17").compile("demo"); // arbitrary library name, pick anything
    println!("cargo:rerun-if-changed=src/main.rs");

    // Use a hardcoded value
    let num_fns = 50000;

    // Generate the Rust source file with the macro invocation
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated_macro_invocation.rs");
    fs::write(dest_path, format!("generate_extern_fns!({});", num_fns))
        .expect("Failed to write generated file");

    Ok(())
}