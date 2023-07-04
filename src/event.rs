use std::collections::HashMap;


use super::error::EventError;
use super::pad_map::PadMap;
use super::graw_frame::{GrawFrame};
use super::constants::*;

fn subtract_trace_baseline(trace: &mut Vec<i16>) {
    let sum: i16 = trace.iter().sum();
    let mut n_nonzero: i16 = 0; 
    for element in trace.iter() {
        if *element != 0 {
            n_nonzero += 1;
        }
    }
    let baseline = sum / n_nonzero;
    for element in trace {
        *element -= baseline;
    }
}

#[derive(Debug)]
pub struct Event<'a> {
    nframes: i32,
    traces: HashMap<u64, Vec<i16>>, //maps pad id to the trace for that pad
    pad_map: &'a PadMap,
    pub timestamp: u64,
    pub event_id: u32
}

impl <'a> Event<'a> {

    pub fn new(map: &'a PadMap) -> Self {
        Event { nframes: 0, traces: HashMap::new(), pad_map: map, timestamp: 0, event_id: 0 }
    }

    pub fn append_frame(&mut self, frame: GrawFrame) -> Result<(), EventError> {

        if self.nframes == 0 {
            self.event_id = frame.header.event_id;
            self.timestamp = frame.header.event_time;
        }
        else if self.event_id != frame.header.event_id {
            return Err(EventError::MismatchedEventID(frame.header.event_id, self.event_id));
        }

        let mut pad_id: u64;
        for datum in frame.data {
            pad_id = match self.pad_map.get_pad_id(&frame.header.cobo_id, &frame.header.asad_id, &datum.aget_id, &datum.channel) {
                Some(pad) => *pad,
                None => {
                    return Err(EventError::InvalidHardware(frame.header.cobo_id, frame.header.asad_id, datum.aget_id, datum.channel));
                }
            };

            match self.traces.get_mut(&pad_id) {
                Some(trace) => trace[datum.time_bucket_id as usize] = datum.sample,
                None => {
                    let mut trace: Vec<i16> = vec![0; NUMBER_OF_TIME_BUCKETS as usize];
                    trace[datum.time_bucket_id as usize] = datum.sample;
                    self.traces.insert(pad_id, trace);
                }
            }
        }

        self.nframes += 1;

        Ok(())
    }

    //Only for use in special cases! This will not throw errors when invalid pads are selected
    fn get_trace_from_hardware_id(&self, cobo_id: &u8, asad_id: &u8, aget_id: &u8, channel_id: &u8) -> Option<&Vec<i16>> {
        let pad_id = match self.pad_map.get_pad_id(cobo_id, asad_id, aget_id, channel_id) {
            Some(pad) => pad,
            None => return None
        };
        return self.traces.get(pad_id);
    }

    pub fn subtract_fixed_pattern_noise(&mut self) {
        const FPN_CHANNELS: [u8; 4] = [11, 22, 45, 56]; //From AGET docs

        for cb_id in 0..NUMBER_OF_COBOS {
            for ad_id in 0..NUMBER_OF_ASADS {
                for ag_id in 0..NUMBER_OF_AGETS {
                    let noise1_trace = match self.get_trace_from_hardware_id(&cb_id, &ad_id, &ag_id, &FPN_CHANNELS[0]) {
                        Some(trace) => trace,
                        None => continue
                    };
                    let noise2_trace = match self.get_trace_from_hardware_id(&cb_id, &ad_id, &ag_id, &FPN_CHANNELS[1]) {
                        Some(trace) => trace,
                        None => continue
                    };
                    let noise3_trace = match self.get_trace_from_hardware_id(&cb_id, &ad_id, &ag_id, &FPN_CHANNELS[2]) {
                        Some(trace) => trace,
                        None => continue
                    };
                    let noise4_trace = match self.get_trace_from_hardware_id(&cb_id, &ad_id, &ag_id, &FPN_CHANNELS[3]) {
                        Some(trace) => trace,
                        None => continue
                    };

                    let mut mean_fpn: Vec<i16> = vec![0; NUMBER_OF_TIME_BUCKETS as usize];

                    for idx in 0..(NUMBER_OF_TIME_BUCKETS as usize) {
                        let sum = noise1_trace[idx] + noise2_trace[idx] + noise3_trace[idx] + noise4_trace[idx];
                        let mut mult: i16 = 0;
                        if noise1_trace[idx] != 0 {
                            mult += 1;
                        }
                        if noise2_trace[idx] != 0 {
                            mult += 1;
                        }
                        if noise3_trace[idx] != 0 {
                            mult += 1;
                        }
                        if noise4_trace[idx] != 0 {
                            mult += 1;
                        }

                        mean_fpn[idx] = sum / mult;
                    }
                }
            }
        }
    }

}