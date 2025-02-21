use crate::session::*;
use encoding_rs::mem::decode_latin1;
use serde::{Deserialize, Serialize};
use serde_yaml::from_str as yaml_from;
use std::borrow::Cow;
use std::convert::TryInto;
use std::default::Default;
use std::ffi::CStr;
use std::fmt;
use std::io::Result as IOResult;
use std::marker::PhantomData;
use std::os::raw::{c_char, c_void};
use std::os::windows::raw::HANDLE;
use std::slice::from_raw_parts;
use std::time::Duration;
use thiserror::Error;
use winapi::shared::minwindef::LPVOID;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::handleapi::CloseHandle;
use winapi::um::memoryapi::{MapViewOfFile, OpenFileMappingW, UnmapViewOfFile, FILE_MAP_READ};
use winapi::um::synchapi::{OpenEventW, ResetEvent, WaitForSingleObject};
use winapi::um::winnt::SYNCHRONIZE;

/// System path where the shared memory map is located.
pub const TELEMETRY_PATH: &str = r"Local\IRSDKMemMapFileName";

/// Magic number specifying an unlimited number of laps
pub const UNLIMITED_LAPS: i32 = 32767;

/// Magic number specifying unlimited time
pub const UNLIMITED_TIME: f32 = 604800.0;

const DATA_EVENT_NAME: &str = r"Local\IRSDKDataValidEvent";

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Header {
    pub version: i32,              // Telemetry version
    pub status: i32,               // Status
    pub tick_rate: i32,            // Tick rate (Hz)
    pub session_info_version: i32, // Increments each time session info is updated
    pub session_info_length: i32,  // Length of session info data
    pub session_info_offset: i32,  // Offset of session info data

    pub n_vars: i32,        // Number of values
    pub header_offset: i32, // Offset to start of variables

    pub n_buffers: i32,     // # of buffers (<= 3 for now)
    pub buffer_length: i32, // Length per line
    pub padding: [u32; 2],  // Padding

    buffers: [ValueBuffer; 4], // Data buffers
}

/// Blocking telemetry interface
///
/// Calling `sample()` on a Blocking interface will block until a new telemetry sample is made available.
///
pub struct Blocking<'conn> {
    origin: *const c_void,
    event_handle: HANDLE,
    phantom: PhantomData<&'conn Connection>,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
struct ValueBuffer {
    pub ticks: i32,        // Tick count
    pub offset: i32,       // Offset
    pub padding: [u32; 2], // (16-byte align) Padding
}

#[derive(Clone)]
#[repr(C)]
struct ValueHeader {
    pub value_type: i32,     // Value type
    pub offset: i32,         // Value offset
    pub count: i32,          // Number of values for an array
    pub count_as_time: bool, // Values in array represent timeseries data

    _pad: [u8; 3],                                                   // Padding
    _name: [c_char; ValueHeader::MAX_VAR_NAME_LENGTH],               // Value name
    _description: [c_char; ValueHeader::MAX_VAR_DESCRIPTION_LENGTH], // Value description
    _unit: [c_char; ValueHeader::MAX_VAR_NAME_LENGTH],               // Value units
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ValueDescription {
    pub value: Value,
    pub count: usize,
    pub count_as_time: bool,

    pub name: String,
    pub description: String,
    pub unit: String,
}

///
/// Sample represents a single sample of telemetry data from iRacing
/// either from live telemetry, or from a telemetry file.
#[derive(Debug, Default)]
pub struct Sample {
    tick: i32,
    buffer: Vec<u8>,
    values: Vec<ValueHeader>,
}

/// Telemetry Value
///
/// Represents a single value in the telemetry.
/// Telemetry data is always quantitive but may be of varying numeric types, plus boolean.
///
/// The iRacing Telemetry documentation describes the data-type expected for a given telemetry measurement.
/// `Into` can be used when the expected data type is known, else `match` can be used to dynamically handle the
/// returned data type.
///
/// # Examples
///
/// ## Known, Expected Data Type
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use iracing::telemetry::{Connection, Value};
/// # use std::time::Duration;
/// # let conn = Connection::new()?;
/// # let sampler = conn.blocking()?;
/// # let sample = sampler.sample(Duration::from_millis(50))?;
/// use std::convert::TryInto;
///
/// let gear: i32 = sample.get("Gear").unwrap().try_into().unwrap();
/// # Ok(())
/// # }
/// ```
///
/// ## Unknown data type
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use iracing::telemetry::{Connection, Value};
/// # use std::time::Duration;
/// # let conn = Connection::new()?;
/// # let sampler = conn.blocking()?;
/// # let sample = sampler.sample(Duration::from_millis(50))?;
/// match sample.get("some_key") {
///     Err(err) => println!("Didn't find that value: {}", err),
///     Ok(value) => match value {
///         Value::CHAR(c) => println!("Value: {:x}", c),
///         Value::BOOL(b) => println!("Value: {}", if b { "True" } else { "False" }),
///         Value::INT(i) => println!("Value: {}", i),
///         Value::BITS(u) => println!("Value: 0x{:32b}", u),
///         Value::FLOAT(f) => println!("Value: {:.2}", f),
///         Value::DOUBLE(f) => println!("Value: {:.2}", f),
///         _  => println!("Unknown Value"),
///     }
/// };
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    CHAR(u8),
    BOOL(bool),
    INT(i32),
    BITS(u32),
    FLOAT(f32),
    DOUBLE(f64),
    UNKNOWN(()),
    IntVec(Vec<i32>),
    FloatVec(Vec<f32>),
    BoolVec(Vec<bool>),
}

impl From<i32> for Value {
    fn from(v: i32) -> Value {
        match v {
            0 => Value::CHAR(0x0),
            1 => Value::BOOL(false),
            2 => Value::INT(0),
            3 => Value::BITS(0x00),
            4 => Value::FLOAT(0.0),
            5 => Value::DOUBLE(0.0),
            _ => Value::UNKNOWN(()),
        }
    }
}

impl Value {
    pub fn size(&self) -> usize {
        match self {
            Self::CHAR(_) | Self::BOOL(_) | Self::BoolVec(_) => 1,
            Self::INT(_) | Self::BITS(_) | Self::FLOAT(_) | Self::IntVec(_) | Self::FloatVec(_) => {
                4
            }
            Self::DOUBLE(_) => 8,
            Self::UNKNOWN(_) => 1,
        }
    }
}

impl TryInto<i32> for Value {
    type Error = ValueError;

    fn try_into(self) -> Result<i32, Self::Error> {
        match self {
            Self::INT(n) => Ok(n),
            _ => Err(ValueError::TryInto("Value is not a signed 4-byte integer")),
        }
    }
}

impl TryInto<u32> for Value {
    type Error = ValueError;

    fn try_into(self) -> Result<u32, Self::Error> {
        match self {
            Self::INT(n) => Ok(n as u32),
            Self::BITS(n) => Ok(n),
            _ => Err(ValueError::TryInto("Value is not a 4-byte integer")),
        }
    }
}

impl TryInto<f32> for Value {
    type Error = ValueError;

    fn try_into(self) -> Result<f32, Self::Error> {
        match self {
            Self::FLOAT(n) => Ok(n),
            _ => Err(ValueError::TryInto("Value is not a float")),
        }
    }
}

impl TryInto<f64> for Value {
    type Error = ValueError;

    fn try_into(self) -> Result<f64, Self::Error> {
        match self {
            Self::DOUBLE(n) => Ok(n),
            Self::FLOAT(f) => Ok(f as f64),
            _ => Err(ValueError::TryInto("Value is not a float or double")),
        }
    }
}

impl From<Value> for bool {
    fn from(value: Value) -> Self {
        match value {
            Value::BOOL(b) => b,
            _ => false,
        }
    }
}

impl From<Value> for Vec<bool> {
    fn from(value: Value) -> Self {
        match value {
            Value::BoolVec(b) => b,
            _ => vec![false],
        }
    }
}

impl ValueHeader {
    ///
    /// Maximum length of a variable name/unit
    const MAX_VAR_NAME_LENGTH: usize = 32;

    ///
    /// Maximum length for a variable description
    const MAX_VAR_DESCRIPTION_LENGTH: usize = 64;

    /// Convert the name from a c_char[32] to a rust String
    pub fn name(&self) -> String {
        let name = unsafe { CStr::from_ptr(self._name.as_ptr()) };
        decode_latin1(name.to_bytes()).to_string()
    }

    pub fn description(&self) -> String {
        let description = unsafe { CStr::from_ptr(self._description.as_ptr()) };
        decode_latin1(description.to_bytes()).to_string()
    }

    pub fn unit(&self) -> String {
        let unit = unsafe { CStr::from_ptr(self._unit.as_ptr()) };
        decode_latin1(unit.to_bytes()).to_string()
    }
}

impl Default for ValueHeader {
    ///
    /// Create a new, empty ValueHeader
    fn default() -> Self {
        ValueHeader {
            value_type: 0,
            offset: 0,
            count: 0,
            count_as_time: false,
            _pad: [0; 3],
            _name: [0; ValueHeader::MAX_VAR_NAME_LENGTH],
            _unit: [0; ValueHeader::MAX_VAR_NAME_LENGTH],
            _description: [0; ValueHeader::MAX_VAR_DESCRIPTION_LENGTH],
        }
    }
}

impl fmt::Debug for ValueHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ValueHeader(name=\"{}\", type={}, count={}, offset={})",
            self.name(),
            self.value_type,
            self.count,
            self.offset
        )
    }
}

impl Header {
    fn latest_buffer(&self) -> (i32, ValueBuffer) {
        let mut buffer = self.buffers[0];
        let mut latest_tick = buffer.ticks;

        for b in self
            .buffers
            .iter()
            .skip(1)
            .take((self.n_buffers - 1) as usize)
        {
            if b.ticks > latest_tick {
                buffer = *b;
                latest_tick = b.ticks;
            }
        }

        (latest_tick, buffer)
    }

    fn var_buffer(&self, lb: ValueBuffer, from_loc: *const c_void) -> &[u8] {
        let sz = self.buffer_length as usize;

        let buffer_loc = from_loc as usize + lb.offset as usize;
        unsafe { from_raw_parts(buffer_loc as *const u8, sz) }
    }

    fn get_var_header(&self, from_loc: *const c_void) -> &[ValueHeader] {
        let n_vars = self.n_vars as usize;
        let header_loc = from_loc as usize + self.header_offset as usize;

        let content = unsafe { from_raw_parts(header_loc as *const ValueHeader, n_vars) };

        content
    }

    fn telemetry(&self, from_loc: *const c_void) -> Sample {
        let (tick, vbh) = self.latest_buffer();
        let value_buffer = self.var_buffer(vbh, from_loc);
        let value_header = self.get_var_header(from_loc);

        Sample::new(tick, value_header.to_vec(), value_buffer.to_vec())
    }
}

impl Sample {
    fn new(tick: i32, header: Vec<ValueHeader>, buffer: Vec<u8>) -> Self {
        Sample {
            tick,
            values: header,
            buffer,
        }
    }

    fn header_for(&self, name: &'static str) -> Option<ValueHeader> {
        for v in self.values.iter() {
            if v.name() == name {
                return Some(v.clone());
            }
        }
        None
    }

    ///
    /// Check if a given variable is available in the telemetry sample
    pub fn has(&self, name: &'static str) -> bool {
        self.header_for(name).is_some()
    }

    /// Gets all values in the same along with names and descriptions.
    ///
    /// Returns a vec of all values in the telemetry sample, along with
    /// additional metadata such as the name, description, unit and value count.
    ///
    /// Note: This method is expensive and will return a large number of values.
    ///       It should be used primarily for debugging, and for most use cases
    ///       Selecting only the values required with `get()` is suggested.
    pub fn all(&self) -> Vec<ValueDescription> {
        let r = self.values.iter().map(|v| {
            let val = self.value(v);

            ValueDescription {
                name: v.name(),
                description: v.description(),
                unit: v.unit(),
                count: v.count as usize,
                count_as_time: v.count_as_time,
                value: val,
            }
        });

        r.collect::<Vec<ValueDescription>>()
    }

    ///
    /// Get a Value from the sample.
    ///
    /// Read a single varialbe from the telemetry sample.
    ///
    /// Returns `Ok(Value)` if the telemetry value is available.
    /// Returns `Err(String)` if the value cannot be found.
    ///
    /// # Parameters
    ///
    /// `name`  Name of the telemetry variable to get
    ///   - see the iRacing Telemtry documentation for a complete list of possible values
    pub fn get(&self, name: &'static str) -> Result<Value, SampleError> {
        match self.header_for(name) {
            None => Err(SampleError::NoValue(format!("No value '{}' found", name))),
            Some(vh) => Ok(self.value(&vh)),
        }
    }

    fn value(&self, vh: &ValueHeader) -> Value {
        let vs = vh.offset as usize; // Value start
        let vt = Value::from(vh.value_type);
        let vz = vt.size();
        let ve = vs + vz;
        let vc = vh.count as usize;

        let raw_val = &self.buffer[vs..ve];

        let v: Value = match vt {
            Value::INT(_) => {
                if vc == 1 {
                    Value::INT(i32::from_le_bytes(raw_val.try_into().unwrap()))
                } else {
                    let mut values: Vec<i32> = Vec::with_capacity(vc);
                    for i in 0..vc - 1 {
                        values.push(i32::from_le_bytes(
                            self.buffer[vs + vz * i..vs + vz * (i + 1)]
                                .try_into()
                                .unwrap(),
                        ));
                    }

                    Value::IntVec(values)
                }
            }
            Value::FLOAT(_) => {
                if vc == 1 {
                    Value::FLOAT(f32::from_le_bytes(raw_val.try_into().unwrap()))
                } else {
                    let mut values: Vec<f32> = Vec::with_capacity(vc);

                    for i in 0..vc - 1 {
                        values.push(f32::from_le_bytes(
                            self.buffer[vs + vz * i..vs + vz * (i + 1)]
                                .try_into()
                                .unwrap(),
                        ));
                    }

                    Value::FloatVec(values)
                }
            }
            Value::DOUBLE(_) => Value::DOUBLE(f64::from_le_bytes(raw_val.try_into().unwrap())),
            Value::BITS(_) => Value::BITS(u32::from_le_bytes(raw_val.try_into().unwrap())),
            Value::CHAR(_) => Value::CHAR(raw_val[0]),
            Value::BOOL(_) => {
                if vc == 1 {
                    Value::BOOL(raw_val[0] > 0)
                } else {
                    let mut values: Vec<bool> = Vec::with_capacity(vc);

                    for i in 0..vc - 1 {
                        values.push(self.buffer[vs + i] > 0);
                    }

                    Value::BoolVec(values)
                }
            }
            _ => unimplemented!(),
        };

        v
    }
}

/// An error which occurs when telemetry samples cannot be read from the memory buffer.
#[derive(Debug, Error)]
pub enum TelemetryError {
    #[error("Invalid duration: {0}")]
    InvalidDuration(#[from] std::num::TryFromIntError),

    #[error("Windows Mutex was abandoned.")]
    Abandoned,

    #[error("Timeout elapsed while waiting for a signal. Duration={0}")]
    Timeout(usize),

    #[error("Unknown error occurred: {0}")]
    Unknown(u32),

    #[error("Wait failed: {0}")]
    WaitFailed(std::io::Error),
}

/// An error which occurs while retrieving session info.
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Text decode error: {0}")]
    Decode(Cow<'static, str>),

    #[error("YAML parse error: {0}")]
    ParseYAML(#[from] serde_yaml::Error),
}

/// An error which occurs when value conversions fail.
#[derive(Debug, Error)]
pub enum ValueError {
    #[error("{0}")]
    TryInto(&'static str),
}

/// An error which may occur when reading values from a sample.
#[derive(Debug, Error)]
pub enum SampleError {
    #[error("{0}")]
    NoValue(String),
}

impl Blocking<'_> {
    fn new(location: *const c_void) -> std::io::Result<Self> {
        let mut event_name: Vec<u16> = DATA_EVENT_NAME.encode_utf16().collect();
        event_name.push(0);

        let handle = unsafe {
            let handle: HANDLE = OpenEventW(SYNCHRONIZE, 0, event_name.as_ptr());

            if handle.is_null() {
                let errno = GetLastError() as i32;
                return Err(std::io::Error::from_raw_os_error(errno));
            }

            handle
        };

        Ok(Blocking {
            origin: location,
            event_handle: handle,
            phantom: PhantomData,
        })
    }

    /// Get the data header
    ///
    /// Reads the data header from the shared memory map and returns a copy of the header
    /// which can be used safely elsewhere.
    fn read_header(&self) -> Header {
        let raw_header = self.origin as *const Header;
        // SAFETY: This read CANNOT be done safely, since it is an unsynchronized clone of a struct
        // larger than AtomicUsize.
        unsafe { *raw_header }
    }

    ///
    /// Sample Telemetry Data
    ///
    /// Waits for new telemetry data up to `timeout` and returns a safe copy of the telemetry data.
    /// Returns an error on timeout or underlying system error.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use iracing::telemetry::Connection;
    /// use std::time::Duration;
    ///
    /// let conn = Connection::new()?;
    /// let sampler = conn.blocking()?;
    /// let sample = sampler.sample(Duration::from_millis(50))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn sample(&self, timeout: Duration) -> Result<Sample, TelemetryError> {
        let wait_time: u32 = timeout.as_millis().try_into()?;

        unsafe {
            let signal = WaitForSingleObject(self.event_handle, wait_time);

            match signal {
                0x80 => Err(TelemetryError::Abandoned),
                0x102 => Err(TelemetryError::Timeout(wait_time as usize)),
                0xFFFFFFFF => {
                    // Error
                    let errno = std::io::Error::from_raw_os_error(GetLastError() as i32);
                    Err(TelemetryError::WaitFailed(errno))
                }
                0x00 => {
                    // OK
                    ResetEvent(self.event_handle);
                    Ok(self.read_header().telemetry(self.origin))
                }
                _ => Err(TelemetryError::Unknown(signal as u32)),
            }
        }
    }
}

impl Drop for Blocking<'_> {
    fn drop(&mut self) {
        unsafe {
            let succ = CloseHandle(self.event_handle);
            if succ == 0 {
                let errno = GetLastError() as i32;
                panic!(
                    "Unable to close handle: {}",
                    std::io::Error::from_raw_os_error(errno)
                );
            }
        }
    }
}

///
/// iRacing live telemetry and session data connection.
///
/// Allows retrival of live data fro iRacing.
/// The data is provided using a shared memory map, allowing the simulator
/// to deposit data roughly every 16ms to be read.
///
/// # Examples
///
/// ```
/// use iracing::telemetry::Connection;
///
/// let _ = Connection::new().expect("Unable to find telemetry data");
/// ```
pub struct Connection {
    mapping: HANDLE,
    location: *mut c_void,
}

impl Connection {
    pub fn new() -> IOResult<Connection> {
        let mut path: Vec<u16> = TELEMETRY_PATH.encode_utf16().collect();
        path.push(0);

        unsafe {
            let mapping: HANDLE = OpenFileMappingW(FILE_MAP_READ, 0, path.as_ptr());
            if mapping.is_null() {
                let errno = GetLastError() as i32;
                return Err(std::io::Error::from_raw_os_error(errno));
            }

            let location: LPVOID = MapViewOfFile(mapping, FILE_MAP_READ, 0, 0, 0);
            if location.is_null() {
                let errno = GetLastError() as i32;
                CloseHandle(mapping); // Ignore return value; we are already handling another error.
                return Err(std::io::Error::from_raw_os_error(errno));
            }

            Ok(Connection { mapping, location })
        }
    }

    ///
    /// Get the data header
    ///
    /// Reads the data header from the shared memory map and returns a copy of the header
    /// which can be used safely elsewhere.
    fn read_header(&self) -> Header {
        let raw_header = self.location as *const Header;
        // SAFETY: This read CANNOT be done safely, since it is an unsynchronized clone of a struct
        // larger than AtomicUsize.
        unsafe { *raw_header }
    }

    ///
    /// Get session information
    ///
    /// Get general session information - This data is mostly static and contains
    /// overall information related to the current or replayed session
    ///
    /// # Examples
    ///
    /// ```
    /// use iracing::telemetry::Connection;
    ///
    /// match Connection::new().expect("Unable to open session").session_info() {
    ///     Ok(session) => println!("Track Name: {}", session.weekend.track_display_name),
    ///     Err(e) => println!("Invalid Session")
    /// };
    /// ```
    pub fn session_info(&self) -> Result<SessionDetails, SessionError> {
        let header = self.read_header();

        let start = (self.location as usize + header.session_info_offset as usize) as *const u8;
        let size = header.session_info_length as usize;

        let data: &[u8] = unsafe { from_raw_parts(start, size) };

        // Decode the data as Latin-1 (Rust wants UTF-8)
        let content = decode_latin1(data);
        let details = yaml_from(&content)?;

        Ok(details)
    }

    ///
    /// Get latest telemetry.
    ///
    /// Get the latest live telemetry data, the telemetry is updated roughtly every 16ms
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), std::io::Error> {
    /// use iracing::telemetry::Connection;
    ///
    /// let sample = Connection::new()?.telemetry();
    /// # Ok(())
    /// # }
    /// ```
    pub fn telemetry(&self) -> Sample {
        let header = self.read_header();
        header.telemetry(self.location as *const std::ffi::c_void)
    }

    ///
    /// Get Blocking Telemetry Interface.
    ///
    /// Creates a new `iracing::telemetry::Blocking` connector which allows telemetry samples to
    /// be collected, and will wait and block until a new sample is available, or a timeout is reached.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use iracing::telemetry::Connection;
    /// use std::time::Duration;
    ///
    /// let conn = Connection::new()?;
    /// let sampler = conn.blocking()?;
    /// let sample = sampler.sample(Duration::from_millis(50))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn blocking(&self) -> IOResult<Blocking<'_>> {
        Blocking::new(self.location)
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe {
            let succ = UnmapViewOfFile(self.location);
            if succ == 0 {
                let errno = GetLastError() as i32;
                CloseHandle(self.mapping); // Ignore return value; we are already handling another error.
                panic!(
                    "Unable to unmap file: {}",
                    std::io::Error::from_raw_os_error(errno)
                );
            }

            let succ = CloseHandle(self.mapping);
            if succ == 0 {
                let errno = GetLastError() as i32;
                panic!(
                    "Unable to close handle: {}",
                    std::io::Error::from_raw_os_error(errno)
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_info() {
        let session_info = Connection::new()
            .expect("Unable to open telemetry")
            .session_info();
        assert!(session_info.is_ok());
    }

    #[test]
    fn test_latest_telemetry() {
        let session_tick: Sample = Connection::new()
            .expect("Unable to open telemetry")
            .telemetry();
    }
}
