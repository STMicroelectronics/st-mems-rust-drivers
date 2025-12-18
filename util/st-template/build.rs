use std::io::Write;
{% if framework == "stm32rs" -%}
use std::env;
use std::path::{PathBuf, Path};
use std::fs::{self, File};
{% else -%}
use std::fs;
use std::path::Path;
{% endif -%}


fn main() {
{% if framework == "stm32rs" -%}
    // Put `memory.x` in our output directory and ensure it's
    // on the linker search path.
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("memory.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    // By default, Cargo will re-run a build script whenever
    // any file in the project changes. By specifying `memory.x`
    // here, we ensure the build script is only re-run when
    // `memory.x` is changed.
    println!("cargo:rerun-if-changed=memory.x");
{% endif -%}
    // Specify linker arguments.

    // `--nmagic` is required if memory section addresses are not aligned to 0x10000,
    // for example the FLASH and RAM sections in your `memory.x`.
    // See https://github.com/rust-embedded/cortex-m-quickstart/pull/95
    println!("cargo:rustc-link-arg=--nmagic");

    // Set the linker script to the one provided by cortex-m-rt.
    println!("cargo:rustc-link-arg=-Tlink.x");
    println!("cargo:rustc-link-arg=-Tdefmt.x");

    // Retrive the taget chip
    let chip = "{{svd_name}}.svd";

    let file_name = "{{svd_name}}.svd";
    let string_path = format!(".vscode/{file_name}");
    let output_path = Path::new(&string_path);
    let url = format!("https://stm32-rs.github.io/stm32-rs/{chip}.patched");

    if output_path.exists() {
        println!("SVD already exitsts, skipping download");
    } else {
        let response = reqwest::blocking::get(&url).unwrap();
        let content = response.text().unwrap();

        let mut file = fs::File::create(output_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        println!("SVD created at {:?}", output_path);
    }
}
