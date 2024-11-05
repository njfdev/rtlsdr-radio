mod nrsc5_header;

use nrsc5_header::*;
use std::ffi::{c_void, CStr};
use std::os::raw::c_char;
use std::ptr;

///--------------------- nrsc5 Function Handling ---------------------///

pub struct Nrsc5 {
    pub nrsc5_state: *mut c_void,
}

unsafe impl Send for Nrsc5 {}

impl Nrsc5 {
    pub fn new() -> Self {
        unsafe {
            // Declare a mutable pointer to c_void
            let mut nrsc5_state: *mut c_void = ptr::null_mut();

            // Pass a pointer to the mutable variable `nrsc5_state`
            let result = nrsc5_open_pipe(&mut nrsc5_state);

            if result == 0 {
                // If the function succeeds, start and assign `nrsc5_state` to the struct
                nrsc5_start(nrsc5_state);

                Self { nrsc5_state }
            } else {
                panic!("Failed to open pipe");
            }
        }
    }

    pub fn pipe_samples(&self, samples: &[i16]) -> i32 {
        unsafe {
            // Convert the Rust slice to a pointer
            let samples_ptr = samples.as_ptr();
            // Call the C function
            nrsc5_pipe_samples_cs16(self.nrsc5_state, samples_ptr, samples.len() as u32)
        }
    }
}

impl Drop for Nrsc5 {
    fn drop(&mut self) {
        unsafe {
            nrsc5_stop(self.nrsc5_state);
            nrsc5_close(self.nrsc5_state);
        }
    }
}

pub fn get_nrsc5_version() -> Option<String> {
    unsafe {
        // Prepare a pointer to hold the C string
        let mut version: *const c_char = ptr::null();

        // Call the C function
        nrsc5_get_version(&mut version);

        // Convert the C string to a Rust string
        if !version.is_null() {
            let c_str = CStr::from_ptr(version);
            match c_str.to_str() {
                Ok(rust_str) => return Some(rust_str.to_string()),
                Err(_) => return None,
            }
        }
    }

    None
}
