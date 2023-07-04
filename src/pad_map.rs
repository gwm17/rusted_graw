use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::fs::File;

use super::graw_frame::GrawData;
use super::error::PadMapError;

const ENTRIES_PER_LINE: usize = 5;

//Generate a unique id number for a given hardware location
fn generate_hardware_id(cobo_id: &u8, asad_id: &u8, aget_id: &u8, channel_id: &u8) -> u64 {
    return (*channel_id as u64) + (*aget_id as u64) * 100 + (*asad_id as u64) * 10_000 + (*cobo_id as u64) * 1_000_000;
}

#[derive(Debug, Clone, Default)]
pub struct PadMap {
    map: HashMap<u64, u64>
}

impl PadMap {
    pub fn new(path: &Path) -> Result<Self, PadMapError> {

        let mut file = File::open(path)?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;
        let mut cb_id: u8 = 0;
        let mut ad_id: u8 = 0;
        let mut ag_id: u8 = 0;
        let mut ch_id: u8 = 0;
        let mut pd_id: u64 = 0;
        let mut hw_id: u64 = 0;

        let mut pm = PadMap::default();

        for line in contents.lines() {
            let entries: Vec<&str> =  line.split_terminator(",").collect();

            if entries.len() < ENTRIES_PER_LINE {
                return Err(PadMapError::BadFileFormat);
            }

            cb_id = entries[0].parse()?;
            ad_id = entries[1].parse()?;
            ag_id = entries[2].parse()?;
            ch_id = entries[3].parse()?;
            pd_id = entries[4].parse()?;

            hw_id = generate_hardware_id(&cb_id, &ad_id, &ag_id, &ch_id);

            pm.map.insert(hw_id, pd_id);
        }

        Ok(pm)
    }

    pub fn get_pad_id(&self, cobo_id: &u8, asad_id: &u8, aget_id: &u8, channel_id: &u8) -> Option<&u64> {
        let hw_id = generate_hardware_id(cobo_id, asad_id, aget_id, channel_id);
        return self.map.get(&hw_id);
    }
}