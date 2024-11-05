// conversions of nrsc5.h to Rust (using ChatGPT)

use std::ffi::c_void;
use std::mem;
use std::os::raw::{c_char, c_int, c_uint, c_ushort};
use std::ptr;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Nrsc5Mode {
    Fm = 0,
    Am = 1,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Nrsc5SigComponentType {
    Audio = 0,
    Data = 1,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Nrsc5SigServiceType {
    Audio = 0,
    Data = 1,
}

pub const NRSC5_SCAN_BEGIN: f64 = 87.9e6;
pub const NRSC5_SCAN_END: f64 = 107.9e6;
pub const NRSC5_SCAN_SKIP: f64 = 0.2e6;

pub const NRSC5_AUDIO_FRAME_SAMPLES: usize = 2048;
pub const NRSC5_SAMPLE_RATE_CU8: f64 = 1488375.0;
pub const NRSC5_SAMPLE_RATE_CS16_FM: f64 = 744187.5;
pub const NRSC5_SAMPLE_RATE_CS16_AM: f64 = 46511.71875;
pub const NRSC5_SAMPLE_RATE_AUDIO: u32 = 44100;

#[repr(C)]
pub struct Nrsc5SigComponent {
    pub next: *mut Nrsc5SigComponent, // Linked list
    pub component_type: u8,           // Audio or Data
    pub id: u8,
    pub data_audio: Nrsc5DataAudio, // Union of data and audio components
}

#[repr(C)]
pub union Nrsc5DataAudio {
    pub data: Nrsc5DataComponent,
    pub audio: Nrsc5AudioComponent,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Nrsc5DataComponent {
    pub port: u16,
    pub service_data_type: u16,
    pub data_type: u8,
    pub mime: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Nrsc5AudioComponent {
    pub port: u8,
    pub audio_type: u8,
    pub mime: u32,
}

#[repr(C)]
pub struct Nrsc5SigService {
    pub next: *mut Nrsc5SigService,
    pub service_type: u8,
    pub number: u16,
    pub name: *const c_char,
    pub components: *mut Nrsc5SigComponent,
}

#[repr(C)]
pub struct Nrsc5SisAsd {
    pub next: *mut Nrsc5SisAsd,
    pub program: u32,
    pub access: u32,
    pub asd_type: u32,
    pub sound_exp: u32,
}

#[repr(C)]
pub struct Nrsc5SisDsd {
    pub next: *mut Nrsc5SisDsd,
    pub access: u32,
    pub dsd_type: u32,
    pub mime_type: u32,
}

#[repr(C)]
pub struct Nrsc5Event {
    event: c_uint,
    data: EventData,
}

#[repr(C)]
pub union EventData {
    iq: IQData,
    ber: BERData,
    mer: MERData,
    hdc: HDCData,
    audio: AudioData,
    id3: ID3Data,
    stream: StreamData,
    packet: PacketData,
    lot: LOTData,
    sig: SigData,
    sis: SISData,
}

// Define each union member's struct
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct IQData {
    data: *const *mut c_void,
    count: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BERData {
    cber: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MERData {
    lower: f32,
    upper: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HDCData {
    program: c_uint,
    data: *const u8,
    count: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AudioData {
    program: c_uint,
    data: *const i16,
    count: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ID3Data {
    program: c_uint,
    title: *const c_char,
    artist: *const c_char,
    album: *const c_char,
    genre: *const c_char,
    ufid: UFIDData,
    xhdr: XHDRData,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UFIDData {
    owner: *const c_char,
    id: *const c_char,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct XHDRData {
    mime: u32,
    param: c_int,
    lot: c_int,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct StreamData {
    port: c_ushort,
    seq: c_ushort,
    size: c_uint,
    mime: u32,
    data: *const u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PacketData {
    port: c_ushort,
    seq: c_ushort,
    size: c_uint,
    mime: u32,
    data: *const u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LOTData {
    port: c_ushort,
    lot: c_uint,
    size: c_uint,
    mime: u32,
    name: *const c_char,
    data: *const u8,
    expiry_utc: *mut libc::tm, // Assuming tm is defined in libc
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SigData {
    services: *mut Nrsc5SigService, // Replace with actual type
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SISData {
    country_code: *const c_char,
    fcc_facility_id: c_int,
    name: *const c_char,
    slogan: *const c_char,
    message: *const c_char,
    alert: *const c_char,
    latitude: f32,
    longitude: f32,
    altitude: c_int,
    audio_services: *mut Nrsc5SISASD, // Replace with actual type
    data_services: *mut Nrsc5SISDSD,  // Replace with actual type
}

// Define the callback function type
pub type Nrsc5Callback = extern "C" fn(evt: *const Nrsc5Event, opaque: *mut *mut c_void);

// Define types for the services if needed
pub struct Nrsc5SISASD; // Define based on actual structure
pub struct Nrsc5SISDSD; // Define based on actual structure

extern "C" {
    pub fn nrsc5_get_version(version: *mut *const c_char);

    pub fn nrsc5_service_data_type_name(type_: u32, name: *mut *const c_char);

    pub fn nrsc5_program_type_name(type_: u32, name: *mut *const c_char);

    pub fn nrsc5_open_pipe(st: *mut *mut c_void) -> c_int;

    pub fn nrsc5_close(st: *mut c_void);

    pub fn nrsc5_start(st: *mut c_void);

    pub fn nrsc5_stop(st: *mut c_void);

    pub fn nrsc5_set_mode(st: *mut c_void, mode: c_int) -> c_int;

    pub fn nrsc5_set_callback(st: *mut c_void, callback: Nrsc5Callback, opaque: *mut *mut c_void);

    pub fn nrsc5_pipe_samples_cu8(st: *mut c_void, samples: *const u8, length: c_uint) -> c_int;

    pub fn nrsc5_pipe_samples_cs16(st: *mut c_void, samples: *const i16, length: c_uint) -> c_int;
}
