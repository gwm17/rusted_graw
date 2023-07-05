use std::collections::HashMap;
use ndarray::{Array1, Array2, s};

use super::error::EventError;
use super::pad_map::{PadMap, HardwareID};
use super::graw_frame::GrawFrame;
use super::constants::*;

fn subtract_trace_baseline(trace: &mut Array1<i16>) {
    let sum: i16 = trace.iter().sum();
    let mut n_nonzero: i16 = 0; 
    for element in trace.iter() {
        if *element != 0 {
            n_nonzero += 1;
        }
    }
    let baseline = sum / n_nonzero;
    *trace -= baseline;
}




#[derive(Debug)]
pub struct Event {
    nframes: i32,
    traces: HashMap<HardwareID, Array1<i16>>, //maps pad id to the trace for that pad
    pub timestamp: u64,
    pub event_id: u32
}

impl Event {

    pub fn new(pad_map: &PadMap, frames: &Vec<GrawFrame>) -> Result<Self, EventError> {
        let mut event = Event { nframes: 0, traces: HashMap::new(), timestamp: 0, event_id: 0 };

        for frame in frames {
            event.append_frame(pad_map, frame)?;
        }

        event.subtract_fixed_pattern_noise(pad_map);

        Ok(event)
    }

    pub fn convert_to_data_matrix(self) -> Array2<i16> {
        let mut data_matrix = Array2::<i16>::zeros([self.traces.len(), NUMBER_OF_MATRIX_COLUMNS]);
        for (row, (hw_id, trace)) in self.traces.into_iter().enumerate() {
            data_matrix[[row, 0]] = hw_id.cobo_id as i16;
            data_matrix[[row, 1]] = hw_id.asad_id as i16;
            data_matrix[[row, 2]] = hw_id.aget_id as i16;
            data_matrix[[row, 3]] = hw_id.channel as i16;
            data_matrix[[row, 4]] = hw_id.pad_id as i16;
            let mut trace_slice = data_matrix.slice_mut(s![row, 5..NUMBER_OF_MATRIX_COLUMNS]);
            trace.move_into(&mut trace_slice);
        }

        return data_matrix;
    }

    fn append_frame(&mut self, pad_map: &PadMap, frame: &GrawFrame) -> Result<(), EventError> {

        if self.nframes == 0 {
            self.event_id = frame.header.event_id;
            self.timestamp = frame.header.event_time;
        }
        else if self.event_id != frame.header.event_id {
            return Err(EventError::MismatchedEventID(frame.header.event_id, self.event_id));
        }

        let mut hw_id: &HardwareID;
        for datum in frame.data.iter() {
            hw_id = match pad_map.get_hardware_id(&frame.header.cobo_id, &frame.header.asad_id, &datum.aget_id, &datum.channel) {
                Some(hw) => hw,
                None => {
                    return Err(EventError::InvalidHardware(frame.header.cobo_id, frame.header.asad_id, datum.aget_id, datum.channel));
                }
            };

            match self.traces.get_mut(&hw_id) {
                Some(trace) => trace[datum.time_bucket_id as usize] = datum.sample,
                None => {
                    let mut trace: Array1<i16> = Array1::<i16>::zeros(NUMBER_OF_TIME_BUCKETS as usize);
                    trace[datum.time_bucket_id as usize] = datum.sample;
                    self.traces.insert(hw_id.clone(), trace);
                }
            }
        }

        self.nframes += 1;

        Ok(())
    }

    //Only for use in special cases! This will not throw errors when invalid pads are selected
    fn get_trace_from_hardware_id(&self, pad_map: &PadMap, cobo_id: &u8, asad_id: &u8, aget_id: &u8, channel_id: &u8) -> Option<&Array1<i16>> {
        if let Some(hw_id) = pad_map.get_hardware_id(cobo_id, asad_id, aget_id, channel_id) {
            return self.traces.get(hw_id);
        } else {
            return None;
        }
    }

    //Only for use in special cases! This will not throw errors when invalid pads are selected
    fn get_mutable_trace_from_hardware_id(&mut self, pad_map: &PadMap, cobo_id: &u8, asad_id: &u8, aget_id: &u8, channel_id: &u8) -> Option<&mut Array1<i16>> {
        if let Some(hw_id) = pad_map.get_hardware_id(cobo_id, asad_id, aget_id, channel_id) {
            return self.traces.get_mut(hw_id);
        } else {
            return None;
        }
    }

    //Remove a trace, if it exists
    fn remove_trace(&mut self, pad_map: &PadMap, cobo_id: &u8, asad_id: &u8, aget_id: &u8, channel_id: &u8) {
        if let Some(hw_id) = pad_map.get_hardware_id(cobo_id, asad_id, aget_id, channel_id) {
            self.traces.remove(hw_id);
        }
    }

    fn subtract_fixed_pattern_noise(&mut self, pad_map: &PadMap) {
        const FPN_CHANNELS: [u8; 4] = [11, 22, 45, 56]; //From AGET docs

        for cb_id in 0..NUMBER_OF_COBOS {
            for ad_id in 0..NUMBER_OF_ASADS {
                for ag_id in 0..NUMBER_OF_AGETS {
                    let mut mean_fpn: Array1<i16>;
                    {
                        let noise1_trace: &Array1<i16>;
                        let noise2_trace: &Array1<i16>;
                        let noise3_trace: &Array1<i16>;
                        let noise4_trace: &Array1<i16>;
                        if let Some(n1) = self.get_trace_from_hardware_id(pad_map, &cb_id, &ad_id, &ag_id, &FPN_CHANNELS[0]) {
                            noise1_trace = n1;
                        } else {
                            continue;
                        }
                        if let Some(n2) = self.get_trace_from_hardware_id(pad_map, &cb_id, &ad_id, &ag_id, &FPN_CHANNELS[1]) {
                            noise2_trace = n2;
                        } else {
                            continue;
                        }
                        if let Some(n3) = self.get_trace_from_hardware_id(pad_map, &cb_id, &ad_id, &ag_id, &FPN_CHANNELS[2]) {
                            noise3_trace = n3;
                        } else {
                            continue;
                        }
                        if let Some(n4) = self.get_trace_from_hardware_id(pad_map, &cb_id, &ad_id, &ag_id, &FPN_CHANNELS[3]) {
                            noise4_trace = n4;
                        } else {
                            continue;
                        }

                        mean_fpn = noise1_trace + noise2_trace + noise3_trace + noise4_trace;
                        for idx in 0..(NUMBER_OF_TIME_BUCKETS as usize) {
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

                            mean_fpn[idx] /= mult;
                        }
                    }

                    subtract_trace_baseline(&mut mean_fpn); //Correct for baseline of noise traces?

                    //Remove the fixed-pattern noise from the remaining traces (note here we don't skip over fpn channels)
                    for channel in 0..(NUMBER_OF_CHANNELS as usize) {
                        if let Some(trace) = self.get_mutable_trace_from_hardware_id(pad_map, &cb_id, &ad_id, &ag_id, &(channel as u8)) {
                            *trace = trace.iter()
                                .zip(mean_fpn.iter())
                                .map(|(x, y)| x - y)
                                .collect();
                        } else {
                            continue;
                        }
                    }

                    //Drop the fpn traces as we no longer need them
                    for channel in FPN_CHANNELS {
                        self.remove_trace(pad_map, &cb_id, &ad_id, &ag_id, &channel);
                    }
                }
            }
        }
    }

}