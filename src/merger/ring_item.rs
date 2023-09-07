pub struct RingItem {
    pub size: u32,
    pub bytes: Vec<u8>
}

impl RingItem {

    pub fn new() -> RingItem {
        RingItem {size: 0, bytes: vec![]}
    }

    pub fn begin(&self, index: usize, run_info: &mut RunInfo) {
        let mut ind: usize = index;
        self.extract32(&mut ind, &mut run_info.run); // run number
        ind += 4; // skip 4 bytes
        self.extract32(&mut ind, &mut run_info.start); // start time stamp
        ind += 4; // skip 4 bytes
        let mut n = ind;
        let mut str: Vec<u8> = vec![];
        while n < ind+80 && self.bytes[n] != 0 {
            str.push(self.bytes[n]);
            n += 1;
       }
        run_info.title = String::from_utf8(str).unwrap(); // run title
    }

    pub fn end(&self, index: usize, run_info: &mut RunInfo) {
        let mut ind: usize = index+4;
        self.extract32(&mut ind, &mut run_info.seconds); // elasped seconds
        self.extract32(&mut ind, &mut run_info.stop); // stop time stamp
    }

    pub fn dummy(&self) {
        // I am a dummy therefore I do nothing...
    }

    pub fn counter(&self, counter: &mut u32) {
        *counter += 1;
    }

    pub fn scaler(&self, ind: usize, scalers: &mut Scalers) {
        let mut buf: [u8;4] = [0,0,0,0];
        buf.copy_from_slice(&self.bytes[ind..ind+4]);
        scalers.header.push(u32::from_le_bytes(buf)); // start offset
        buf.copy_from_slice(&self.bytes[ind+4..ind+8]);
        scalers.header.push(u32::from_le_bytes(buf)); // stop offset
        buf.copy_from_slice(&self.bytes[ind+8..ind+12]);
        scalers.header.push(u32::from_le_bytes(buf)); // time stamp
        buf.copy_from_slice(&self.bytes[ind+16..ind+20]);
        let count: u32 = u32::from_le_bytes(buf);
        scalers.header.push(count); // count
        buf.copy_from_slice(&self.bytes[ind+20..ind+24]);
        scalers.header.push(u32::from_le_bytes(buf)); // incremental
        for i in 0..count {
            buf.copy_from_slice(&self.bytes[ind+usize::try_from(i).unwrap()+24..ind+usize::try_from(i).unwrap()+28]);
            scalers.data.push(u32::from_le_bytes(buf));
        }
    }

    pub fn remove_boundaries(&mut self, index: usize) {
        let mut wlength: u16;
        let mut buf: [u8;2] = [0,0];
        let mut ind: usize = index;
        while ind < self.bytes.len() {
            buf.copy_from_slice(&self.bytes[ind..ind+2]);
            wlength = u16::from_le_bytes(buf)&0xfff; // buffer length
            self.bytes.remove(ind);
            self.bytes.remove(ind); // 2 bytes to remove
            ind += usize::try_from(wlength*2).unwrap(); // next boundary
        }
    }

    pub fn physics(&self, index: usize, physics: &mut Physics) {
        let mut ind: usize = index;
        let mut event: u32 = 0; // event number
        self.extract32(&mut ind, &mut event);
        physics.header.push(event);
         let mut timestamp: u32 = 0; // time stamp
        self.extract32(&mut ind, &mut timestamp);
        physics.header.push(timestamp);
        let mut tag: u16 = 0;
        while ind < self.bytes.len() {
            self.extract16(&mut ind, &mut tag);
           match tag {
                0x1903 => self.sis3300(&mut ind, physics),
                0x977 => self.v977(&mut ind, physics),
                _ => log::info!("Unrecognized packet tag: {:#x}!", tag)
            }
        }
    }

    pub fn sis3300(&self, ind: &mut usize, physics: &mut Physics) {
        let mut group_enable: u16 = 0;
        self.extract16(ind, &mut group_enable);
        let mut daq_register: u32 = 0;
        self.extract32(ind, &mut daq_register);
        let mut header: u16 = 0;
        let mut group_trigger: u32 = 0;
        let mut samples: u32 = 0;
        let mut pointer: u32;
        let mut trailer: u16 = 0;
        let mut adc1: [u8;2] = [0,0];
        let mut adc2: [u8;2] = [0,0];
        for group in 0..4 {
            if group_enable&(1<<group) == 0 { // skip if group is not enabled
                continue;
            }
            physics.fadc.channels += 2; // channels are read in pairs
            self.extract16(ind, &mut header);
            if header != 0xfadc {
                log::info!("Invalid SIS3300 header: {:#x}!", header);
                break;
            }
            self.extract32(ind, &mut group_trigger);
            self.extract32(ind, &mut samples);
            physics.fadc.samples = usize::try_from(samples).unwrap(); // number of samples
            physics.fadc.traces[group*2] = vec![0;physics.fadc.samples]; // allocate memory
            physics.fadc.traces[group*2+1] = vec![0;physics.fadc.samples]; // allocate memory
            pointer = group_trigger&0x1ffff; // write pointer
            if group_trigger&0x80000 != 0 { // if wrap around bit == 1
                let istart: usize = usize::try_from(pointer+1).unwrap();
                let inc: usize = usize::try_from(samples-pointer-2).unwrap();
                for p in 0..inc+1 {
                    adc1[0] = self.bytes[*ind+(istart+p)*4];
                    adc1[1] = self.bytes[*ind+(istart+p)*4+1];
                    physics.fadc.traces[group*2+1][p] = u16::from_le_bytes(adc1)&0xfff;
                    adc2[0] = self.bytes[*ind+(istart+p)*4+2];
                    adc2[1] = self.bytes[*ind+(istart+p)*4+3];
                    physics.fadc.traces[group*2][p] = u16::from_le_bytes(adc2)&0xfff;
                }
                let istop: usize = usize::try_from(samples).unwrap()-inc-1;
                for p in 0..istop {
                    adc1[0] = self.bytes[*ind+p*4];
                    adc1[1] = self.bytes[*ind+p*4+1];
                    physics.fadc.traces[group*2+1][p+inc+1] = u16::from_le_bytes(adc1)&0xfff;
                    adc2[0] = self.bytes[*ind+p*4+2];
                    adc2[1] = self.bytes[*ind+p*4+3];
                    physics.fadc.traces[group*2][p+inc+1] = u16::from_le_bytes(adc2)&0xfff;
                }
            } else {
                for p in 00..usize::try_from(samples).unwrap() {
                    adc1[0] = self.bytes[*ind+p*4];
                    adc1[1] = self.bytes[*ind+p*4+1];
                    physics.fadc.traces[group*2+1][p] = u16::from_le_bytes(adc1)&0xfff;
                    adc2[0] = self.bytes[*ind+p*4+2];
                    adc2[1] = self.bytes[*ind+p*4+3];
                    physics.fadc.traces[group*2][p] = u16::from_le_bytes(adc2)&0xfff;
                }
            }
            *ind += usize::try_from(samples*4).unwrap();
            self.extract16(ind, &mut trailer);
            if trailer != 0xffff {
                log::info!("Invalid SIS3300 trailer: {:#x}!", trailer);
                break;
            }
        }
    }

    pub fn v977(&self, ind: &mut usize, physics: &mut Physics) {
        self.extract16(ind, &mut physics.coinc.coinc); // coincidence register
    }

    pub fn extract16(&self, ind: &mut usize, var: &mut u16) {
        let mut buf16: [u8;2] = [0,0];
        buf16.copy_from_slice(&self.bytes[*ind..*ind+2]);
        *var = u16::from_le_bytes(buf16);
        *ind += 2;
    }

    pub fn extract32(&self, ind: &mut usize, var: &mut u32) {
        let mut buf32: [u8;4] = [0,0,0,0];
        buf32.copy_from_slice(&self.bytes[*ind..*ind+4]);
        *var = u32::from_le_bytes(buf32);
        *ind += 4;
    }

}
/// RunInfo contains general information about the run
pub struct RunInfo {
    pub run: u32,
    pub start: u32,
    pub stop: u32,
    pub seconds: u32,
    pub title: String,
}

impl RunInfo {
    pub fn new() -> RunInfo {
        RunInfo {run: 0, start: 0, stop: 0, seconds: 0, title: "".to_string()}
    }
}

/// Scalers are composed of a header containing the timing of the scaler data
/// and a data vector that contains the scalers themselves (32 bits)
pub struct Scalers {
    pub header: Vec<u32>,
    pub data: Vec<u32>,
}

impl Scalers {
    pub fn new() -> Scalers {
        Scalers {header: vec![], data: vec![]}
    }
}

/// Physics contains the various modules read by the VMEUSB controller stack
/// For now this an ad hoc list that only contains the modules present in the readout
pub struct Physics {
    pub header: Vec<u32>,
    pub fadc: SIS3300,
    pub coinc: V977,
}

impl Physics {
    pub fn new() -> Physics {
        Physics {header: vec![], fadc: SIS3300::new(), coinc: V977::new()}
    }
}

// Struck module SIS3300: 8 channel flash ADC (12 bits)
pub struct SIS3300 {
    pub traces: Vec<Vec<u16>>,
    pub samples: usize,
    pub channels: usize,
}

impl SIS3300 {
    pub fn new() -> SIS3300 {
        SIS3300 { traces: vec![vec![];8], samples: 0, channels: 0 }
    }
}

/// CAEN module V977: 16 bit coincidence register
pub struct V977 {
    pub coinc: u16,
}

impl V977 {
    pub fn new() -> V977 {
        V977{coinc: 0}
    }
}
