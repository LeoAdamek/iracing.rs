use libc::c_char;
use libc::c_void;
use std::ffi::CStr;
use std::fmt;
use std::slice::from_raw_parts;
use std::convert::TryInto;


const DATA_EVENT_NAME: &'static str = "Local\\IRSDKDataValidEvent";

///
/// Session / Telemetry Data Header
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

///
/// Sample represents a single sample of telemetry data from iRacing
/// either from live telemetry, or from a telemetry file.
#[derive(Debug, Default)]
pub struct Sample {
    buffer: Vec<u8>,
    values: Vec<ValueHeader>
}

bitflags! {
    /**
     * Current warnings / status flags of the engine.
     */
    #[derive(Default)]
    pub struct EngineWarnings: u32 {
        const WATER_TEMPERATURE = 0x00;
        const FUEL_PRESSURE = 0x02;
        const OIL_PRESSURE = 0x04;
        const ENGINE_STALLED = 0x08;
        const PIT_SPEED_LIMITER = 0x10;
        const REV_LIMITER_ACTIVE = 0x20;
    }
}

bitflags! {
    ///
    /// Bitfield of current camera state
    ///
    /// # Examples
    ///
    /// ```
    /// use iracing::telemetry::CameraState;
    ///
    /// let very_scenic = CameraState::UI_HIDDEN | CameraState::IS_SCENIC_ACTIVE;
    /// ```
    #[derive(Default)]
    pub struct CameraState: u32 {
        const IS_SESSION_SCREEN = 0x01;
        const IS_SCENIC_ACTIVE = 0x02;

        const CAM_TOOL_ACTIVE = 0x04;
        const UI_HIDDEN = 0x08;
        const USE_AUTO_SHOT_SELECTION = 0x10;
        const USE_TEMPORARY_EDITS = 0x20;
        const USE_KEY_ACCELERATION = 0x40;
        const USE_KEY_10X_ACCELERATION = 0x80;
        const USE_MOUSE_AIM_MODE = 0x100;
    }
}

bitflags! {
    ///
    /// Bitfield of requested services for the next pitstop.
    #[derive(Default)]
    pub struct PitServices: u32 {
        const CHANGE_LEFT_FRONT = 0x01;
        const CHANGE_RIGHT_FRONT = 0x02;
        const CHANGE_LEFT_REAR = 0x04;
        const CHANGE_RIGHT_REAR = 0x08;
        const REFUEL = 0x10;
        const SCREEN_TEAROFF = 0x20;
        const FAST_REPAIR = 0x40;
    }
}

/// Telemetry Value
/// 
#[derive(Debug,Copy,Clone)]
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

/**
 * Action which will be initiated by the "RESET" button
 */
#[derive(Debug, Copy, Clone)]
pub enum ResetAction {
    ENTER,
    EXIT,
    RESET,
}

impl Default for ResetAction {
    fn default() -> Self {
        Self::EXIT
    }
}

/**
 * Current units being displayed
 */
#[derive(Debug, Copy, Clone)]
pub enum Units {
    IMPERIAL,
    METRIC,
}

impl Default for Units {
    fn default() -> Self {
        Self::METRIC
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
    fn latest_buffer(self) -> ValueBuffer {
        let mut latest_tick: i32 = 0;
        let mut buffer: ValueBuffer = self.buffers[0];

        for i in 1..self.n_buffers {
            let b = self.buffers[i as usize];

            if b.ticks > latest_tick {
                buffer = b;
                latest_tick = b.ticks;
            }
        }

        return buffer;
    }

    fn latest_var_buffer(&self, from_loc: *const c_void) -> &[u8] {
        let lb = self.latest_buffer();
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
        let value_buffer = self.latest_var_buffer(from_loc);
        let value_header = self.get_var_header(from_loc);

        Ok(Sample::new(value_header.to_vec(), value_buffer.to_vec()))
    }
}

impl Sample {
    fn new(header: Vec<ValueHeader>, buffer: Vec<u8>) -> Self {
        Sample {
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

    pub fn has(&self, name: &'static str) -> bool {
        match self.header_for(name) {
            Some(_) => true,
            None => false
        }
    }

    pub fn get(&self, name: &'static str) -> Option<Value> {
        match self.header_for(name) {
            None => None,
            Some(vh) => {
               let vs = vh.offset as usize; // Value start
               let vt = Value::from(vh.value_type);
               let ve = vs + vt.size(); // Value end = Value Start + Value Size
                
               let raw_val = &self.buffer[vs..ve];

               let v: Value;

               v = match vt {
                   Value::INT(_) => Value::INT(i32::from_le_bytes( raw_val.try_into().unwrap() )),
                   Value::FLOAT(_) => Value::FLOAT(f32::from_le_bytes( raw_val.try_into().unwrap() )),
                   Value::DOUBLE(_) => Value::DOUBLE(f64::from_le_bytes( raw_val.try_into().unwrap() )),
                   Value::CHAR(_) => Value::CHAR(raw_val[0] as u8),
                   Value::BOOL(_) => Value::BOOL(raw_val[0] > 0),
                   _ => unimplemented!()
               };

               Some(v)
            }
        }
    }
}