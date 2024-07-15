pub mod nrsc5 {
    use core::time;
    use std::{sync::{atomic::{AtomicBool, Ordering}, Arc}, thread};

    use tauri::{api::process::{Command, CommandEvent}, async_runtime::{block_on, Mutex}, Window};

  pub struct Nrsc5State(Arc<Mutex<Nrsc5>>);
  pub struct Nrsc5 {
    pub nrsc5_thread: Option<thread::JoinHandle<()>>,
    pub shutdown_flag: Arc<AtomicBool>
  }

  impl Nrsc5State {
    pub fn new() -> Self {
      Nrsc5State(Arc::new(Mutex::new(Nrsc5 { nrsc5_thread: None, shutdown_flag: Arc::new(AtomicBool::new(false)) })))
    }

    pub fn startThread(&self, window: Window, fm_freq: String, channel: String) {
      // we can make a clone because a clone of Arc is just making another reference to the original
      let nrsc5_state = self.0.clone();
      let nrsc5_state_clone = nrsc5_state.clone();

      let shutdown_flag = block_on(nrsc5_state.lock()).shutdown_flag.clone();

      let nrsc5_thread = thread::spawn(move || {
        let (mut rx, mut child) = Command::new_sidecar("nrsc5")
          .expect("failed to create `nrsc5` binary command")
          .args([fm_freq, channel])
          .spawn()
          .expect("Failed to spawn sidecar");

        let shutdown_flag_clone = shutdown_flag.clone();

        tauri::async_runtime::spawn(async move {
          // read events
          while let Some(event) = rx.recv().await {
            if let CommandEvent::Stdout(line) = event {
              window.emit("message", Some(format!("'{}'", line)))
                .expect("failed to emit event");
            }
          }

          while !(shutdown_flag_clone.load(Ordering::SeqCst)) {
            thread::sleep(time::Duration::from_millis(100))
          }
        });

        while !(shutdown_flag.load(Ordering::SeqCst)) {
          thread::sleep(time::Duration::from_millis(100));
        }

        child.kill();
      });

      let mut nrsc5 = block_on(nrsc5_state.lock());
      nrsc5.nrsc5_thread = Some(nrsc5_thread);
    }

    pub fn stopThread(&self) {
      if let Ok(mut nrsc5) = self.0.try_lock() {

        nrsc5.shutdown_flag.store(true, Ordering::SeqCst);

        if let Some(thread) = nrsc5.nrsc5_thread.take() {
          thread.join().expect("Failed to join thread");
        }

        nrsc5.shutdown_flag.store(false, Ordering::SeqCst);
      } else {
        println!("Could not acquire lock immediately");
      }
    }
  }
}