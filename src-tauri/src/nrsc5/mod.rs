use libc::c_char;
use std::ffi::CStr;
use std::ptr;

extern "C" {
    fn nrsc5_get_version(version: *mut *const c_char);
}

pub fn get_nrsc5_version() {
    unsafe {
        // Prepare a pointer to hold the C string
        let mut version: *const c_char = ptr::null();

        // Call the C function
        nrsc5_get_version(&mut version);

        // Convert the C string to a Rust string
        if !version.is_null() {
            let c_str = CStr::from_ptr(version);
            match c_str.to_str() {
                Ok(rust_str) => println!("Version: {}", rust_str),
                Err(_) => eprintln!("Failed to convert C string to Rust string"),
            }
        }
    }
}
