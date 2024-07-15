// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::thread;
use tauri::{api::process::{Command, CommandEvent}, Manager, Window};

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![start_nrsc5])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[tauri::command]
fn start_nrsc5(window: Window) {
  thread::spawn(|| {
    let (mut rx, mut child) = Command::new_sidecar("nrsc5")
      .expect("failed to create `nrsc5` binary command")
      .args(["101.5", "0"])
      .spawn()
      .expect("Failed to spawn sidecar");

    tauri::async_runtime::spawn(async move {
      // read events
      while let Some(event) = rx.recv().await {
        if let CommandEvent::Stdout(line) = event {
          window.emit("message", Some(format!("'{}'", line)))
            .expect("failed to emit event");

          // write to stdin
          child.write("message from Rust\n".as_bytes()).unwrap();
        }
      }
    })
  });
}