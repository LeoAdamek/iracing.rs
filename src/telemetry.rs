use std::convert::TryInto;
use std::default::Default;
use std::error::Error;
use std::ffi::CStr;
use std::fmt::Display;
use std::fmt;
use std::os::windows::raw::HANDLE;
use std::ptr::null;
use std::slice::from_raw_parts;
use std::time::Duration;

use libc::c_char;
use libc::c_void;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::minwinbase::LPSECURITY_ATTRIBUTES;
use winapi::um::synchapi::{CreateEventW,WaitForSingleObject,ResetEvent};

use serde::{Serialize, Deserialize};

const DATA_EVENT_NAME: &'static str = "Local\\IRSDKDataValidEvent";

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
pub struct Blocking {
    origin: *const c_void,
    values: Vec<ValueHeader>,
    header: Header,
    event_handle: HANDLE
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
    pub count_as_time: bool, // ???

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
    pub unit: String
}

///
/// Sample represents a single sample of telemetry data from iRacing
/// either from live telemetry, or from a telemetry file.
#[derive(Debug, Default)]
pub struct Sample {
    tick: i32,
    buffer: Vec<u8>,
    values: Vec<ValueHeader>
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
/// use iracing::telemetry::Sample;
/// 
/// let s: Sample = some_sample_getter();
/// 
/// let gear: i32 = s.get("Gear").unwrap().into();
/// ```
/// 
/// ## Unknown data type
/// 
/// ```
/// use iracing::telemtry::{Sample, Value};
/// 
/// let v: &'static str = some_input_for_var_name();
/// let s: Sample = some_sample_getter();
/// 
/// match s.get(v) {
///     None => println!("Didn't find that value");
///     Some(value) => match {
///         Value::CHAR(c) => println!("Value: {:x}", c),
///         Value::BOOL(b) => println!("Yes") if b,
///         Value::INT(i) => println!("Value: {}", i),
///         Value::BITS(u) => println!("Value: 0x{:32b}", u),
///         Value::FLOAT(f) | Value::DOUBLE(f) => println!("Value: {:.2}", f),
///         _  => println!("Unknown Value")
///     }
/// };  
/// ```
#[derive(Debug,Copy,Clone,Serialize,Deserialize)]
pub enum Value {
    CHAR(u8),
    BOOL(bool),
    INT(i32),
    BITS(u32),
    FLOAT(f32),
    DOUBLE(f64),
    UNKNOWN(())
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
            _ => Value::UNKNOWN(())
        }
    }
}

impl Value {
    pub fn size(&self) -> usize {
        match self {
            Self::CHAR(_) | Self::BOOL(_) => 1,
            Self::INT(_) | Self::BITS(_) | Self::FLOAT(_) => 4,
            Self::DOUBLE(_) => 8,
            Self::UNKNOWN(_) => 1
        }
    }
}

impl Into<i32> for Value {
    fn into(self) -> i32 {
        match self {
            Self::INT(n) => n,
            _ => 0
        }
    }
}

impl Into<u32> for Value {
    fn into(self) -> u32 {
        match self {
            Self::INT(n) => n as u32,
            Self::BITS(n) => n,
            _ => 0
        }
    }
}

impl Into<f32> for Value {
    fn into(self) -> f32 {
        match self {
            Self::FLOAT(n) => n,
            _ => 0.0
        }
    }
}

impl Into<f64> for Value {
    fn into(self) -> f64 {
        match self {
            Self::DOUBLE(n) => n,
            _ => 0.0
        }
    }
}

impl Into<bool> for Value {
    fn into(self) -> bool {
        match self {
            Self::BOOL(b) => b,
            _ => false
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
    /// Expect that we won't have any encoding issues as the values should always be ASCII
    pub fn name(&self) -> String {
        let name = unsafe { CStr::from_ptr(self._name.as_ptr()) };
        name.to_string_lossy().to_string()
    }

    pub fn description(&self) -> String {
        let description = unsafe { CStr::from_ptr(self._description.as_ptr()) };
        description.to_string_lossy().to_string()
    }

    pub fn unit(&self) -> String {
        let unit = unsafe { CStr::from_ptr(self._unit.as_ptr()) };
        unit.to_string_lossy().to_string()
    }

}


// Workaround to handle cloning var descriptions which are [u8; 64] thus cannot be derived
struct VarDescription([c_char; ValueHeader::MAX_VAR_DESCRIPTION_LENGTH]);

impl Clone for VarDescription {
    fn clone(&self) -> Self {
        let mut new = VarDescription([0; ValueHeader::MAX_VAR_DESCRIPTION_LENGTH]);

        for i in 1 .. ValueHeader::MAX_VAR_DESCRIPTION_LENGTH {
            new.0[i] = self.0[i];
        }

        new
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
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
        let mut latest_tick: i32 = 0;
        let mut buffer = self.buffers[0];

        for b in self.buffers.iter() {

            if b.ticks > latest_tick {
                buffer = *b;
                latest_tick = b.ticks;
            }
        }

        return (latest_tick, buffer);
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

        content.clone()
    }

    pub fn telemetry(
        &self,
        from_loc: *const c_void,
    ) -> Result<Sample, Box<dyn std::error::Error>> {
        let (tick, vbh) = self.latest_buffer();
        let value_buffer = self.var_buffer(vbh, from_loc);
        let value_header = self.get_var_header(from_loc);

        Ok(Sample::new(tick, value_header.to_vec(), value_buffer.to_vec()))
    }
}

impl Sample {
    fn new(tick: i32, header: Vec<ValueHeader>, buffer: Vec<u8>) -> Self {
        Sample {
            tick: tick,
            values: header,
            buffer: buffer,
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
        match self.header_for(name) {
            Some(_) => true,
            None => false
        }
    }

    pub fn all(&self) -> Vec<ValueDescription> {

        let r = self.values.iter().map( |v| {
            let val = self.value(v);

            ValueDescription{
                name: v.name().to_owned(),
                description: v.description().to_owned(),
                unit: v.unit().to_owned(),
                count: v.count as usize,
                count_as_time: v.count_as_time,
                value: val
            }
        });

        r.collect::<Vec<ValueDescription>>()
    }

    ///
    /// Get a Value from the sample.
    /// 
    /// Read a single varialbe from the telemetry sample.
    /// 
    /// Returns `None` if the value is not present in the sample.
    /// Returns `Some(Value)` if the telemetry value is available.
    /// Panics if an unknown data-type is encountered.
    /// 
    /// # Parameters
    /// 
    /// `name`  Name of the telemetry variable to get
    ///   - see the iRacing Telemtry documentation for a complete list of possible values
    pub fn get(&self, name: &'static str) -> Option<Value> {
        match self.header_for(name) {
            None => None,
            Some(vh) => {
               Some(self.value(&vh))
            }
        }
    }

    fn value(&self, vh: &ValueHeader) -> Value {
        let vs = vh.offset as usize; // Value start
        let vt = Value::from(vh.value_type);
        let ve = vs + vt.size(); // Value end = Value Start + Value Size
        
        let raw_val = &self.buffer[vs..ve];

        let v: Value;
        v = match vt {
            Value::INT(_) => Value::INT( i32::from_le_bytes( raw_val.try_into().unwrap() )),
            Value::FLOAT(_) => Value::FLOAT( f32::from_le_bytes( raw_val.try_into().unwrap() )),
            Value::DOUBLE(_) => Value::DOUBLE( f64::from_le_bytes( raw_val.try_into().unwrap() )),
            Value::BITS(_) => Value::BITS( u32::from_le_bytes( raw_val.try_into().unwrap() )),
            Value::CHAR(_) => Value::CHAR(raw_val[0] as u8),
            Value::BOOL(_) => Value::BOOL(raw_val[0] > 0),
            _ => unimplemented!()
        };

        v
    }
}

///
/// Telemetry Error
/// 
/// An error which occurs when telemetry samples cannot be read from the memory buffer.
#[derive(Debug)]
pub enum TelemetryError {
    ABANDONED,
    TIMEOUT(usize),
    UNKNOWN(u32)
}

impl Display for TelemetryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ABANDONED => write!(f, "{}", "Abandoned"),
            Self::TIMEOUT(ms) => write!(f, "Timeout after {}ms", ms),
            Self::UNKNOWN(v) => write!(f, "Unknown error code = {:x?}", v)
        }
    }
}

impl Error for TelemetryError {
}


impl Blocking {
    pub fn new(location: *const c_void, head: Header) -> std::io::Result<Self> {
        let values = head.get_var_header(location).to_vec();
    
        let mut event_name: Vec<u16> = DATA_EVENT_NAME.encode_utf16().collect();
        event_name.push(0);

        let sc: LPSECURITY_ATTRIBUTES = unsafe { std::mem::zeroed() };

        let handle: HANDLE = unsafe { CreateEventW(sc, 0, 0, event_name.as_ptr()) };

        if null() == handle {
            let errno: i32 = unsafe { GetLastError() as i32 };

            return Err(std::io::Error::from_raw_os_error(errno));
        }


        Ok(Blocking{
           origin: location,
           header: head,
           values: values,
           event_handle: handle
        }) 
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
    /// use iracing::Connection;
    /// use std::time::Duration;
    /// 
    /// let sampler = Connection::new()?.blocking()?;
    /// let sample = sampler.get(Duration::from_millis(50))?;
    /// ```
    pub fn sample(&self, timeout: Duration) -> Result<Sample, Box<dyn Error>> {
        
        let wait_time: u32 = match timeout.as_millis().try_into() {
            Ok(v) => v,
            Err(e) => return Err(Box::new(e))
        };

        let signal = unsafe { WaitForSingleObject(self.event_handle, wait_time)  };

        match signal {
            0x80 => Err(Box::new(TelemetryError::ABANDONED)), // Abandoned
            0x102 => Err(Box::new(TelemetryError::TIMEOUT(wait_time as usize))), // Timeout
            0xFFFFFFFF => { // Error
                let errno = unsafe { GetLastError() as i32 };
                Err(Box::new(std::io::Error::from_raw_os_error(errno)))
            }, 
            0x00 => { // OK
                unsafe { ResetEvent(self.event_handle) };
                self.header.telemetry(self.origin)
            }
            _ => Err(Box::new(TelemetryError::UNKNOWN(signal as u32)))
        }
    }
}