fn main() {
    // Tell Cargo the place of the shared libraries
    println!("cargo:rustc-link-search=../build/dist/resources/lib");
    // for linux
    println!("cargo:rustc-link-arg=-Wl,-rpath,'$ORIGIN'/resources/lib");
    // for macos
    println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path/resources/lib");
    println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path/../Resources/resources/lib");
    println!("cargo:rustc-link-lib=nrsc5");

    tauri_build::build()
}
