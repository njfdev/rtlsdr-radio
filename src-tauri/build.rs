fn main() {
    // Tell Cargo the place of the shared libraries
    println!("cargo:rustc-link-search=../build/lib");
    // for linux
    println!("cargo:rustc-link-arg=-Wl,-rpath,'$ORIGIN'/resources/lib");
    // for macos
    #[cfg(debug_assertions)]
    println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path/resources/lib");
    #[cfg(not(debug_assertions))]
    println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path/../Resources/resources/lib");

    tauri_build::build()
}
