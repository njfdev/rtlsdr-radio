use std::{
    ffi::{c_char, c_void, CStr},
    fs,
    path::Path,
    ptr::{self, null, null_mut},
    sync::{Arc, Mutex},
};

use crate::{
    modes::*,
    nrsc5::{
        bindings::{
            nrsc5_event_t, nrsc5_program_type_name, nrsc5_service_data_type_name,
            NRSC5_ACCESS_PUBLIC, NRSC5_EVENT_AUDIO, NRSC5_EVENT_BER, NRSC5_EVENT_ID3,
            NRSC5_EVENT_LOST_SYNC, NRSC5_EVENT_LOT, NRSC5_EVENT_MER, NRSC5_EVENT_PACKET,
            NRSC5_EVENT_SIG, NRSC5_EVENT_SIS, NRSC5_EVENT_STREAM, NRSC5_EVENT_SYNC,
            NRSC5_MIME_JPEG, NRSC5_SAMPLE_RATE_AUDIO, NRSC5_SIG_COMPONENT_AUDIO,
            NRSC5_SIG_SERVICE_AUDIO,
        },
        Nrsc5,
    },
};
use radiorust::{
    flow::{new_receiver, new_sender, ReceiverConnector, SenderConnector},
    impl_block_trait,
    numbers::Float,
    prelude::{ChunkBufPool, Complex},
    signal::Signal,
};
use tauri::ipc::Channel;
use tokio::spawn;

pub struct HdRadioDecode<Flt> {
    receiver_connector: ReceiverConnector<Signal<Complex<Flt>>>,
    sender_connector: SenderConnector<Signal<Complex<Flt>>>,
}

static mut AUDIO_SAMPLES: Vec<i16> = vec![];

unsafe extern "C" fn nrsc5_custom_callback(event: *const nrsc5_event_t, opaque: *mut c_void) {
    if (*event).event == NRSC5_EVENT_ID3 && (*event).__bindgen_anon_1.id3.program == 0 {
        let raw_title = (*event).__bindgen_anon_1.id3.title;
        if !raw_title.is_null() {
            let title = CStr::from_ptr(raw_title).to_string_lossy();
            println!("Title: {}", title);
        }
        let raw_artist = (*event).__bindgen_anon_1.id3.artist;
        if !raw_artist.is_null() {
            let artist = CStr::from_ptr(raw_artist).to_string_lossy();
            println!("Artist: {}", artist);
        }
        let raw_album = (*event).__bindgen_anon_1.id3.album;
        if !raw_album.is_null() {
            let album = CStr::from_ptr(raw_album).to_string_lossy();
            println!("Album: {}", album);
        }
        let raw_genre = (*event).__bindgen_anon_1.id3.genre;
        if !raw_genre.is_null() {
            let genre = CStr::from_ptr(raw_genre).to_string_lossy();
            println!("Genre: {}", genre);
        }
        let lot_id = (*event).__bindgen_anon_1.id3.xhdr.lot;
        println!("LOT ID: {}", lot_id);
    } else if (*event).event == NRSC5_EVENT_AUDIO && (*event).__bindgen_anon_1.audio.program == 0 {
        let data_ptr = (*event).__bindgen_anon_1.audio.data;
        let data_len = (*event).__bindgen_anon_1.audio.count as usize;
        // Safety: We assume that the data pointer is valid and has the correct length.
        let audio_data = std::slice::from_raw_parts(data_ptr, data_len);

        // update AUDIO_SAMPLES
        AUDIO_SAMPLES.extend_from_slice(audio_data);
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
        fs::write(
            image_path,
            std::slice::from_raw_parts(
                (*event).__bindgen_anon_1.lot.data,
                (*event).__bindgen_anon_1.lot.size as usize,
            ),
        );
        println!(
            "  LOT: {}, MIME: {:#x}, Port: {}, Size: {}",
            (*event).__bindgen_anon_1.lot.lot,
            (*event).__bindgen_anon_1.lot.mime,
            (*event).__bindgen_anon_1.lot.port,
            (*event).__bindgen_anon_1.lot.size
        );
    } else if (*event).event == NRSC5_EVENT_SYNC {
        println!("Synced to Station");
    } else if (*event).event == NRSC5_EVENT_LOST_SYNC {
        println!("Lost Sync to Station");
    } else if (*event).event == NRSC5_EVENT_BER {
        println!(
            "Bit Error Ratio: {}%",
            (*event).__bindgen_anon_1.ber.cber * 100.0
        );
    } else if (*event).event == NRSC5_EVENT_MER {
        println!(
            "Modulation Error Ratio: Lower {}, Upper {}",
            (*event).__bindgen_anon_1.mer.lower,
            (*event).__bindgen_anon_1.mer.upper
        );
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
                            "      Audio Component (MIME {:#x})",
                            (*cur_component).__bindgen_anon_1.audio.mime
                        );
                    } else {
                        println!(
                            "      Data Component (MIME {:#x})",
                            (*cur_component).__bindgen_anon_1.data.mime
                        );
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
            println!("{} Station Info", name);

            let raw_country = sis.country_code;
            if !raw_country.is_null() {
                let country = CStr::from_ptr(raw_country).to_string_lossy();
                println!(
                    "  Country: {} - FCC Facility ID: {}",
                    country, sis.fcc_facility_id
                );
            }

            let raw_slogan = sis.slogan;
            if !raw_slogan.is_null() {
                let slogan = CStr::from_ptr(raw_slogan).to_string_lossy();
                println!("  Slogan: {}", slogan);
            }

            let raw_message = sis.message;
            if !raw_message.is_null() {
                let message = CStr::from_ptr(raw_message).to_string_lossy();
                println!("  Message: {}", message);
            }

            let raw_alert = sis.alert;
            if !raw_alert.is_null() {
                let alert = CStr::from_ptr(raw_alert).to_string_lossy();
                println!("  Alert: {}", alert);
            }

            println!(
                "  Location: {}, {} - Altitude: {}ft",
                sis.latitude, sis.longitude, sis.altitude
            );

            println!("  Audio Services:");
            let mut cur_aud = sis.audio_services;
            while !cur_aud.is_null() {
                let mut service_type_name: *const c_char = ptr::null();
                let service_type_name_ptr: *mut *const c_char = &mut service_type_name;
                nrsc5_program_type_name((*cur_aud).type_, service_type_name_ptr);
                if !service_type_name.is_null() {
                    let service_type = CStr::from_ptr(service_type_name).to_string_lossy();
                    println!(
                        "      {}. {} ({}) w/ {}",
                        (*cur_aud).program,
                        service_type,
                        if (*cur_aud).access == NRSC5_ACCESS_PUBLIC {
                            "public"
                        } else {
                            "restricted"
                        },
                        if (*cur_aud).sound_exp == 2 {
                            "Dolby Pro Logic II Surround"
                        } else {
                            "stereo"
                        }
                    );
                }
                cur_aud = (*cur_aud).next;
            }
        }
    }
}

impl_block_trait! { <Flt> Consumer<Signal<Complex<Flt>>> for HdRadioDecode<Flt> }
impl_block_trait! { <Flt> Producer<Signal<Complex<Flt>>> for HdRadioDecode<Flt> }

impl<Flt> HdRadioDecode<Flt>
where
    Flt: Float + Into<f64>,
{
    pub fn new(pass_along: bool) -> Self {
        let (mut receiver, receiver_connector) = new_receiver::<Signal<Complex<Flt>>>();
        let (sender, sender_connector) = new_sender::<Signal<Complex<Flt>>>();

        let mut buf_pool = ChunkBufPool::<Complex<Flt>>::new();

        let nrsc5_decoder = Nrsc5::new(Some(nrsc5_custom_callback), ptr::null_mut());

        spawn(async move {
            loop {
                let Ok(signal) = receiver.recv().await else {
                    return;
                };
                match signal {
                    Signal::Samples {
                        sample_rate,
                        chunk: input_chunk,
                    } => {
                        nrsc5_decoder.pipe_samples(
                            Self::convert_complex_to_iq_samples(
                                input_chunk.iter().map(|value| value.clone()),
                            )
                            .as_slice(),
                        );

                        let mut new_audio_samples: Vec<i16> = vec![];

                        unsafe {
                            if AUDIO_SAMPLES.len() == 0 {
                                continue;
                            }

                            println!("Audio Samples length: {}", AUDIO_SAMPLES.len());

                            new_audio_samples = AUDIO_SAMPLES.clone();
                            AUDIO_SAMPLES.clear();
                        }

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
}
