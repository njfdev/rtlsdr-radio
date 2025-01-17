use std::{
    ffi::{c_char, c_void, CStr},
    fs,
    ops::Deref,
    path::Path,
    ptr::{self, null, null_mut},
    sync::{Arc, Mutex},
};

use crate::{
    modes::*,
    nrsc5::{
        bindings::{
            nrsc5_event_t, nrsc5_program_type_name, nrsc5_service_data_type_name,
            NRSC5_ACCESS_PUBLIC, NRSC5_AUDIO_FRAME_SAMPLES, NRSC5_EVENT_AUDIO, NRSC5_EVENT_BER,
            NRSC5_EVENT_HDC, NRSC5_EVENT_ID3, NRSC5_EVENT_LOST_SYNC, NRSC5_EVENT_LOT,
            NRSC5_EVENT_MER, NRSC5_EVENT_PACKET, NRSC5_EVENT_SIG, NRSC5_EVENT_SIS,
            NRSC5_EVENT_STREAM, NRSC5_EVENT_SYNC, NRSC5_MIME_JPEG, NRSC5_MIME_PNG,
            NRSC5_MIME_PRIMARY_IMAGE, NRSC5_MIME_STATION_LOGO, NRSC5_SAMPLE_RATE_AUDIO,
            NRSC5_SIG_COMPONENT_AUDIO, NRSC5_SIG_SERVICE_AUDIO,
        },
        Nrsc5,
    },
};
use base64::Engine;
use radiorust::{
    flow::{new_receiver, new_sender, ReceiverConnector, SenderConnector},
    impl_block_trait,
    numbers::Float,
    prelude::{ChunkBufPool, Complex},
    signal::Signal,
};
use serde::Serialize;
use tauri::ipc::Channel;
use tokio::{spawn, sync::watch};

pub struct HdRadioDecode<Flt> {
    receiver_connector: ReceiverConnector<Signal<Complex<Flt>>>,
    sender_connector: SenderConnector<Signal<Complex<Flt>>>,
    program: watch::Sender<u32>,
    should_reset: watch::Sender<bool>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct HdRadioState {
    pub program: u32,
    last_program: u32,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub genre: String,
    pub thumbnail_data: Option<String>,
    pub audio_bitrate: f32,
    audio_packets: u32,
    audio_bytes: u32,
    pub lot_id: i32,
    // a list of ports in the format (port_mime, port_number)
    pub ports: Vec<(u32, u16)>,
    pub ber: f32,
    pub station_info: Option<StationInfo>,
}

impl HdRadioState {
    pub fn new(program: u32) -> Self {
        Self {
            program: program,
            last_program: program,
            title: String::new(),
            artist: String::new(),
            album: String::new(),
            genre: String::new(),
            thumbnail_data: None,
            audio_bitrate: 0.0,
            audio_packets: 0,
            audio_bytes: 0,
            lot_id: -1,
            ports: vec![],
            ber: 0.0,
            station_info: None,
        }
    }

    pub fn increase_audio_bytes(&mut self, bytes: u32) {
        // increase by the new audio packet
        self.audio_bytes += bytes;
        self.audio_packets += 1;

        // update the bitrate if necessary
        if self.audio_packets >= 32 {
            self.update_audio_bitrate();
        }
    }

    fn update_audio_bitrate(&mut self) {
        self.audio_bitrate = (self.audio_bytes as f32 * 8.0 * NRSC5_SAMPLE_RATE_AUDIO as f32
            / NRSC5_AUDIO_FRAME_SAMPLES as f32
            / self.audio_packets as f32
            / 100.0)
            .round()
            / 10.0;
        self.audio_bytes = 0;
        self.audio_packets = 0;
    }

    fn set_last_program(&mut self, program: u32) {
        self.last_program = program;
    }

    fn get_last_program(&self) -> u32 {
        self.last_program
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct StationInfo {
    pub name: String,
    pub country_code: String,
    pub fcc_id: i32,
    pub slogan: String,
    pub message: String,
    pub alert: String,
    pub location: (f32, f32),
    // in meters
    pub altitude: i32,
    pub audio_services: Vec<AudioService>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct AudioService {
    pub program: u32,
    pub service_type: String,
    pub is_restricted: bool,
    pub sound_experience: String,
}

pub struct Nrsc5CallbackOpaque {
    state: HdRadioState,
    callback: Arc<dyn Fn(HdRadioState) + Send + Sync>,
    audio_samples: Arc<Mutex<Vec<i16>>>,
}

impl Nrsc5CallbackOpaque {
    pub fn extend_audio_samples(&self, samples: &[i16]) {
        let audio_samples_arc = self.audio_samples.clone();
        let mut audio_samples = audio_samples_arc.lock();
        audio_samples.as_mut().unwrap().extend_from_slice(samples);
    }

    pub fn get_audio_samples(&self) -> Vec<i16> {
        let audio_samples_arc = self.audio_samples.clone();
        let audio_samples = audio_samples_arc.lock().as_mut().unwrap().clone();
        audio_samples_arc.lock().as_mut().unwrap().clear();
        return audio_samples;
    }
}

#[derive(Clone)]
pub struct LotFile {
    lot_id: u32,
    mime_type: u32,
    port: u16,
    data: Vec<u8>,
}

// the structure is Vec<(fcc_id, Vec<(lot_id, mime_type, port, data)>)>
static mut NRSC5_LOT_FILES: Vec<(i32, Vec<LotFile>)> = vec![];

unsafe fn store_lot_file(fcc_id: i32, lot_id: u32, mime_type: u32, port: u16, data: &[u8]) {
    let mut station_lots = NRSC5_LOT_FILES.iter_mut().find(|val| val.0 == fcc_id);
    if station_lots.is_none() {
        NRSC5_LOT_FILES.push((fcc_id, vec![]));
        station_lots = NRSC5_LOT_FILES.get_mut(NRSC5_LOT_FILES.len() - 1);
    }
    let station_lots = station_lots.unwrap();

    let old_lot_file_index = station_lots
        .1
        .iter()
        .enumerate()
        .find(|(i, val)| val.lot_id == lot_id);
    if old_lot_file_index.is_some() {
        station_lots.1.remove(old_lot_file_index.unwrap().0);
    }
    station_lots.1.push(LotFile {
        lot_id: lot_id,
        mime_type: mime_type,
        port: port,
        data: data.to_vec(),
    });
}

unsafe fn get_lot_file(fcc_id: i32, lot_id: i32, port: i32) -> Option<LotFile> {
    let station_lots = NRSC5_LOT_FILES.iter().find(|val| val.0 == fcc_id);
    if station_lots.is_none() {
        return None;
    }
    let station_lots = station_lots.unwrap();

    let lot_file = station_lots
        .1
        .iter()
        .find(|val| if lot_id == -1 { true } else { val.lot_id == lot_id as u32 } && if port == -1 { true } else { val.port == port as u16 });
    if lot_file.is_none() {
        return None;
    }

    Some(lot_file.unwrap().clone())
}

fn lot_to_base64_url(lot_file: LotFile) -> String {
    format!(
        "data:image/{};base64,{}",
        if lot_file.mime_type == NRSC5_MIME_PNG {
            "png"
        } else {
            "jpeg"
        },
        base64::engine::general_purpose::STANDARD.encode(&lot_file.data)
    )
}

unsafe extern "C" fn nrsc5_custom_callback(event: *const nrsc5_event_t, opaque: *mut c_void) {
    if opaque.is_null() {
        println!("The nrsc5 callback opaque value is empty!");
        return;
    }
    let callback_opaque = &mut *(opaque as *mut Nrsc5CallbackOpaque);
    let orig_state = callback_opaque.state.clone();
    let mut lots_updated = false;

    if (*event).event == NRSC5_EVENT_ID3
        && (*event).__bindgen_anon_1.id3.program == callback_opaque.state.program
    {
        let raw_title = (*event).__bindgen_anon_1.id3.title;
        if !raw_title.is_null() {
            let title = CStr::from_ptr(raw_title).to_string_lossy();
            callback_opaque.state.title = title.to_string();
            //println!("Title: {}", title);
        }
        let raw_artist = (*event).__bindgen_anon_1.id3.artist;
        if !raw_artist.is_null() {
            let artist = CStr::from_ptr(raw_artist).to_string_lossy();
            callback_opaque.state.artist = artist.to_string();
            //println!("Artist: {}", artist);
        }
        let raw_album = (*event).__bindgen_anon_1.id3.album;
        if !raw_album.is_null() {
            let album = CStr::from_ptr(raw_album).to_string_lossy();
            callback_opaque.state.album = album.to_string();
            //println!("Album: {}", album);
        }
        let raw_genre = (*event).__bindgen_anon_1.id3.genre;
        if !raw_genre.is_null() {
            let genre = CStr::from_ptr(raw_genre).to_string_lossy();
            callback_opaque.state.genre = genre.to_string();
            //println!("Genre: {}", genre);
        }

        // get ufid data
        let raw_ufid_owner = (*event).__bindgen_anon_1.id3.ufid.owner;
        if !raw_ufid_owner.is_null() {
            let ufid_owner = CStr::from_ptr(raw_ufid_owner).to_string_lossy();
            println!("UFID Owner: {}", ufid_owner);
        }
        let raw_ufid_id = (*event).__bindgen_anon_1.id3.ufid.id;
        if !raw_ufid_id.is_null() {
            let ufid_id = CStr::from_ptr(raw_ufid_id).to_string_lossy();
            println!("UFID ID: {}", ufid_id);
        }

        let lot_id = (*event).__bindgen_anon_1.id3.xhdr.lot;
        callback_opaque.state.lot_id = lot_id;
    } else if (*event).event == NRSC5_EVENT_AUDIO
        && ((*event).__bindgen_anon_1.audio.program == callback_opaque.state.program
            || (*event).__bindgen_anon_1.audio.program == callback_opaque.state.get_last_program())
    {
        /* If we are transitioning from the last program, we waited to update
         * until the last program audio chunk is sent so we don't forever add
         * tiny in-between buffers that make switching take longer and longer.
         */
        if (*event).__bindgen_anon_1.audio.program == callback_opaque.state.get_last_program()
            && callback_opaque.state.program != callback_opaque.state.get_last_program()
        {
            callback_opaque
                .state
                .set_last_program(callback_opaque.state.program);
        }

        // only add new audio samples if the new chunk is for the current program and we are not waiting to clear the old buffer
        if (*event).__bindgen_anon_1.audio.program == callback_opaque.state.program
            && (*event).__bindgen_anon_1.audio.program == callback_opaque.state.get_last_program()
        {
            let data_ptr = (*event).__bindgen_anon_1.audio.data;
            let data_len = (*event).__bindgen_anon_1.audio.count as usize;
            // Safety: We assume that the data pointer is valid and has the correct length.
            let audio_data = std::slice::from_raw_parts(data_ptr, data_len);

            // update AUDIO_SAMPLES
            callback_opaque.extend_audio_samples(audio_data);
        }
    } else if (*event).event == NRSC5_EVENT_LOT {
        println!(
            "-----------------Name: {}",
            CStr::from_ptr((*event).__bindgen_anon_1.lot.name)
                .to_str()
                .unwrap()
        );
        let path_string = ("../temp/".to_owned()
            + CStr::from_ptr((*event).__bindgen_anon_1.lot.name)
                .to_str()
                .unwrap());
        let image_path = Path::new(path_string.as_str());
        let binary_data = std::slice::from_raw_parts(
            (*event).__bindgen_anon_1.lot.data,
            (*event).__bindgen_anon_1.lot.size as usize,
        );
        fs::write(image_path, binary_data);

        // TODO: store lot files in a DB and remove them after their expiry date (currently, memory usage will increase continually until restarting the app)
        // save lot file to buffer
        store_lot_file(
            callback_opaque.state.station_info.as_ref().unwrap().fcc_id,
            (*event).__bindgen_anon_1.lot.lot,
            (*event).__bindgen_anon_1.lot.mime,
            (*event).__bindgen_anon_1.lot.port,
            binary_data,
        );

        println!(
            "  LOT: {}, MIME: {:#x}, Port: {}, Size: {}",
            (*event).__bindgen_anon_1.lot.lot,
            (*event).__bindgen_anon_1.lot.mime,
            (*event).__bindgen_anon_1.lot.port,
            (*event).__bindgen_anon_1.lot.size
        );
        lots_updated = true;
    } else if (*event).event == NRSC5_EVENT_SYNC {
        println!("Synced to Station");
    } else if (*event).event == NRSC5_EVENT_LOST_SYNC {
        println!("Lost Sync to Station");
    } else if (*event).event == NRSC5_EVENT_BER {
        callback_opaque.state.ber = (*event).__bindgen_anon_1.ber.cber;
        // println!(
        //     "Bit Error Ratio: {}%",
        //     (*event).__bindgen_anon_1.ber.cber * 100.0
        // );
    } else if (*event).event == NRSC5_EVENT_MER {
        // println!(
        //     "Modulation Error Ratio: Lower {}, Upper {}",
        //     (*event).__bindgen_anon_1.mer.lower,
        //     (*event).__bindgen_anon_1.mer.upper
        // );
    } else if (*event).event == NRSC5_EVENT_SIG {
        println!("Station Channels:");
        let mut cur_sig = (*event).__bindgen_anon_1.sig.services;
        while !cur_sig.is_null() {
            let raw_name = (*cur_sig).name;
            if !raw_name.is_null() {
                let name = CStr::from_ptr(raw_name).to_string_lossy();
                println!(
                    "  {}. {} ({})",
                    (*cur_sig).number,
                    name,
                    if (*cur_sig).type_ == NRSC5_SIG_SERVICE_AUDIO as u8 {
                        "Audio"
                    } else {
                        "Data"
                    }
                );

                let mut cur_component = (*cur_sig).components;
                while !cur_component.is_null() {
                    if (*cur_component).type_ == NRSC5_SIG_COMPONENT_AUDIO as u8 {
                        println!(
                            "      Audio Component (MIME {:#x} - PORT {})",
                            (*cur_component).__bindgen_anon_1.audio.mime,
                            (*cur_component).__bindgen_anon_1.audio.port
                        );
                    } else {
                        println!(
                            "      Data Component (MIME {:#x} - PORT {})",
                            (*cur_component).__bindgen_anon_1.data.mime,
                            (*cur_component).__bindgen_anon_1.data.port
                        );
                    }
                    if (*cur_sig).number == 1 {
                        callback_opaque.state.ports.push((
                            (*cur_component).__bindgen_anon_1.data.mime,
                            (*cur_component).__bindgen_anon_1.data.port,
                        ));
                    }
                    cur_component = (*cur_component).next;
                }
            }
            cur_sig = (*cur_sig).next;
        }
    } else if (*event).event == NRSC5_EVENT_SIS {
        let sis: crate::nrsc5::bindings::nrsc5_event_t__bindgen_ty_1__bindgen_ty_11 =
            (*event).__bindgen_anon_1.sis;
        let raw_name = sis.name;
        if !raw_name.is_null() {
            let name = CStr::from_ptr(raw_name).to_string_lossy();
            //println!("{} Station Info", name);

            if callback_opaque.state.station_info.is_none()
                || callback_opaque.state.station_info.as_ref().unwrap().fcc_id
                    != sis.fcc_facility_id
            {
                callback_opaque.state.station_info = Some(StationInfo {
                    name: name.to_string(),
                    country_code: String::new(),
                    fcc_id: sis.fcc_facility_id,
                    slogan: String::new(),
                    message: String::new(),
                    alert: String::new(),
                    location: (sis.latitude, sis.longitude),
                    altitude: sis.altitude,
                    audio_services: vec![],
                });
            }

            let raw_country = sis.country_code;
            if !raw_country.is_null() {
                let country = CStr::from_ptr(raw_country).to_string_lossy();
                callback_opaque
                    .state
                    .station_info
                    .as_mut()
                    .unwrap()
                    .country_code = country.to_string();
                // println!(
                //     "  Country: {} - FCC Facility ID: {}",
                //     country, sis.fcc_facility_id
                // );
            }

            let raw_slogan = sis.slogan;
            if !raw_slogan.is_null() {
                let slogan = CStr::from_ptr(raw_slogan).to_string_lossy();
                callback_opaque.state.station_info.as_mut().unwrap().slogan = slogan.to_string();
                //println!("  Slogan: {}", slogan);
            }

            let raw_message = sis.message;
            if !raw_message.is_null() {
                let message = CStr::from_ptr(raw_message).to_string_lossy();
                callback_opaque.state.station_info.as_mut().unwrap().message = message.to_string();
                //println!("  Message: {}", message);
            }

            let raw_alert = sis.alert;
            if !raw_alert.is_null() {
                let alert = CStr::from_ptr(raw_alert).to_string_lossy();
                callback_opaque.state.station_info.as_mut().unwrap().alert = alert.to_string();
                //println!("  Alert: {}", alert);
            }

            // println!(
            //     "  Location: {}, {} - Altitude: {}m",
            //     sis.latitude, sis.longitude, sis.altitude
            // );
            callback_opaque
                .state
                .station_info
                .as_mut()
                .unwrap()
                .location = (sis.latitude, sis.longitude);
            callback_opaque
                .state
                .station_info
                .as_mut()
                .unwrap()
                .altitude = sis.altitude;

            //println!("  Audio Services:");
            let mut cur_aud = sis.audio_services;
            while !cur_aud.is_null() {
                let mut service_type_name: *const c_char = ptr::null();
                let service_type_name_ptr: *mut *const c_char = &mut service_type_name;
                nrsc5_program_type_name((*cur_aud).type_, service_type_name_ptr);
                if !service_type_name.is_null() {
                    let service_type = CStr::from_ptr(service_type_name).to_string_lossy();
                    let audio_service = AudioService {
                        program: (*cur_aud).program,
                        service_type: service_type.to_string(),
                        is_restricted: (*cur_aud).access != NRSC5_ACCESS_PUBLIC,
                        sound_experience: if (*cur_aud).sound_exp == 2 {
                            "Dolby Pro Logic II Surround"
                        } else {
                            "Stereo"
                        }
                        .to_string(),
                    };
                    if callback_opaque
                        .state
                        .station_info
                        .as_ref()
                        .unwrap()
                        .audio_services
                        .iter()
                        .find(|val| val == &&audio_service)
                        .is_none()
                    {
                        callback_opaque
                            .state
                            .station_info
                            .as_mut()
                            .unwrap()
                            .audio_services
                            .push(audio_service);
                    }

                    // println!(
                    //     "      {}. {} ({}) w/ {}",
                    //     (*cur_aud).program,
                    //     service_type,
                    //     if (*cur_aud).access == NRSC5_ACCESS_PUBLIC {
                    //         "public"
                    //     } else {
                    //         "restricted"
                    //     },
                    //     if (*cur_aud).sound_exp == 2 {
                    //         "Dolby Pro Logic II Surround"
                    //     } else {
                    //         "stereo"
                    //     }
                    // );
                }
                cur_aud = (*cur_aud).next;
            }
        }
    } else if (*event).event == NRSC5_EVENT_HDC
        && (*event).__bindgen_anon_1.hdc.program == callback_opaque.state.program
    {
        callback_opaque
            .state
            .increase_audio_bytes((*event).__bindgen_anon_1.hdc.count as u32);
    }

    if orig_state != callback_opaque.state || lots_updated {
        if (callback_opaque.state.lot_id != orig_state.lot_id
            || callback_opaque.state.ports != orig_state.ports
            || lots_updated)
            && callback_opaque.state.ports.len() > 0
        {
            let mut updated = false;

            if callback_opaque.state.lot_id != -1 {
                let lot_file = get_lot_file(
                    callback_opaque.state.station_info.as_ref().unwrap().fcc_id,
                    callback_opaque.state.lot_id,
                    callback_opaque
                        .state
                        .ports
                        .iter()
                        .find(|val| val.0 == NRSC5_MIME_PRIMARY_IMAGE)
                        .unwrap()
                        .1 as i32,
                );

                if lot_file.is_some() {
                    callback_opaque.state.thumbnail_data =
                        Some(lot_to_base64_url(lot_file.unwrap()));
                    updated = true;
                } else {
                    callback_opaque.state.thumbnail_data = None;
                }
            }

            if !updated {
                let station_logo_port = callback_opaque
                    .state
                    .ports
                    .iter()
                    .find(|val| val.0 == NRSC5_MIME_STATION_LOGO)
                    .unwrap()
                    .1;

                let station_logo = get_lot_file(
                    callback_opaque.state.station_info.as_ref().unwrap().fcc_id,
                    -1,
                    station_logo_port as i32,
                );

                if station_logo.is_some() {
                    callback_opaque.state.thumbnail_data =
                        Some(lot_to_base64_url(station_logo.unwrap()));
                } else {
                    callback_opaque.state.thumbnail_data = None;
                }
            }
        }

        (callback_opaque.callback)(callback_opaque.state.clone());
    }
}

impl_block_trait! { <Flt> Consumer<Signal<Complex<Flt>>> for HdRadioDecode<Flt> }
impl_block_trait! { <Flt> Producer<Signal<Complex<Flt>>> for HdRadioDecode<Flt> }

impl<Flt> HdRadioDecode<Flt>
where
    Flt: Float + Into<f64>,
{
    pub fn new(
        program: u32,
        pass_along: bool,
        hdradio_callback: impl Fn(HdRadioState) + Send + Sync + 'static,
    ) -> Self {
        let (mut receiver, receiver_connector) = new_receiver::<Signal<Complex<Flt>>>();
        let (sender, sender_connector) = new_sender::<Signal<Complex<Flt>>>();

        let (program_send, mut program_recv) = watch::channel(program);
        let (should_reset_send, mut should_reset_recv) = watch::channel(false);

        let mut buf_pool = ChunkBufPool::<Complex<Flt>>::new();

        spawn(async move {
            let mut nrsc5_opaque = Box::new(Nrsc5CallbackOpaque {
                state: HdRadioState::new(program),
                callback: Arc::new(hdradio_callback),
                audio_samples: Arc::new(Mutex::new(vec![])),
            });
            let mut nrsc5_decoder = Nrsc5::new(
                Some(nrsc5_custom_callback),
                &mut *nrsc5_opaque as *mut _ as *mut c_void,
            );

            loop {
                let Ok(signal) = receiver.recv().await else {
                    return;
                };
                match signal {
                    Signal::Samples {
                        sample_rate,
                        chunk: input_chunk,
                    } => {
                        if program_recv.has_changed().unwrap_or(false) {
                            nrsc5_opaque.state.program = program_recv.borrow_and_update().clone();
                            nrsc5_opaque.get_audio_samples();
                        }
                        if should_reset_recv.has_changed().unwrap_or(false) {
                            should_reset_recv.mark_unchanged();
                            let should_reset = should_reset_recv.borrow_and_update();
                            if should_reset.clone() == true {
                                nrsc5_decoder.reset_state();
                                nrsc5_opaque.state = HdRadioState::new(program);
                                (nrsc5_opaque.callback)(nrsc5_opaque.state.clone());
                            }
                        }

                        nrsc5_decoder.pipe_samples(
                            Self::convert_complex_to_iq_samples(
                                input_chunk.iter().map(|value| value.clone()),
                            )
                            .as_slice(),
                        );

                        let new_audio_samples: Vec<i16> = nrsc5_opaque.get_audio_samples();

                        if new_audio_samples.len() == 0 {
                            continue;
                        }

                        println!("{} Audio samples are being sent", new_audio_samples.len());

                        let mut output_chunk = buf_pool.get();

                        for sample in new_audio_samples.iter() {
                            output_chunk.push(Complex {
                                re: Flt::from((*sample) as f32 / i16::MAX as f32).unwrap(),
                                im: Flt::from(0.0).unwrap(),
                            });
                        }

                        let Ok(()) = sender
                            .send(Signal::Samples {
                                sample_rate: NRSC5_SAMPLE_RATE_AUDIO as f64,
                                chunk: output_chunk.finalize(),
                            })
                            .await
                        else {
                            println!("Receiver is no longer available");
                            return;
                        };
                        // if pass_along {
                        //     let Ok(()) = sender
                        //         .send(Signal::Samples {
                        //             sample_rate,
                        //             chunk: input_chunk,
                        //         })
                        //         .await
                        //     else {
                        //         return;
                        //     };
                        // }
                    }
                    Signal::Event(event) => {
                        if pass_along {
                            let Ok(()) = sender.send(Signal::Event(event)).await else {
                                return;
                            };
                        }
                    }
                }
            }
        });
        Self {
            receiver_connector,
            sender_connector,
            program: program_send,
            should_reset: should_reset_send,
        }
    }

    fn convert_complex_to_iq_samples<'a>(iter: impl Iterator<Item = Complex<Flt>>) -> Vec<i16> {
        let mut iq_samples = Vec::new();

        for sample in iter {
            let i = sample.re; // Real part (I)
            let q = sample.im; // Imaginary part (Q)

            // Convert to i16 and push to the output vector (assuming we want to keep the precision)
            iq_samples.push((i.to_f32().unwrap() * 32767.0) as i16);
            iq_samples.push((q.to_f32().unwrap() * 32767.0) as i16);
        }

        iq_samples
    }

    /// Get current program
    pub fn get(&self) -> u32 {
        self.program.borrow().clone()
    }
    /// Set current program
    pub fn set(&self, gain: u32) {
        self.program.send_replace(gain);
    }
    /// reset state
    pub fn reset_state(&self) {
        self.should_reset.send_replace(true);
    }
}
