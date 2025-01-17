pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/nrsc5_bindings.rs"));
}

use bindings::{
    nrsc5_callback_t, nrsc5_close, nrsc5_event_t, nrsc5_get_version, nrsc5_open, nrsc5_open_pipe,
    nrsc5_pipe_samples_cs16, nrsc5_set_callback, nrsc5_set_frequency, nrsc5_start, nrsc5_stop,
    nrsc5_t, NRSC5_EVENT_AUDIO, NRSC5_EVENT_ID3,
};
use std::env;
use std::ffi::{c_void, CStr};
use std::os::raw::c_char;
use std::ptr;

///--------------------- nrsc5 Function Handling ---------------------///

pub struct Nrsc5 {
    pub nrsc5_state: *mut nrsc5_t,
    callback: nrsc5_callback_t,
    opaque: *mut c_void,
}

unsafe impl Send for Nrsc5 {}

impl Nrsc5 {
    pub fn new(callback: nrsc5_callback_t, opaque: *mut c_void) -> Self {
        let result = Self::init(callback, opaque);
        if result.is_ok() {
            Self {
                nrsc5_state: result.unwrap(),
                callback: callback,
                opaque: opaque,
            }
        } else {
            panic!("Error while initiating nrsc5: {}", result.unwrap_err());
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

    pub fn reset_state(&mut self) {
        unsafe {
            nrsc5_stop(self.nrsc5_state);
            nrsc5_close(self.nrsc5_state);
        }
        self.nrsc5_state = Self::init(self.callback, self.opaque).unwrap();
    }

    fn init(callback: nrsc5_callback_t, opaque: *mut c_void) -> Result<*mut nrsc5_t, String> {
        // Declare a mutable pointer to c_void
        let mut nrsc5_state: *mut nrsc5_t = ptr::null_mut();

        unsafe {
            // Pass a pointer to the mutable variable `nrsc5_state` to create the object
            let mut result = nrsc5_open_pipe(&mut nrsc5_state);

            if result == 0 {
                // set the callback
                nrsc5_set_callback(nrsc5_state, callback, opaque);

                // If the function succeeds, start and assign `nrsc5_state` to the struct
                nrsc5_start(nrsc5_state);

                let cstr = unsafe { CStr::from_ptr(nrsc5_state as *const _) }.to_string_lossy();
                println!("{}", cstr);

                return Ok(nrsc5_state);
            } else {
                return Err("Failed to open pipe".to_string());
            }
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
