use std::slice::from_raw_parts;
use std::fmt;
use std::ffi::CStr;
use libc::c_char;

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


#[repr(C)]
struct ValueHeader {
    pub value_type: i32, // Value type
    pub offset: i32, // Value offset
    pub count: i32, // Number of values for an array
    pub count_as_time: bool, // ???

    _pad: [u8; 3], // Padding
    _name: [c_char; ValueHeader::MAX_VAR_NAME_LENGTH], // Value name
    _description: [c_char; ValueHeader::MAX_VAR_DESCRIPTION_LENGTH], // Value description
    _unit: [c_char; ValueHeader::MAX_VAR_NAME_LENGTH] // Value units
}

///
/// RawSample represents a raw sample of telemetry data from iRacing
/// either from live telemetry, or from a telemetry file.
#[derive(Debug, Copy, Clone)]
pub struct RawSample {
    pub air_density: f32,                // Air Density (kgm^-3)
    pub air_pressure: f32,               // Barometric pressure (inHg)
    pub air_temperature: f32,            // Air Temperature (deg C)
    pub altitude: f32,                   // Altitude (m)
    pub brake: f32,                      // Brake Input (%)
    pub brake_raw: f32,                  // Brake Input (Raw) (%)
    pub camera_number: i32,              // Camera Number
    pub camera_state: CameraState,       // Current camera state
    pub camera_car_index: i32,           // Car index of camera focus
    pub camera_group: i32,               // Camera group
    pub clutch: f32,                     // Clutch input (%)
    pub cpu_usage_bg: f32,               // Background CPU Usage (%)
    pub dc_driver_so_far: i32,           // Count of drivers who have driven a stint
    pub dc_lap_status: i32,              // Status of driver change lap requirements
    pub display_units: Units,            // Units being displayed
    pub driver_marker: bool,             // Driver Marker
    pub engine_warnings: EngineWarnings, // Current engine warnings
    pub enter_exit_reset: ResetAction,   // Current reset action
    pub fog: f32,                        // Current fog level (%)
    pub frame_rate: f32,                 // Current fps
    pub fuel_level: f32,                 // Fuel Level (l))
    pub fuel_level_perc: f32,            // Fuel level as a percentage (%)
    pub fuel_pressure: f32,              // Fuel Pressure (hPa)
    pub fuel_usage_per_hour: f32,        // Fuel Usage (kg/hour)
    pub gear: i32,                       // Current engaged gear

    // Game state flags
    pub is_disk_logging_active: bool, // Telemetry being logged to disk
    pub is_disk_logging_enabled: bool, // Telemetry enabled
    pub is_car_in_garage: bool,       // Car in Garage?
    pub is_on_track: bool,            // Car on track with plauyer
    pub is_on_track_car: bool,        // Car on track
    pub is_replay_running: bool,      // Replay playing

    // Laps
    pub lap_number: i32,                          // Current lap number
    pub lap_best_lap: i32, // Lap number on which the best lap time was set
    pub lap_best_lap_time: f32, // Best lap time
    pub lap_best_n_lap_lap: i32, // Player last lap in base N average laps
    pub lap_best_n_lap_time: f32, // Player best N average lap time
    pub lap_current_time_est: f32, // Estimated time for current lap completion
    pub lap_delta_to_best: f32, // Delta between current lap and best lap
    pub lap_delta_to_best_change: f32, // Rate of change in lap-to-best delta
    pub lap_detla_to_best_ok: bool, // Lap-to-best delta is valid
    pub lap_delta_to_optimal: f32, // Delta between current lap and optimal lap
    pub lap_delta_to_optimal_change: f32, // Range of change between lap-to-optimal delta
    pub lap_delta_to_optimal_ok: bool, // Lap-to-optimal delta is valid
    pub lap_delta_to_session_best: f32, // Delta between current lap and best lap of session
    pub lap_delta_to_session_best_change: f32, // Rate of change between lap-to-session-best delta
    pub lap_delta_to_session_best_ok: bool,    // Lap-to-session-best delta is valid
    pub lap_delta_to_session_last: f32, // Delta between current lap and last lap of session
    pub lap_delta_to_session_last_change: f32, // Rate of change between lap-to-session-last delta
    pub lap_delta_to_session_last_ok: bool,    // Lap-to-session-last delta is valid
    pub lap_delta_to_session_optimal: f32, // Delta between current lap and optimal lap of session
    pub lap_delta_to_session_optimal_change: f32, // Rate of change between lap-to-session-optimal delta
    pub lap_delta_to_session_optimal_ok: bool,    // Lap-to-session-optimal delta is valid
    pub lap_distance: f32, // Distances travelled from start/finish line this lap (m)
    pub lap_distance_perc: f32, // Lap completion percentage (%)
    pub lap_last_n_lap_seq: i32, // Consecutive clean laps completed for N average#
    pub lap_last_lap_time: f32, // Last lap time
    pub lap_last_n_lap_time: f32, // Average lap time last N laps

    pub latitude: f64,          // Player latitude (deg)
    pub lat_accel: f32,         // Lateral acceleration, including gravity (ms^-2)
    pub longitude: f64,         // Plater Longitude (deg)
    pub lng_accel: f32,         // Longitudinal acceleration, including gravity (ms^-2)
    pub manifold_pressure: f32, // Manifold pressure (bar)
    pub oil_level: f32,         // Oil Level (l)
    pub oil_pressure: f32,      // Oil Pressure
    pub oil_temperature: f32,   // Oil temperature (degC)
    pub on_pit_road: bool,      // Player car on pit road between limiter cones
    pub pitch: f32,             // Vehicle pitch (rad)
    pub pitch_rate: f32,        // Vehicle pitch rate (rads^-1)

    // Pitting
    pub pit_optinal_repairs_remaining: f32, // Pit time remaining for optional repairs
    pub pit_repairs_remaining: f32,         // Pit time remaining for required repairs
    pub pit_services: PitServices,          // Pit Services Enabled
    pub pit_service_fuel_level: f32,        // Current pit refuelling (l)
    pub pit_pressures_left_front: f32,      // Left front tyre pressure (kPa)
    pub pit_pressures_left_rear: f32,       // Left rear tyre pressure (kPa)
    pub pit_pressures_right_front: f32,     // Left front tyre pressure (kPa)
    pub pit_pressures_right_rear: f32,      // Left rear tyre pressure (kPa)

    pub player_car_class_position: i32, // Player car position in class
    pub player_car_position: i32,       // Player car position overall
    pub race_laps: i32,                 // Total laps completed in race

    pub radio_transmit_car: i32, // Car currenty broadcasting on radio
    pub radio_transmit_frequency: i32, // Radio frequency index
    pub radio_transmit_channel: i32, // Radio channel

    pub relative_humidity: f32, // Relative humidity (%)

    pub replay_frame: i32,          // Current frame in replay
    pub replay_frame_end: i32,      // Frame in replay from end
    pub replay_slow_motion: bool,   // Replay in slow-motion
    pub replay_playback_speed: i32, // Replay playback speed relative to real-time
    pub replay_session_number: i32, // Session number of replay
    pub replay_session_time: f64,   // Seconds since replay session start

    pub roll: f32,      // Vehicle roll (rad)
    pub roll_rate: f32, // Vehicle roll rate (rads^-1)
    pub rpm: f32,       // Engine RPM (rev min^-1)
}

bitflags! {
    /**
     * Current warnings / status flags of the engine.
     */
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

/**
 * Action which will be initiated by the "RESET" button
 */
#[derive(Debug, Copy, Clone)]
pub enum ResetAction {
    ENTER,
    EXIT,
    RESET,
}

/**
 * Current units being displayed
 */
#[derive(Debug, Copy, Clone)]
pub enum Units {
    IMPERIAL,
    METRIC,
}

impl ValueHeader {
    ///
    /// Maximum length of a variable name/unit
    const MAX_VAR_NAME_LENGTH: usize = 32;

    ///
    /// Maximum length for a variable description
    const MAX_VAR_DESCRIPTION_LENGTH: usize = 64;

    ///
    /// Convert the name from a c_char[32] to a rust String
    /// Expect that we won't have any encoding issues as the values should always be ASCII
    pub fn name(&self) -> String {
        let name = unsafe { CStr::from_ptr(self._name.as_ptr()) };
        name.to_string_lossy().to_string()
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
            _description: [0; ValueHeader::MAX_VAR_DESCRIPTION_LENGTH]
        }
    }
}

impl fmt::Debug for ValueHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ValueHeader(name=\"{}\", type={}, count={}, offset={})", self.name(), self.value_type, self.count, self.offset)
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


    pub fn telemetry(&mut self, from_loc: *const std::ffi::c_void) -> Result<RawSample, Box<dyn std::error::Error>> {
        let lb = self.latest_buffer();
        let sz = self.buffer_length as usize;

        let loc = from_loc as usize + lb.offset as usize;
        let _data: &[u8] = unsafe { from_raw_parts(loc as *const u8, sz) };
        let n_vars = self.n_vars as usize;

        let header_loc = from_loc as usize + self.header_offset as usize;
        let header_size = n_vars * std::mem::size_of::<ValueHeader>();

        let values = unsafe { from_raw_parts(header_loc as *const ValueHeader, n_vars) };

        println!("{:#?}", values);


        unimplemented!()
    }
}