use core::time;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
};

use log::{debug, error, info};
use tauri::{AppHandle, Emitter};
use tauri_plugin_shell::{process::CommandEvent, ShellExt};

pub struct Nrsc5State(Arc<Mutex<Nrsc5>>);
pub struct Nrsc5 {
    pub nrsc5_thread: Option<thread::JoinHandle<()>>,
    pub shutdown_flag: Arc<AtomicBool>,
}

impl Nrsc5State {
    pub fn new() -> Self {
        Nrsc5State(Arc::new(Mutex::new(Nrsc5 {
            nrsc5_thread: None,
            shutdown_flag: Arc::new(AtomicBool::new(false)),
        })))
    }

    pub fn start_thread(&self, app: AppHandle, fm_freq: String, channel: String) {
        // we can make a clone because a clone of Arc is just making another reference to the original
        let nrsc5_state = self.0.clone();

        let shutdown_flag = nrsc5_state.lock().unwrap().shutdown_flag.clone();

        let nrsc5_thread = thread::spawn(move || {
            let (mut rx, child) = app
                .shell()
                .sidecar("nrsc5")
                .expect("failed to create `nrsc5` binary command")
                .args([fm_freq, channel])
                .spawn()
                .expect("Failed to spawn sidecar");

            let shutdown_flag_clone = shutdown_flag.clone();

            tauri::async_runtime::spawn(async move {
                // read events
                while let Some(event) = rx.recv().await {
                    match event {
                        CommandEvent::Stdout(line) => {
                            debug!("{}", String::from_utf8(line).unwrap());
                        }
                        // for some reason all nrsc5 output is found under Stderr
                        CommandEvent::Stderr(line) => {
                            let string_output = String::from_utf8(line).unwrap();
                            debug!("{}", string_output);
                            Nrsc5State::handle_nrsc5_output(app.clone(), string_output);
                        }
                        CommandEvent::Error(line) => {
                            error!("Sidecar Error: {}", line);
                        }
                        CommandEvent::Terminated(payload) => {
                            info!(
                                "Nrsc5 Terminated with exit code {}",
                                payload.code.unwrap_or(-1)
                            );
                        }
                        _ => {}
                    }
                }

                while !(shutdown_flag_clone.load(Ordering::SeqCst)) {
                    thread::sleep(time::Duration::from_millis(100))
                }
            });

            while !(shutdown_flag.load(Ordering::SeqCst)) {
                thread::sleep(time::Duration::from_millis(100));
            }

            let _ = child.kill();
        });

        let mut nrsc5 = nrsc5_state.lock().unwrap();
        nrsc5.nrsc5_thread = Some(nrsc5_thread);
    }

    pub fn stop_thread(&self, app: AppHandle) {
        if let Ok(mut nrsc5) = self.0.try_lock() {
            nrsc5.shutdown_flag.store(true, Ordering::SeqCst);

            if let Some(thread) = nrsc5.nrsc5_thread.take() {
                thread.join().expect("Failed to join thread");
            }

            nrsc5.shutdown_flag.store(false, Ordering::SeqCst);

            app.emit("nrsc5_status", Some("stopped"))
                .expect("failed to emit event");
        } else {
            error!("Could not acquire lock immediately");
        }
    }

    pub fn is_playing(&self) -> bool {
        return self.0.clone().lock().unwrap().nrsc5_thread.is_some();
    }

    fn is_timestamp(string: String) -> bool {
        let parts: Vec<&str> = string.split(":").collect();

        if parts.len() != 3 {
            return false;
        }

        if parts[0].parse::<usize>().is_err()
            || parts[1].parse::<usize>().is_err()
            || parts[2].parse::<usize>().is_err()
        {
            return false;
        }

        return true;
    }

    fn handle_nrsc5_output(app: AppHandle, line: String) {
        if line.starts_with("Found") {
            app.emit("nrsc5_status", Some("sdr-found"))
                .expect("failed to emit event");
        } else if Nrsc5State::is_timestamp(
            line.split(" ")
                .nth(0)
                .expect("Unexpected output from nrsc5")
                .to_owned(),
        ) {
            let message = line.split(" ").skip(1).collect::<Vec<&str>>().join(" ");

            // continuously send synchronized message to keep frontend updated (a timestamp always means synced)
            if !message.starts_with("Lost synchronization") {
                app.emit("nrsc5_status", Some("synchronized"))
                    .expect("failed to emit event");
            } else {
                app.emit("nrsc5_status", Some("synchronization_lost"))
                    .expect("failed to emit event");
            }

            if message.starts_with("Title: ") {
                app.emit("nrsc5_title", message.strip_prefix("Title: "))
                    .expect("failed to emit event");
            } else if message.starts_with("Artist: ") {
                app.emit("nrsc5_artist", message.strip_prefix("Artist: "))
                    .expect("failed to emit event");
            } else if message.starts_with("Audio bit rate: ") {
                app.emit("nrsc5_br", message.strip_prefix("Audio bit rate: "))
                    .expect("failed to emit event");
            } else if message.starts_with("Station name: ") {
                app.emit("nrsc5_station", message.strip_prefix("Station name: "))
                    .expect("failed to emit event");
            } else if message.starts_with("Slogan: ") {
                app.emit("nrsc5_slogan", message.strip_prefix("Slogan: "))
                    .expect("failed to emit event");
            } else if message.starts_with("Message: ") {
                app.emit("nrsc5_message", message.strip_prefix("Message: "))
                    .expect("failed to emit event");
            } else if message.starts_with("BER: ") {
                app.emit(
                    "nrsc5_ber",
                    message
                        .strip_prefix("BER: ")
                        .unwrap()
                        .split(",")
                        .nth(0)
                        .unwrap(),
                )
                .expect("failed to emit event");
            }
        }
    }
}
