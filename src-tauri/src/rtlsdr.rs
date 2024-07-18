pub mod rtlsdr {
    use std::{sync::{atomic::AtomicBool, Arc, Mutex}, thread};

    use num_complex::Complex32;
    use soapysdr::{Device, Direction::Rx};
    use tauri::Window;

  pub struct RtlSdrState(Arc<Mutex<RtlSdrData>>);
  pub struct RtlSdrData {
    pub rtlsdr_device: Option<Device>,
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
      self.0.clone().lock().unwrap().rtlsdr_device = Some(device);
    }

    pub fn start_stream(&self, window: Window, fm_freq: String) {
      let rtlsdr_state = self.0.clone();

      if rtlsdr_state.lock().unwrap().rtlsdr_device.is_none() {
        self.connect_to_sdr();
      }

      // set sample rate
      let sample_rate = 2.048e6;
      rtlsdr_state.lock().unwrap().rtlsdr_device.as_ref().unwrap().set_sample_rate(Rx, 0, sample_rate);

      // set center frequency
      let sdr_freq = fm_freq.parse::<f64>().expect("FM Frequency could not be parsed as a float") * 1_000_000.0;
      rtlsdr_state.lock().unwrap().rtlsdr_device.as_ref().unwrap().set_frequency(Rx, 0, sdr_freq, "").expect("Failed to set frequency");


      // create an RX stream
      let mut rx_stream = rtlsdr_state.lock().unwrap().rtlsdr_device.as_ref().unwrap().rx_stream::<Complex32>(&[0]).expect("Failed to create RX stream");

      // activate the stream
      rx_stream.activate(None).expect("Failed to activate RX stream");
    }
  }
}