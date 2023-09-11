use std::io::{Cursor, Read};
use byteorder::{ReadBytesExt, LittleEndian};
use super::error::EvtItemError;

const BEGIN_RUN_VAL: u8 = 1;
const END_RUN_VAL: u8 = 2;
const DUMMY_VAL: u8 = 12;
const SCALERS_VAL: u8 = 20;
const PHYSICS_VAL: u8 = 30;
const COUNTER_VAL: u8 = 31;


#[derive(Debug, Clone)]
pub enum RingType {
    BeginRun,
    EndRun,
    Dummy,
    Scalers,
    Physics,
    Counter,
    Invalid,
}

impl From<u8> for RingType {
    fn from(value: u8) -> Self {
        match value {
            BEGIN_RUN_VAL => RingType::BeginRun,
            END_RUN_VAL => RingType::EndRun,
            DUMMY_VAL => RingType::Dummy,
            SCALERS_VAL => RingType::Scalers,
            PHYSICS_VAL => RingType::Physics,
            COUNTER_VAL => RingType::Counter,
            _ => RingType::Invalid
        }
    }
}

#[derive(Debug, Clone)]
pub struct RingItem {
    pub size: usize,
    pub bytes: Vec<u8>,
    pub ring_type: RingType
}


impl TryFrom<Vec<u8>> for RingItem {
    type Error = EvtItemError;
    fn try_from(buffer: Vec<u8>) -> Result<Self, Self::Error> {
        let rt_data: u8;
        {
            let type_data = buffer.get(4);
            match type_data {
                Some(data) => rt_data = data.clone(),
                None => return Err(EvtItemError::ItemSizeError)
            };
        }
        let item_data_buffer: Vec<u8>;
        if buffer[8] == 20 && buffer.len() >= 28 { // ring header might or might not be present
            item_data_buffer = buffer[28..].to_vec();
        } else if buffer.len() >= 12 {
            item_data_buffer = buffer[12..].to_vec();
        } else {
            return Err(EvtItemError::ItemSizeError);
        }
        Ok(Self { size: buffer.len(), bytes: item_data_buffer, ring_type: RingType::from(rt_data) })
    }
}

impl RingItem {

    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {size: 0, bytes: vec![], ring_type: RingType::Invalid}
    }

    pub fn remove_boundaries(&mut self) {
        let mut wlength: u16;
        let mut buf: [u8;2] = [0,0];
        let mut ind: usize = 0;
        while ind < self.bytes.len() {
            buf.copy_from_slice(&self.bytes[ind..ind+2]);
            wlength = u16::from_le_bytes(buf)&0xfff; // buffer length
            self.bytes.remove(ind);
            self.bytes.remove(ind); // 2 bytes to remove
            ind += usize::try_from(wlength*2).unwrap(); // next boundary
        }
    }

}

//Below are the various explicit ring item types. RingItems can be cast into these objects using
//try_from semantics.

/// RunInfo contains general information about the run
#[derive(Debug, Clone)]
pub struct BeginRunItem {
    pub run: u32,
    pub start: u32,
    pub title: String,
}

impl TryFrom<RingItem> for BeginRunItem {
    type Error = EvtItemError;
    fn try_from(ring: RingItem) -> Result<Self, EvtItemError>  {
        let mut cursor = Cursor::new(ring.bytes);
        let mut info = BeginRunItem::new();
        info.run = cursor.read_u32::<LittleEndian>()?;
        cursor.set_position(cursor.position() + 4);
        info.start = cursor.read_u32::<LittleEndian>()?;
        cursor.set_position(cursor.position() + 4);
        cursor.read_to_string(&mut info.title)?;
        return Ok(info);
    }
}



impl BeginRunItem {
    pub fn new() -> Self {
        Self {run: 0, start: 0, title: String::new()}
    }
}

#[derive(Debug, Clone)]
pub struct EndRunItem {
    pub stop: u32,
    pub time: u32
}

impl TryFrom<RingItem> for EndRunItem {
    type Error = EvtItemError;
    fn try_from(ring: RingItem) -> Result<Self, Self::Error> {
        let mut cursor = Cursor::new(ring.bytes);
        let mut info = EndRunItem::new();
        
        info.stop = cursor.read_u32::<LittleEndian>()?;
        info.time = cursor.read_u32::<LittleEndian>()?;
        return Ok(info);
    }
}

impl EndRunItem {
    pub fn new() -> Self {
        Self { stop: 0, time: 0 }
    }
}

/// Simple holder for the begin and end run info
#[derive(Debug, Clone)]
pub struct RunInfo
{
    pub begin: BeginRunItem,
    pub end: EndRunItem
}

impl RunInfo {
    pub fn new() -> RunInfo {
        RunInfo { begin: BeginRunItem::new(), end: EndRunItem::new() }
    }
}

impl RunInfo {
    pub fn print_begin(&self) -> String {
        format!("Run Number: {} Title: {}", self.begin.run, self.begin.title)
    }

    pub fn print_end(&self) -> String {
        format!("Run Number: {} Ellapsed Time: {}s", self.begin.run, self.end.time)
    }
}
/// Scalers are composed of a header containing the timing of the scaler data
/// and a data vector that contains the scalers themselves (32 bits)
#[derive(Debug, Clone)]
pub struct Scalers {
    pub start_offset: u32,
    pub stop_offset: u32,
    pub timestamp: u32,
    pub incremental: u32,
    pub data: Vec<u32>
}

impl TryFrom<RingItem> for Scalers {
    type Error = EvtItemError;
    fn try_from(ring: RingItem) -> Result<Self, Self::Error> {
        let mut cursor = Cursor::new(ring.bytes);
        let mut info = Scalers::new();
        info.start_offset = cursor.read_u32::<LittleEndian>()?;
        info.stop_offset = cursor.read_u32::<LittleEndian>()?;
        info.timestamp = cursor.read_u32::<LittleEndian>()?;
        let count = cursor.read_u32::<LittleEndian>()?;
        info.incremental = cursor.read_u32::<LittleEndian>()?;
        info.data.resize(count as usize, 0);
        for value in info.data.iter_mut() {
            *value = cursor.read_u32::<LittleEndian>()?;
        }

        return Ok(info);
    }
}

impl Scalers {
    pub fn new() -> Scalers {
        Scalers {start_offset: 0, stop_offset: 0, timestamp: 0, incremental: 0, data: vec![]}
    }

    pub fn get_header_array(&self) -> Vec<u32> {
        vec![self.start_offset, self.stop_offset, self.timestamp, self.data.len() as u32, self.incremental]
    }
}

#[derive(Debug, Clone)]
pub struct CounterItem {
    pub count: u64
}

impl TryFrom<RingItem> for CounterItem {
    type Error = EvtItemError;
    fn try_from(ring: RingItem) -> Result<Self, Self::Error> {
        let mut cursor = Cursor::new(ring.bytes);
        let mut info = Self::new();
        cursor.set_position(12);
        info.count = cursor.read_u64::<LittleEndian>()?;
        return Ok(info);
    }
}

impl CounterItem {
    pub fn new() -> Self {
        return Self{ count: 0 };
    }
}

/// Physics contains the various modules read by the VMEUSB controller stack
/// For now this an ad hoc list that only contains the modules present in the readout
#[derive(Debug, Clone)]
pub struct Physics {
    pub event: u32,
    pub timestamp: u32,
    pub fadc: SIS3300,
    pub coinc: V977,
}

impl TryFrom<RingItem> for Physics {
    type Error = EvtItemError;
    fn try_from(ring: RingItem) -> Result<Self, Self::Error> {
        let _end_position = ring.bytes.len() as u64;
        let mut cursor = Cursor::new(ring.bytes);
        let mut info = Physics::new();
        info.event = cursor.read_u32::<LittleEndian>()?;
        info.timestamp = cursor.read_u32::<LittleEndian>()?;
        if cursor.read_u16::<LittleEndian>()? != 0x1903 {
            return Err(EvtItemError::StackOrderError);
        }
        info.fadc.extract_data(&mut cursor)?;
        if cursor.read_u16::<LittleEndian>()?  != 0x977 {
            return Err(EvtItemError::StackOrderError);
        }
        info.coinc.extract_data(&mut cursor)?;

        return Ok(info);
    }
}

impl Physics {
    pub fn new() -> Physics {
        Physics {event: 0, timestamp: 0, fadc: SIS3300::new(), coinc: V977::new()}
    }

    pub fn get_header_array(&self) -> Vec<u32> {
        return vec![self.event, self.timestamp];
    }
}

// Struck module SIS3300: 8 channel flash ADC (12 bits)
#[derive(Debug, Clone)]
pub struct SIS3300 {
    pub traces: Vec<Vec<u16>>,
    pub samples: usize,
    pub channels: usize,
}

impl SIS3300 {
    pub fn new() -> SIS3300 {
        SIS3300 { traces: vec![vec![];8], samples: 0, channels: 0 }
    }

    pub fn extract_data(&mut self, cursor: &mut std::io::Cursor<Vec<u8>>) -> Result<(), EvtItemError> {
        let group_enable_flags = cursor.read_u16::<LittleEndian>()?;
        let _daq_register = cursor.read_u32::<LittleEndian>()?;

        //Some data
        let mut header: u16;
        let mut group_trigger: u32;
        let mut pointer: usize;
        let mut trailer: u16;


        for group in 0..4 {
            if group_enable_flags&(1<<group) == 0 { // skip if group is not enabled
                continue;
            }
            self.channels += 2; // channels are read in pairs
            header = cursor.read_u16::<LittleEndian>()?;
            if header != 0xfadc {
                log::info!("Invalid SIS3300 header: {:#x}!", header);
                break;
            }
            group_trigger = cursor.read_u32::<LittleEndian>()?;
            self.samples = cursor.read_u32::<LittleEndian>()? as usize;
            self.traces[group*2] = vec![0; self.samples];
            self.traces[group*2 + 1] = vec![0; self.samples];
            pointer = (group_trigger&0x1ffff) as usize; // write pointer
            let starting_position = cursor.position();
            if ((group_trigger&0x80000) != 0) && (pointer < self.samples-1) { // if wrap around bit == 1
                let istart: usize = pointer + 1;
                let inc: usize = self.samples - pointer - 2;
                cursor.set_position(starting_position + ((istart * 4) as u64));
                for p in 0..inc+1 {
                    self.traces[group*2+1][p] = cursor.read_u16::<LittleEndian>()? & 0xfff;
                    self.traces[group*2][p] = cursor.read_u16::<LittleEndian>()? & 0xfff;
                }
                let istop: usize = self.samples - inc - 1;
                cursor.set_position(starting_position);
                for p in 0..istop {
                    self.traces[group*2+1][p+inc+1] = cursor.read_u16::<LittleEndian>()? & 0xfff;
                    self.traces[group*2][p+inc+1] = cursor.read_u16::<LittleEndian>()? & 0xfff;
                }
            } else {
                for p in 0..self.samples {
                    self.traces[group*2+1][p] = cursor.read_u16::<LittleEndian>()? & 0xfff;
                    self.traces[group*2][p] = cursor.read_u16::<LittleEndian>()? & 0xfff;
                }
            }
            cursor.set_position(starting_position + ((self.samples*4) as u64));
            trailer = cursor.read_u16::<LittleEndian>()?;
            if trailer != 0xffff {
                log::info!("Invalid SIS3300 trailer: {:#x}!", trailer);
                break;
            }
        }

        Ok(())
    }
}

/// CAEN module V977: 16 bit coincidence register
#[derive(Debug, Clone)]
pub struct V977 {
    pub coinc: u16,
}

impl V977 {
    pub fn new() -> V977 {
        V977{coinc: 0}
    }

    pub fn extract_data(&mut self, cursor: &mut Cursor<Vec<u8>>) -> Result<(), EvtItemError> {
        self.coinc = cursor.read_u16::<LittleEndian>()?;
        return Ok(());
    }
}
