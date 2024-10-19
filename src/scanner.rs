use crate::{
    input::ByteCounter,
    mission::{Mission, Missions},
};
use encoding_rs::Decoder;
use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

pub struct ScannerStates {
    pub v: Vec<Arc<Mutex<ScannerState>>>,
}

impl ScannerStates {
    pub fn new(missions: Missions) -> Self {
        let v = missions.v.iter().fold(vec![], |mut acc, m| {
            acc.push(Arc::new(Mutex::new(ScannerState::new(m.clone()))));
            acc
        });
        Self { v }
    }
}

impl Deref for ScannerStates {
    type Target = Vec<Arc<Mutex<ScannerState>>>;
    fn deref(&self) -> &Self::Target {
        &self.v
    }
}

pub struct ScannerState {
    pub mission: Arc<Mission>,
    pub decoder: Decoder,
    pub last_scan_run_leftover: String,
    pub last_run_str_was_printed_and_is_maybe_cut_str: bool,
    pub consumed_bytes: ByteCounter,
}

impl ScannerState {
    pub fn new(mission: Arc<Mission>) -> Self {
        Self {
            consumed_bytes: mission.counter_offset,
            decoder: mission.encoding.new_decoder_without_bom_handling(),
            last_scan_run_leftover: String::with_capacity(mission.output_line_char_nb_max),
            last_run_str_was_printed_and_is_maybe_cut_str: false,
            mission,
        }
    }
}
