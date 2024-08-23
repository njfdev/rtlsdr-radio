fn main() {
    // Tell Cargo the place of the shared libraries
    println!("cargo:rustc-link-search=../build/usr/local/lib");
    // for linux
    println!("cargo:rustc-link-arg=-Wl,-rpath,'$ORIGIN'/resources/lib");
    // for macos
    println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path/resources/lib");
    println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path/../Resources/resources/lib");

    tauri_build::build()
}
