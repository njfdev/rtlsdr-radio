fn main() {
    // Tell Cargo the place of the shared libraries
    println!("cargo:rustc-link-search=../build/SoapySDR/build/lib");
    println!("cargo:rustc-link-search=native=./resources/libs");

    tauri_build::build()
}
