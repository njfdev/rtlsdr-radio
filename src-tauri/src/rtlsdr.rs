pub mod rtlsdr {
    use std::{sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex}, thread, time::Duration};

    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    //use soapysdr::{Device, Direction::Rx};
    use rtlsdr::RTLSDRDevice;
    use tauri::Window;
    use cpal;
    use num::{complex::Complex32, Complex};

  pub struct RtlSdrState(Arc<Mutex<RtlSdrData>>);
  pub struct RtlSdrData {
    pub radio_stream_thread: Option<thread::JoinHandle<()>>,
    pub shutdown_flag: Arc<AtomicBool>
  }

  impl RtlSdrState {
    pub fn new() -> Self {
      RtlSdrState(Arc::new(Mutex::new(RtlSdrData { radio_stream_thread: None, shutdown_flag: Arc::new(AtomicBool::new(false)) })))
    }

    pub fn start_stream(&self, window: Window, fm_freq: String) {
      let rtlsdr_state = self.0.clone();
      let rtlsdr_state_clone = rtlsdr_state.clone();

      let shutdown_flag = rtlsdr_state.lock().unwrap().shutdown_flag.clone();

      rtlsdr_state.lock().unwrap().radio_stream_thread = Some(thread::spawn(move || {
        // connect to SDR
        let mut rtlsdr_dev = rtlsdr::open(0).expect("Failed to connect to RTL-SDR");

        // set sample rate
        let sample_rate = 2.048e6 as u32;
        rtlsdr_dev.set_sample_rate(sample_rate);

        // set center frequency
        let sdr_freq = (fm_freq.parse::<f32>().expect("FM Frequency could not be parsed as a float") * 1_000_000.0) as u32;
        rtlsdr_dev.set_center_freq( sdr_freq).expect("Failed to set frequency");

        // setup audio output with cpal
        let host = cpal::default_host();
        let device = host.default_output_device().expect("Failed to get default output device");
        let config = device.default_output_config().expect("Failed to get default output config");

        let audio_sample_rate = config.sample_rate().0 as f32;
        let channels = config.channels() as usize;

        let err_fn = |err| eprintln!("An error occurred on output audio stream: {}", err);

        let audio_buffer = Arc::new(Mutex::new(Vec::new()));
        let audio_buffer_clone = audio_buffer.clone();

        let can_close = Arc::new(AtomicBool::new(true));
        let can_close_clone = can_close.clone();

        let audio_stream = device.build_output_stream(&config.into(), move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
          let mut buffer = audio_buffer_clone.lock().unwrap();
          for frame in data.chunks_mut(channels) {
            if let Some(sample) = buffer.pop() {
              for sample_out in frame.iter_mut() {
                *sample_out = sample;
            }
            } else {
              for sample_out in frame.iter_mut() {
                *sample_out = 0.0;
              }
            }
          }
          can_close_clone.store(true, Ordering::SeqCst);
        }, err_fn, None).expect("Failed to build output stream");

        audio_stream.play().expect("Failed to play audio stream");
        rtlsdr_dev.reset_buffer();

        // create buffer for the samples
        let ratio = (sample_rate / audio_sample_rate as u32) as usize;

        // create FmDemod object
        let mut fm_demoder = demod_fm::FmDemod::new(75_000, sample_rate as u32);

        // notify frontend that audio is playing
        window.emit("rtlsdr_status", "running")
          .expect("failed to emit event");

        // process the samples and stream to the audio output
        while !(shutdown_flag.load(Ordering::SeqCst)) {
          match rtlsdr_dev.read_sync(4096) {
            Ok(samples_read) => {
              let mut demodulated = Vec::with_capacity(samples_read.len() / 2);

              for chunk in samples_read.chunks(2) {
                if chunk.len() == 2 {
                  let i = (chunk[0] as i8) as f32 / 128.0;
                  let q = (chunk[1] as i8) as f32 / 128.0;
                  let complex_sample = Complex::new(i, q);
                  demodulated.push(fm_demoder.feed(complex_sample));
                }
              }

              let audio_samples: Vec<_> = demodulated
                .iter()
                .step_by(ratio)
                .map(|&x| x)
                .collect();

              can_close.store(false, Ordering::SeqCst);

              let mut buffer = audio_buffer.lock().unwrap();
              buffer.extend(audio_samples);
            },
            Err(err) => eprintln!("Error reading samples: {:?}", err)
          }
        }

        while !can_close.load(Ordering::SeqCst) {
          thread::sleep(Duration::from_millis(100));
        }
      }));
    }

    pub fn stop_stream(&self, window: Window) {
      if let Ok(mut rtlSdrData) = self.0.clone().lock() {

        rtlSdrData.shutdown_flag.store(true, Ordering::SeqCst);

        if let Some(thread) = rtlSdrData.radio_stream_thread.take() {
          thread.join().expect("Failed to join thread");
        }

        rtlSdrData.shutdown_flag.store(false, Ordering::SeqCst);

        window.emit("rtlsdr_status", Some("idle"))
          .expect("failed to emit event");
      } else {
        println!("Could not acquire lock immediately");
        return;
      }
    }
  }
}