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

    pub fn start_thread(&self, window: Window, fm_freq: String, channel: String) {
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
            match event {
              CommandEvent::Stdout(line) => {
                print!("{}", line);
              }
              // for some reason all nrsc5 output is found under Stderr
              CommandEvent::Stderr(line) => {
                print!("{}", line);
                Nrsc5State::handle_nrsc5_output(&window, line);
              }
              CommandEvent::Error(line) => {
                eprint!("Sidecar Error: {}", line);
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

      let mut nrsc5 = block_on(nrsc5_state.lock());
      nrsc5.nrsc5_thread = Some(nrsc5_thread);
    }

    pub fn stop_thread(&self, window: Window) {
      if let Ok(mut nrsc5) = self.0.try_lock() {

        nrsc5.shutdown_flag.store(true, Ordering::SeqCst);

        if let Some(thread) = nrsc5.nrsc5_thread.take() {
          thread.join().expect("Failed to join thread");
        }

        nrsc5.shutdown_flag.store(false, Ordering::SeqCst);

        window.emit("nrsc5_status", Some("stopped"))
          .expect("failed to emit event");
      } else {
        println!("Could not acquire lock immediately");
      }
    }

    fn is_timestamp(string: String) -> bool {
      let parts: Vec<&str> = string.split(":").collect();

      if parts.len() != 3 {
        return false;
      }

      if parts[0].parse::<usize>().is_err() || parts[1].parse::<usize>().is_err() || parts[2].parse::<usize>().is_err() {
        return false;
      }

      return true;
    }

    fn handle_nrsc5_output(window: &Window, line: String) {
      if line.starts_with("Found") {
        window.emit("nrsc5_status", Some("sdr-found"))
          .expect("failed to emit event");
      } else if Nrsc5State::is_timestamp(line.split(" ").nth(0).expect("Unexpected output from nrsc5").to_owned()) {
        let message  = line.split(" ").skip(1).collect::<Vec<&str>>().join(" ");

        // continuously send synchronized message to keep frontend updated (a timestamp always means synced)
        if (!message.starts_with("Lost synchronization")) {
          window.emit("nrsc5_status", Some("synchronized"))
            .expect("failed to emit event");
        } else {
          window.emit("nrsc5_status", Some("synchronization_lost"))
            .expect("failed to emit event");
        }

        if message.starts_with("Title: ") {
          window.emit("nrsc5_title", message.strip_prefix("Title: "))
            .expect("failed to emit event");
        } else if message.starts_with("Artist: ") {
          window.emit("nrsc5_artist", message.strip_prefix("Artist: "))
            .expect("failed to emit event");
        } else if message.starts_with("Audio bit rate: ") {
          window.emit("nrsc5_br", message.strip_prefix("Audio bit rate: "))
            .expect("failed to emit event");
        } else if message.starts_with("Station name: ") {
          window.emit("nrsc5_station", message.strip_prefix("Station name: "))
            .expect("failed to emit event");
        } else if message.starts_with("Slogan: ") {
          window.emit("nrsc5_slogan", message.strip_prefix("Slogan: "))
            .expect("failed to emit event");
        } else if message.starts_with("Message: ") {
          window.emit("nrsc5_message", message.strip_prefix("Message: "))
            .expect("failed to emit event");
        } else if message.starts_with("BER: ") {
          window.emit("nrsc5_ber", message.strip_prefix("BER: ").unwrap().split(",").nth(0).unwrap())
            .expect("failed to emit event");
        }
      }
    }
  }
}