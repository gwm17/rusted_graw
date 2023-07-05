use std::fmt::Display;
use std::path::PathBuf;
use std::error::Error;

use super::constants::*;

/*
    GrawData errors
 */
#[derive(Debug, Clone)]
pub enum GrawDataError {
    BadAgetID(u8),
    BadChannel(u8),
    BadTimeBucket(u16),
}

impl Display for GrawDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GrawDataError::BadAgetID(id) => write!(f, "Invaild aget ID {} found in GrawData!", id),
            GrawDataError::BadChannel(chan) => write!(f, "Invalid channel {} found in GrawData!", chan),
            GrawDataError::BadTimeBucket(bucket) => write!(f, "Invalid time bucket {} found in GrawData!", bucket)
        }
    }
}

impl Error for GrawDataError {

}

/*
    GrawFrame errors
 */
#[derive(Debug, Clone)]
pub enum GrawFrameError {
    ParsingError,
    IncorrectMetaType(u8),
    IncorrectFrameSize(u32, u32),
    IncorrectFrameType(u16),
    IncorrectHeaderSize(u16),
    IncorrectItemSize(u16),
    IncorrectNumberOfItems(u32, u32),
    BadDatum(GrawDataError)
}

impl From<GrawDataError> for GrawFrameError {
    fn from(value: GrawDataError) -> Self {
        GrawFrameError::BadDatum(value)
    }
}

impl Display for GrawFrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GrawFrameError::ParsingError => write!(f, "Error parsing buffer into GrawFrame!"),
            GrawFrameError::IncorrectMetaType(t) => write!(f, "Incorrect meta type found for GrawFrame! Found: {} Expected: {}", t, EXPECTED_META_TYPE),
            GrawFrameError::IncorrectFrameSize(s, cs) => write!(f, "Incorrect frame size found for GrawFrame! Found: {}, Expected: {}", s, cs),
            GrawFrameError::IncorrectFrameType(t) => write!(f, "Incorrect frame type found for GrawFrame! Found: {}, Expected: {} or {}", t, EXPECTED_FRAME_TYPE_FULL, EXPECTED_FRAME_TYPE_PARTIAL),
            GrawFrameError::IncorrectHeaderSize(s) => write!(f, "Incorrect header size found for GrawFrame! Found: {}, Expected: {}", s, EXPECTED_HEADER_SIZE),
            GrawFrameError::IncorrectItemSize(s) => write!(f, "Incorrect item size found for GrawFrame! Found: {}, Expected: {} or {}", s, EXPECTED_ITEM_SIZE_FULL, EXPECTED_ITEM_SIZE_PARTIAL),
            GrawFrameError::IncorrectNumberOfItems(s, cs) => write!(f, "Incorrect number of items in GrawFrame! Found: {}, Expected: {}", s, cs),
            GrawFrameError::BadDatum(e) => write!(f, "Bad datum found in GrawFrame! Error: {}", e)
        }
    }
}

impl Error for GrawFrameError {

}


/*
    GrawFile errors
 */

#[derive(Debug)]
pub enum GrawFileError {
    BadFrame(GrawFrameError),
    BadFilePath(PathBuf),
    EndOfFile,
    IOError(std::io::Error)
}

impl From<GrawFrameError> for GrawFileError {
    fn from(value: GrawFrameError) -> Self {
        GrawFileError::BadFrame(value)
    }
}

impl From<std::io::Error> for GrawFileError {
    fn from(value: std::io::Error) -> Self {
        GrawFileError::IOError(value)
    }
}

impl Display for GrawFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GrawFileError::BadFrame(frame) => write!(f, "Bad frame found when reading GrawFile! Error: {}", frame),
            GrawFileError::BadFilePath(path) => write!(f, "File {} does not exist at GrawFile::new!", path.display()),
            GrawFileError::EndOfFile => write!(f, "File reached end!"),
            GrawFileError::IOError(e) => write!(f, "GrawFile recieved an io error: {}!", e)
        }
    }
}

impl Error for GrawFileError {

}

/*
    PadMap errors
 */

#[derive(Debug)]
pub enum PadMapError {
    IOError(std::io::Error),
    ParsingError(std::num::ParseIntError),
    BadFileFormat
}

impl From<std::io::Error> for PadMapError {
    fn from(value: std::io::Error) -> Self {
        PadMapError::IOError(value)
    }
}

impl From<std::num::ParseIntError> for PadMapError {
    fn from(value: std::num::ParseIntError) -> Self {
        PadMapError::ParsingError(value)
    }
}

impl Display for PadMapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PadMapError::IOError(e) => write!(f, "PadMap recieved an io error: {}", e),
            PadMapError::ParsingError(e) => write!(f, "PadMap error recieved a parsing error: {}", e),
            PadMapError::BadFileFormat => write!(f, "PadMap found a bad file format while reading the map file! Expected .csv without whitespaces")
        }
    }
}

impl Error for PadMapError {

}

/*
    Event errors
 */
#[derive(Debug)]
pub enum EventError {
    InvalidHardware(u8, u8, u8, u8),
    MismatchedEventID(u32, u32),
}

impl Display for EventError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventError::InvalidHardware(cb, ad, ag, ch) => write!(f, "Event found hardware which does not correspond to a valid pad! CoBo: {}, AsAd: {}, AGET: {}, Channel: {}", cb, ad, ag, ch),
            EventError::MismatchedEventID(given, exp) => write!(f, "Event was given a mismatched event id! Given: {}, Expected: {}", given, exp)
        }
    }
}

impl Error for EventError {
    
}

/*
    Merger errors
 */

#[derive(Debug)]
pub enum MergerError {
    FileError(GrawFileError),
    SendError,
    NoFilesError,
    IOError(std::io::Error)
}

impl From<GrawFileError> for MergerError {
    fn from(value: GrawFileError) -> Self {
        MergerError::FileError(value)
    }
}

impl From<std::io::Error> for MergerError {
    fn from(value: std::io::Error) -> Self {
        MergerError::IOError(value)
    }
}

impl Display for MergerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MergerError::FileError(e) => write!(f, "A file error occurred while merging! Error: {}", e),
            MergerError::SendError => write!(f, "An error occurred sending data from the merger!"),
            MergerError::NoFilesError => write!(f, "Merger could not find any files with .graw extension!"),
            MergerError::IOError(e) => write!(f, "The merger recieved an io error: {}", e)
        }
    }
}

impl Error for MergerError {

}

/*
    EventBuilder errors
 */

#[derive(Debug)]
pub enum EventBuilderError {
    SendError,
    EventError(EventError)
}

impl From<EventError> for EventBuilderError {
    fn from(value: EventError) -> Self {
        Self::EventError(value)
    }
}

impl Display for EventBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SendError => write!(f, "An error occurred sending an event from the EventBuilder!"),
            Self::EventError(val) => write!(f, "The EventBuilder recieved an event error: {}", val)
        }
    }
}

impl Error for EventBuilderError {

}