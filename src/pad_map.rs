use std::hash::Hash;
use std::io::Read;
use std::path::Path;
use std::fs::File;

use fxhash::FxHashMap;

use super::error::PadMapError;

const ENTRIES_PER_LINE: usize = 5;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct HardwareID {
    pub cobo_id: usize,
    pub asad_id: usize,
    pub aget_id: usize,
    pub channel: usize,
    pub pad_id: usize
}

impl HardwareID {
    pub fn new(cobo_id: &u8, asad_id: &u8, aget_id: &u8, channel: &u8, pad_id: &u64) -> Self{
        HardwareID {
            cobo_id: *cobo_id as usize,
            asad_id: *asad_id as usize,
            aget_id: *aget_id as usize,
            channel: *channel as usize,
            pad_id: *pad_id as usize
        }
    }
}

impl Hash for HardwareID {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.pad_id.hash(state)
    }
}

//Generate a unique id number for a given hardware location
fn generate_uuid(cobo_id: &u8, asad_id: &u8, aget_id: &u8, channel_id: &u8) -> u64 {
    return (*channel_id as u64) + (*aget_id as u64) * 100 + (*asad_id as u64) * 10_000 + (*cobo_id as u64) * 1_000_000;
}

#[derive(Debug, Clone, Default)]
pub struct PadMap {
    map: FxHashMap<u64, HardwareID>
}

impl PadMap {
    pub fn new(path: &Path) -> Result<Self, PadMapError> {

        let mut file = File::open(path)?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;
        let mut cb_id: u8;
        let mut ad_id: u8;
        let mut ag_id: u8;
        let mut ch_id: u8;
        let mut pd_id: u64;
        let mut uuid: u64;
        let mut hw_id: HardwareID;

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

            uuid = generate_uuid(&cb_id, &ad_id, &ag_id, &ch_id);
            hw_id = HardwareID::new(&cb_id, &ad_id, &ag_id, &ch_id, &pd_id);
            pm.map.insert(uuid, hw_id);
        }

        Ok(pm)
    }

    pub fn get_hardware_id(&self, cobo_id: &u8, asad_id: &u8, aget_id: &u8, channel_id: &u8) -> Option<&HardwareID> {
        let uuid = generate_uuid(cobo_id, asad_id, aget_id, channel_id);
        let val = self.map.get(&uuid);
        return val;
    }
}