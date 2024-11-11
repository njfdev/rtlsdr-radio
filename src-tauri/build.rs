use bindgen;
use std::{env, path::PathBuf};

fn main() {
    // Tell Cargo the place of the shared libraries
    println!("cargo:rustc-link-search=../build/dist/resources/lib");
    // for linux
    println!("cargo:rustc-link-arg=-Wl,-rpath,'$ORIGIN'/resources/lib");
    // for macos
    println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path/resources/lib");
    println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path/../Resources/resources/lib");
    println!("cargo:rustc-link-lib=nrsc5");

    // create Rust binds for C lib nrsc5
    let bindings = bindgen::Builder::default()
        .header("../build/dist/resources/include/nrsc5.h") // Adjust this path if needed
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("nrsc5_bindings.rs"))
        .expect("Couldn't write bindings!");

    tauri_build::build()
}
