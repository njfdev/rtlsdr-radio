pub mod utils {
    use std::env;

    use libloading::Library;
    use tauri::{App, Manager};

    pub fn setup_dependencies(app: &mut App) {
        let resource_dir = app.path().resource_dir().unwrap();

        // set env for SoapySDR modules
        //env::set_var(key, value);
        let mut modules_path = resource_dir.join("resources/lib/SoapySDR/modules0.8/");
        env::set_var("SOAPY_SDR_PLUGIN_PATH", modules_path.as_mut_os_str());

        // Determine the correct file extension for the shared library based on the OS
        let os_ext = if cfg!(target_os = "windows") {
            ".dll"
        } else if cfg!(target_os = "macos") {
            ".dylib"
        } else {
            ".so"
        };

        // load libusb shared library
        unsafe {
            let _lib = Library::new(format!(
                "{}/resources/lib/libusb-1.0.0{}",
                resource_dir.as_os_str().to_str().unwrap(),
                os_ext
            ))
            .expect("Failed to load shared library");
        }
    }
}
