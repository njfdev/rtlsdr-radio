pub mod utils {
    use std::{env, fs, path::PathBuf};

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

        // Find the first libusb shared library in the resources/lib directory
        let libusb_path = find_libusb_library(&resource_dir, os_ext)
            .expect("Failed to find libusb shared library");

        // load libusb shared library
        unsafe {
            let _lib = Library::new(libusb_path).expect("Failed to load shared library");
        }
    }

    fn find_libusb_library(resource_dir: &PathBuf, os_ext: &str) -> Option<PathBuf> {
        let lib_dir = resource_dir.join("resources/lib");
        let entries = fs::read_dir(lib_dir).ok()?;

        for entry in entries {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name() {
                    if let Some(file_name_str) = file_name.to_str() {
                        if file_name_str.starts_with("libusb") && file_name_str.ends_with(os_ext) {
                            return Some(path);
                        }
                    }
                }
            }
        }
        None
    }
}
