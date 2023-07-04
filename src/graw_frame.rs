
use nom::number::complete::*;
use nom::bytes::complete::*;
use bitvec::prelude::*;

use super::constants::*;
use super::error::{GrawFrameError, GrawDataError};

//Data from a single time-bucket (sampled point along the waveform)
#[derive(Debug, Clone)]
pub struct GrawData {
    pub aget_id: u8,
    pub channel: u8,
    pub time_bucket_id: u16,
    pub sample: i16
}

impl Default for GrawData {
    fn default() -> Self {
        GrawData { aget_id: 0, channel: 0, time_bucket_id: 0, sample: 0 }
    }
}

impl GrawData {
    pub fn check_data(&self) -> Result<(), GrawDataError> {
        if self.aget_id > NUMBER_OF_AGETS {
            return Err(GrawDataError::BadAgetID(self.aget_id));
        }
        if self.channel > NUMBER_OF_CHANNELS {
            return Err(GrawDataError::BadChannel(self.channel));
        }
        if (self.time_bucket_id as u32) > NUMBER_OF_TIME_BUCKETS {
            return Err(GrawDataError::BadTimeBucket(self.time_bucket_id));
        }

        Ok(())
    }
}

/*x
    Little parsing functions to handle big-endian 
 */

fn parse_u8(buffer: &[u8]) -> Result<(&[u8], u8), GrawFrameError> {
    match be_u8::<&[u8], nom::error::Error<&[u8]>>(buffer) {
        Ok(b) => Ok(b),
        Err(_) => Err(GrawFrameError::ParsingError)
    }
}

fn parse_u16(buffer: &[u8]) -> Result<(&[u8], u16), GrawFrameError> {
    match be_u16::<&[u8], nom::error::Error<&[u8]>>(buffer) {
        Ok(b) => Ok(b),
        Err(_) => Err(GrawFrameError::ParsingError)
    }
}

fn parse_u24_to_u32(buffer: &[u8]) -> Result<(&[u8], u32), GrawFrameError> {
    match be_u24::<&[u8], nom::error::Error<&[u8]>>(buffer) {
        Ok(b) => Ok(b),
        Err(_) => Err(GrawFrameError::ParsingError)
    }
}

fn parse_u32(buffer: &[u8]) -> Result<(&[u8], u32), GrawFrameError> {
    match be_u32::<&[u8], nom::error::Error<&[u8]>>(buffer) {
        Ok(b) => Ok(b),
        Err(_) => Err(GrawFrameError::ParsingError)
    }
}

//48 bit words suck
fn parse_u48_to_u64(buffer: &[u8]) -> Result<(&[u8], u64), GrawFrameError> {
    match take::<usize, &[u8], nom::error::Error<&[u8]>>(4usize)(buffer) {
        Ok((buf_slice, word)) => {
            let mut full_word: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
            for i in 0..4 {
                full_word[i] = word[i]
            }
            return Ok((buf_slice, u64::from_be_bytes(full_word)));
        }
        Err(_) => Err(GrawFrameError::ParsingError)
    }
}

fn parse_bitsets(buffer: &[u8]) -> Result<(&[u8], Vec<BitVec<u8>>), GrawFrameError> {

    let mut sets: Vec<BitVec<u8>> = Vec::with_capacity(4);
    let mut storage_index: usize;
    let mut buf_slice: &[u8] = buffer;
    let mut byte: u8;
    for _ in 0..4 {
        let mut aget_bits = bitvec![u8, Lsb0; 0; SIZE_OF_BITSET];
        for index in (0..9).rev() {
            storage_index = 8 - index;
            (buf_slice, byte) = parse_u8(buf_slice)?;
            aget_bits[storage_index..=(storage_index+8)].store(byte);
        }
        sets.push(aget_bits);
    }

    return Ok((buf_slice, sets));
}

fn parse_multiplicity(buffer: &[u8]) -> Result<(&[u8], Vec<u16>), GrawFrameError> {
    let mut mults: Vec<u16> = Vec::with_capacity(4);
    let mut mult: u16;
    let mut buf_slice: &[u8] = buffer;
    for _ in 0..4 {
        (buf_slice, mult) = parse_u16(buf_slice)?;
        mults.push(mult);
    }

    return Ok((buf_slice, mults));
}

/*
    FrameMetadata provides the graw file a way of querying the event (hardware-level)
    information without accessing the entire frame
 */
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FrameMetadata {
    pub event_id: u32,
    pub event_time: u64
}

impl From<GrawFrameHeader> for FrameMetadata {
    fn from(value: GrawFrameHeader) -> Self {
        FrameMetadata { event_id: value.event_id, event_time: value.event_time }
    }
}

/*
    A GrawFrame is the representation of a single readout of an AsAd. Each readout contains
    traces from the four AGET ASICS on the AsAd.
    GrawFrames are composed with a GrawFrameHeader which contains metadata about the frame,
    and an array of GrawData (the samples of the waveform)
 */
#[derive(Debug, Clone, Default)]
pub struct GrawFrameHeader {
    pub meta_type: u8, //set to 0x6 ?
    pub frame_size: u32, //in 32-bit words
    pub data_source: u8,
    pub frame_type: u16,
    pub revision: u8,
    pub header_size: u16,
    pub item_size: u16,
    pub n_items: u32,
    pub event_time: u64,
    pub event_id: u32,
    pub cobo_id: u8,
    pub asad_id: u8,
    pub read_offset: u16,
    pub status: u8
}

impl GrawFrameHeader {
    pub fn check_header(&self, buffer_length: u32) -> Result<(), GrawFrameError> {
        if self.meta_type != EXPECTED_META_TYPE {
            return Err(GrawFrameError::IncorrectMetaType(self.meta_type));
        }
        if self.frame_size * SIZE_UNIT != buffer_length {
            return Err(GrawFrameError::IncorrectFrameSize(self.frame_size, buffer_length))
        }
        if self.frame_type != EXPECTED_FRAME_TYPE_FULL || self.frame_type != EXPECTED_FRAME_TYPE_PARTIAL {
            return Err(GrawFrameError::IncorrectFrameType(self.frame_type));
        }
        if self.header_size != EXPECTED_HEADER_SIZE {
            return Err(GrawFrameError::IncorrectHeaderSize(self.header_size))
        }
        if (self.frame_type == EXPECTED_FRAME_TYPE_FULL && self.item_size != EXPECTED_ITEM_SIZE_FULL) || 
            (self.frame_type == EXPECTED_FRAME_TYPE_PARTIAL && self.item_size != EXPECTED_ITEM_SIZE_FULL)
        {
            return Err(GrawFrameError::IncorrectItemSize(self.item_size));
        }
        let expected_n_items = (((self.frame_size - self.header_size as u32) as f64) / (SIZE_UNIT as f64)).ceil() as u32;
        if self.n_items != expected_n_items {
            return Err(GrawFrameError::IncorrectNumberOfItems(self.n_items, expected_n_items))
        }
        Ok(())
    }

    pub fn read_from_buffer(buffer: &[u8]) -> Result<(&[u8], GrawFrameHeader), GrawFrameError> {
        let mut buf_slice: &[u8] = buffer;
        let mut header = GrawFrameHeader::default();
        (buf_slice, header.meta_type) = parse_u8(buf_slice)?;
        (buf_slice, header.frame_size) = parse_u24_to_u32(buf_slice)?; //Obnoxious. Actually a 24 bit word
        (buf_slice, header.data_source) = parse_u8(buf_slice)?;
        (buf_slice, header.frame_type) = parse_u16(buf_slice)?;
        (buf_slice, header.revision) = parse_u8(buf_slice)?;
        (buf_slice, header.header_size) = parse_u16(buf_slice)?;
        (buf_slice, header.item_size) = parse_u16(buf_slice)?;
        (buf_slice, header.n_items) = parse_u32(buf_slice)?;
        (buf_slice, header.event_time) = parse_u48_to_u64(buf_slice)?; //Obnoxious. Actually a 48 bit word
        (buf_slice, header.event_id) = parse_u32(buf_slice)?;
        (buf_slice, header.cobo_id) = parse_u8(buf_slice)?;
        (buf_slice, header.asad_id) = parse_u8(buf_slice)?;
        (buf_slice, header.read_offset) = parse_u16(buf_slice)?;
        (buf_slice, header.status) = parse_u8(buf_slice)?;

        Ok((buf_slice, header))
    }
}

#[derive(Debug)]
pub struct GrawFrame {
    pub header: GrawFrameHeader,
    hit_patterns: Vec<BitVec<u8>>,
    multiplicity: Vec<u16>,
    pub data: Vec<GrawData>
}

impl TryFrom<Vec<u8>> for GrawFrame {
    type Error = GrawFrameError;

    fn try_from(buffer: Vec<u8>) -> Result<Self , Self::Error>{
        let mut buf_slice = buffer.as_slice();

        let mut frame = GrawFrame::new();
        
        (buf_slice, frame.header) = GrawFrameHeader::read_from_buffer(buf_slice)?;
        frame.header.check_header(buffer.len() as u32)?;
        (buf_slice, frame.hit_patterns) = parse_bitsets(buf_slice)?;
        (buf_slice, frame.multiplicity) = parse_multiplicity(buf_slice)?;

        if frame.header.frame_type == EXPECTED_FRAME_TYPE_PARTIAL {
            frame.extract_partial_data(buf_slice)?;
        }
        else if frame.header.frame_type == EXPECTED_FRAME_TYPE_FULL {
            frame.extract_full_data(buf_slice)?;
        }

        Ok(frame)
    }
}

impl GrawFrame {

    pub fn new() -> GrawFrame {
        GrawFrame { header: GrawFrameHeader::default(), hit_patterns: vec![], multiplicity: vec![], data: vec![] }
    }

    fn extract_partial_data(&mut self, buffer: &[u8]) -> Result<(), GrawFrameError> {

        let mut datum: GrawData;
        let mut buf_slice = buffer;
        let mut raw: u32;

        while buf_slice.len() != 0 {
            datum = GrawData::default();


            (buf_slice, raw) = parse_u32(buf_slice)?;
            datum.aget_id = GrawFrame::extract_aget_id(&raw);
            datum.channel = GrawFrame::extract_channel(&raw);
            datum.time_bucket_id = GrawFrame::extract_time_bucket_id(&raw);
            datum.sample = GrawFrame::extract_sample(&raw);

            datum.check_data()?;

            self.data.push(datum);
        }

        Ok(())
    }

    fn extract_full_data(&mut self, buffer: &[u8]) -> Result<(), GrawFrameError> {

        let mut datum: GrawData;
        let mut raw: u16;
        let mut aget_counters: Vec<u64> = vec![0, 0, 0, 0];
        let mut buf_slice = buffer;

        while buffer.len() != 0 {
            datum = GrawData::default();
            (buf_slice, raw) = parse_u16(buf_slice)?;
            datum.aget_id = GrawFrame::extract_aget_id_full(&raw);
            let aget_index: usize = datum.aget_id as usize;
            datum.sample = GrawFrame::extract_sample_full(&raw);
            datum.time_bucket_id = (aget_counters[aget_index] / 68) as u16; //integer division always rounds down
            datum.channel = (aget_counters[aget_index] % 68) as u8; // % operator in Rust is the remainder

            datum.check_data()?;

            self.data.push(datum);

            aget_counters[aget_index] += 1;
        }

        Ok(())
    }

    fn extract_aget_id(raw_item: &u32) -> u8 {
        ((raw_item & 0xC0000000) >> 30) as u8
    }

    fn extract_channel(raw_item: &u32) -> u8 {
        ((raw_item & 0x3F800000) >> 23) as u8
    }

    fn extract_time_bucket_id(raw_item: &u32) -> u16 {
        ((raw_item & 0x007FC000) >> 14) as u16
    }

    fn extract_sample(raw_item: &u32) -> i16 {
        (raw_item & 0x00000FFF) as i16
    }

    fn extract_aget_id_full(raw_item: &u16) -> u8 {
        ((raw_item & 0xC000) >> 14) as u8
    }

    fn extract_sample_full(raw_item: &u16) -> i16 {
        (raw_item & 0x0FFF) as i16
    }

    

}