pub mod rtlsdr {
    use std::{sync::{atomic::AtomicBool, Arc, Mutex}, thread, time::Duration};

    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use soapysdr::{Device, Direction::Rx};
    use tauri::Window;
    use cpal;
    use num::complex::Complex32;

  pub struct RtlSdrState(Arc<Mutex<RtlSdrData>>);
  pub struct RtlSdrData {
    pub rtlsdr_device: Option<Arc<Device>>,
    pub radio_stream_thread: Option<thread::JoinHandle<()>>,
    pub shutdown_flag: Arc<AtomicBool>
  }

  impl RtlSdrState {
    pub fn new() -> Self {
      RtlSdrState(Arc::new(Mutex::new(RtlSdrData { rtlsdr_device: None, radio_stream_thread: None, shutdown_flag: Arc::new(AtomicBool::new(false)) })))
    }

    pub fn connect_to_sdr(&self) {
      let args = "driver=rtlsdr";
      let device = Device::new(args).expect("Failed to create a SoapySDR device");
      self.0.clone().lock().unwrap().rtlsdr_device = Some(Arc::new(device));
    }

    pub fn start_stream(&self, window: Window, fm_freq: String) {
      let rtlsdr_state = self.0.clone();

      if rtlsdr_state.lock().unwrap().rtlsdr_device.is_none() {
        self.connect_to_sdr();
      }

      let rtlsdr_dev = rtlsdr_state.lock().unwrap().rtlsdr_device.clone().unwrap();

      // set sample rate
      let sample_rate = 2.048e6;
      rtlsdr_dev.set_sample_rate(Rx, 0, sample_rate);

      // set center frequency
      let sdr_freq = fm_freq.parse::<f64>().expect("FM Frequency could not be parsed as a float") * 1_000_000.0;
      rtlsdr_dev.set_frequency(Rx, 0, sdr_freq, "").expect("Failed to set frequency");

      // create an RX stream
      let mut rx_stream = rtlsdr_dev.rx_stream::<Complex32>(&[0]).expect("Failed to create RX stream");

      // activate the stream
      rx_stream.activate(None).expect("Failed to activate RX stream");

      // setup audio output with cpal
      let host = cpal::default_host();
      let device = host.default_output_device().expect("Failed to get default output device");
      let config = device.default_output_config().expect("Failed to get default output config");

      let audio_sample_rate = config.sample_rate().0 as f32;
      let channels = config.channels() as usize;

      let err_fn = |err| eprintln!("An error occurred on output audio stream: {}", err);

      let audio_buffer = Arc::new(Mutex::new(Vec::new()));
      let audio_buffer_clone = audio_buffer.clone();

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
      }, err_fn, None).expect("Failed to build output stream");

      audio_stream.play().expect("Failed to play audio stream");

      // create buffer for the samples
      let mut buffer = vec![Complex32::new(0.0, 0.0); 4096];
      let ratio = (sample_rate / (audio_sample_rate as f64)) as usize;

      // create FmDemod object
      let mut fm_demoder = demod_fm::FmDemod::new(75_000, sample_rate as u32);

      // process the samples and stream to the audio output
      loop {
        match rx_stream.read(&mut [&mut buffer], Duration::from_secs(1).as_micros() as i64) {
          Ok(samples_read) => {
            let mut demodulated = Vec::with_capacity(samples_read);
            for &sample in &buffer[..samples_read] {
              demodulated.push(fm_demoder.feed(sample));
            }

            let audio_samples: Vec<_> = demodulated
              .iter()
              .step_by(ratio)
              .map(|&x| x)
              .collect();

            let mut buffer = audio_buffer.lock().unwrap();
            buffer.extend(audio_samples);
          },
          Err(err) => eprintln!("Error reading samples: {:?}", err)
        }
      }

      // deactivate the stream (currently unreachable)
      rx_stream.deactivate(None).expect("Failed to deactivate RX stream");
    }
  }
}