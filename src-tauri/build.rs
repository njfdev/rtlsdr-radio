fn main() {
    // Tell Cargo the place of the shared libraries
    println!("cargo:rustc-link-search=../build/SoapySDR/build/lib");
    // for linux
    println!("cargo:rustc-link-arg=-Wl,-rpath,'$ORIGIN'/resources/libs");
    // for macos
    println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path/resources/libs");

    tauri_build::build()
}
