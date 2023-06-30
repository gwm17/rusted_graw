use std::collections::HashMap;

#[derive(Debug)]
pub struct Event {
    nframes: i32,
    data: HashMap<u64, Vec<i16>>,
    pub timestamp: u64,
    pub event_id: u64
}

impl Default for Event {
    fn default() -> Self {
        Event { nframes: 0, data: HashMap::new(), timestamp: 0, event_id: 0 }
    }
}

impl Event {

    pub fn append_frame(&self) {
        todo!()
    }

    pub fn get_trace(&self, pad_id: &u64) -> Option<Vec<i16>> {
        todo!()
    }

    pub fn subtract_fixed_pattern_noise(&mut self) {
        todo!()
    }

    pub fn subtract_pedestal(&mut self) {
        todo!()
    }

    pub fn apply_threshold(&mut self, threshold: i16) {
        todo!()
    }
}