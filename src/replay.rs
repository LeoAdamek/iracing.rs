use chrono::NaiveDateTime;
use std::io;
use std::io::Read;
use std::io::Result as IOResult;
use std::io::{Error as IOError, ErrorKind};

/// Magic number found at the start of replay files
pub const FILE_MAGIC: &[u8] = b"YLPR";

/// Length of an individual entry in the file preamble.
const ENTRY_LENGTH: usize = 12;

/// A Replay is a pre-recorded stream of data from iRacing which includes metadata
/// as well as
#[derive(Debug)]
pub struct Replay<R: Read> {
    pub reader: R,
    pub metadata: Header,
}

/// Header is the top-level header data from a replay.
/// `Replay::new` will eagerly load this data.
#[derive(Debug)]
pub struct Header {
    pub user_name: String,
    pub timestamp: NaiveDateTime,
    pub track: String,
    pub layout: Option<String>,
    pub session_id: u32,
    pub user_id: u32,
    pub user_car_id: u32,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Default)]
pub struct Entry {
    pub id: i32,
    pub car_id: u32,
    pub class_id: u32,
    pub car_name: String,
}

pub struct Driver {
    pub name: String,
}

impl Default for Header {
    fn default() -> Self {
        Header {
            user_name: String::default(),
            timestamp: NaiveDateTime::from_timestamp(0, 0),
            track: String::default(),
            layout: None,
            session_id: 0u32,
            user_id: 0u32,
            user_car_id: 0u32,
            entries: Vec::default(),
        }
    }
}

impl Header {
    /// Load Header data form a `Read`
    pub fn from<R: Read>(mut r: R) -> IOResult<Self> {
        let mut result = Self::default();

        // Skip 10 words
        skip(&mut r, 40)?;

        // Read the driver ID
        let mut raw_word = [0u8; 4];
        r.read_exact(&mut raw_word)?;
        result.user_id = u32::from_le_bytes(raw_word);

        // Read the driver's Car ID
        r.read_exact(&mut raw_word)?;
        result.user_car_id = u32::from_le_bytes(raw_word);

        r.read_exact(&mut raw_word)?;

        result.user_name = read_str(&mut r, 64)?;

        // Skip some more uninteresting data
        skip(&mut r, 8)?;

        // Skip the first entries list (we can get this data and more later in the file)
        // But first we need to know how many entries there are.
        r.read_exact(&mut raw_word)?;
        let entries_count = u32::from_le_bytes(raw_word) as usize;

        // Read the timestamp (null-terminated string of format YYYY-mm-dd hh:MM:ss) - Up to 32 bytes
        let timestamp_str = read_str(&mut r, 32)?;

        // Attempt to parse the timestamp (first 20 chars)
        match NaiveDateTime::parse_from_str(timestamp_str.as_str(), "%Y-%m-%d %H:%M:%S") {
            Ok(val) => result.timestamp = val,
            Err(e) => {
                println!("Error: {:?} / {}", e, e);
                return Err(IOError::new(ErrorKind::InvalidInput, e));
            }
        }

        // Skip more nothingness
        skip(&mut r, 120)?;
        skip(&mut r, entries_count * ENTRY_LENGTH)?;

        let mut raw_asset_list_length = [0u8; 4];
        r.read_exact(&mut raw_asset_list_length)?;

        let asset_list_length = u32::from_le_bytes(raw_asset_list_length) as usize;
        //read_str(&mut r, asset_list_length);
        skip(&mut r, asset_list_length)?;

        // Skip some more bytes
        skip(&mut r, 6)?;

        // Right now we chomp some spaces until we return to word-alignment
        // TODO: Chomp the spaces until we return to word alignment.
        let _padding: Vec<u8> = r
            .by_ref()
            .bytes()
            .take_while(|b| b.as_ref().unwrap() == &b' ')
            .map(|b| b.unwrap())
            .collect();

        skip(&mut r, 27)?;

        // Load another 4 bytes;
        r.read_exact(&mut raw_word)?;
        result.session_id = u32::from_le_bytes(raw_word);

        skip(&mut r, 116)?;

        // Read the track and layout name
        let track_layout = read_str(&mut r, 64)?;

        match track_layout.chars().position(|c| c == '\\') {
            None => {
                result.track = track_layout;
            }

            Some(position) => {
                let (track, layout) = track_layout.split_at(position);

                result.track = track.to_owned();
                result.layout = Some(layout[1..].to_owned());
            }
        }

        Ok(result)
    }
}

/// Skip `length` bytes from the reader and discard them.
#[inline]
fn skip<R: Read>(mut reader: R, length: usize) -> IOResult<()> {
    io::copy(&mut reader.by_ref().take(length as u64), &mut io::sink())?;
    Ok(())
}

// Helper to read `length` bytes from a reader and return it as a `String`
fn read_str<R: Read>(mut reader: R, length: usize) -> IOResult<String> {
    let mut raw_string_bytes = vec![0u8; length];
    reader.read_exact(&mut raw_string_bytes)?;

    // Find the first null byte
    let nul = raw_string_bytes
        .iter()
        .position(|&b| b == 0)
        .expect("Given string does not terminate within given length");

    Ok(String::from_utf8((raw_string_bytes[..nul]).to_vec()).unwrap())
}

impl<R: Read> Replay<R> {
    /// Create a new replay from a Read
    pub fn new(mut r: R) -> IOResult<Self> {
        validate_reader(&mut r)?;

        let metadata = Header::from(&mut r)?;

        Ok(Replay {
            reader: r,
            metadata,
        })
    }
}

/// Validate the given reader contains contains replay data.
///
/// This function consumes the first 4 bytes of data from the reader.
pub fn validate_reader<R: Read>(mut src: R) -> IOResult<()> {
    let mut magic = [0u8; 4];

    src.read_exact(&mut magic[..])?;

    let valid = magic.iter().zip(FILE_MAGIC.iter()).all(|(a, b)| a == b);

    if valid {
        Ok(())
    } else {
        Err(IOError::new(
            ErrorKind::InvalidData,
            "Invalid data at start of stream",
        ))
    }
}

#[cfg(test)]
mod tests {

    use crate::replay::Header;
    use std::fs::File;
    use std::io::BufReader;
    use std::io::ErrorKind;

    #[test]
    fn validate_reader() {
        // Valid readers
        let valid_values = [
            String::from("YLPR THIS IS VALID"),
            String::from("YLPR\" THIS IS ALSO VALID"),
        ];
        let invalid_valutes = [
            String::from("THIS IS NOT VALID"),
            String::from("YLLR\" THIS ISNT VALID EITHER"),
        ];

        for v in valid_values.iter() {
            let r = BufReader::new(v.as_bytes());

            assert!(crate::replay::validate_reader(r).is_ok());
        }

        for v in invalid_valutes.iter() {
            let r = BufReader::new(v.as_bytes());

            let err = crate::replay::validate_reader(r).unwrap_err();

            assert_eq!(err.kind(), ErrorKind::InvalidData);
        }
    }

    #[test]
    fn load_metadata() {
        let mut replay_file = File::open("./subses36491425.rpy").unwrap();

        crate::replay::validate_reader(&mut replay_file).unwrap();

        let metadata = Header::from(replay_file).unwrap();

        println!("Metadata = {:?}", metadata);

        assert_eq!(metadata.user_id, 81797u32);
        assert_eq!(metadata.session_id, 36491425u32);
        assert_eq!(metadata.track, String::from("iowa"));
        assert_eq!(metadata.layout, Some(String::from("oval")));
        assert_eq!(metadata.user_name, String::from("L W Adamek"));
    }
}
