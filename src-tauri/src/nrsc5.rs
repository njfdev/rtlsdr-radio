pub mod nrsc5 {
    use std::{sync::Arc, thread};

    use tauri::{api::process::{Command, CommandEvent}, async_runtime::{block_on, Mutex}, Window};

  pub struct Nrsc5State(Arc<Mutex<Nrsc5>>);
  pub struct Nrsc5 {
    pub nrsc5_thread: Option<thread::JoinHandle<()>>
  }

  impl Nrsc5State {
    pub fn new() -> Self {
      Nrsc5State(Arc::new(Mutex::new(Nrsc5 { nrsc5_thread: None })))
    }

    pub fn startThread(&self, window: Window, fm_freq: String, channel: String) {
      // we can make a clone because a clone of Arc is just making another reference to the original
      let nrsc5_state = self.0.clone();
      let nrsc5_thread = thread::spawn(|| {
        let (mut rx, mut child) = Command::new_sidecar("nrsc5")
          .expect("failed to create `nrsc5` binary command")
          .args([fm_freq, channel])
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
        });
      });

      let mut nrsc5 = block_on(nrsc5_state.lock());
      nrsc5.nrsc5_thread = Some(nrsc5_thread);
    }
  }
}