use serde::{Deserialize, Serialize};

///
/// Session Details
///
/// Top-level details regarding the current session, including race weekend, session and drivers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDetails {
    #[serde(rename = "WeekendInfo")]
    pub weekend: WeekendInfo, // Race Weekend Info (track, location, series etc.)

    #[serde(rename = "SessionInfo")]
    pub session: SessionInfo,

    #[serde(rename = "DriverInfo")]
    pub drivers: DriverInfo, // Driver information
}

///
/// Details of the race weekend. Including details of the track being raced,
/// the weather, racing series, and the rules in play for the session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WeekendInfo {
    pub track_name: String, // Track Name

    #[serde(rename = "TrackID")]
    pub track_id: u32, // iRacing Track ID

    pub track_length: String,             // Track length (as string of km)
    pub track_display_name: String,       // Track display name
    pub track_display_short_name: String, // Track short display name
    pub track_config_name: String,        // Track layout/configuration name
    pub track_city: String,               // Track Location, City
    pub track_country: String,            // Track Location: Country
    pub track_altitude: String,           // Track Altitude (m)
    pub track_latitude: String,           // Track Latitude (deg)
    pub track_longitude: String,          // Track Longitude (deg)
    pub track_north_offset: String,       // Track rotation relative to true north (rad)

    #[serde(rename = "TrackNumTurns")]
    pub track_turns: u32, // Number of turns

    pub track_pit_speed_limit: String, // Pit speed limit (km/h)
    pub track_type: String,            // Track type (Road, Oval, Dirt, DOval)

    #[serde(rename = "TrackWeatherType")]
    pub track_weather: String, // Track Weather

    pub track_skies: String, // Sky state

    #[serde(rename = "TrackSurfaceTemp")]
    pub track_surface_temperature: String, // Track surface temperature (degC)

    #[serde(rename = "TrackAirTemp")]
    pub track_air_tempearture: String, // Track air temperature (degC)

    pub track_air_pressure: String, // Track air pressure (Hg)

    #[serde(rename = "TrackWindVel")]
    pub track_wind_speed: String, // Track wind speed (km/h)

    #[serde(rename = "TrackWindDir")]
    pub track_wind_direction: String, // Track wind direction relative to north (rad)
    pub track_fog_level: String, // Track fogginess
    pub track_cleanup: i32,      // Track cleanup

    #[serde(rename = "TrackDynamicTrack")]
    pub track_dynamic: i32, // Track Dynamic

    #[serde(rename = "SeriesID")]
    pub series_id: i32, // iRacing series ID

    #[serde(rename = "SeasonID")]
    pub season_id: i32, // iRacing season ID

    #[serde(rename = "SessionID")]
    pub session_id: i32, // iRacing session Id

    #[serde(rename = "SubSessionID")]
    pub sub_session_id: i32, // iRacing subsession (split) ID

    #[serde(rename = "LeagueID")]
    pub league_id: i32, // iRacing League ID
    pub official: i8,       // Official Race
    pub race_week: i32,     // Race  Week Number
    pub event_type: String, // Event Type
    pub category: String,   // Category
    pub sim_mode: String,   // Sim Mode
    pub team_racing: i8,    // Is Team Race
    pub min_drivers: i8,    // Minimum drivers per team
    pub max_drivers: i8,    // Maximum drivers per team

    #[serde(rename = "DCRuleSet")]
    pub dc_rule_set: String, // Driver change rules

    pub qualifier_must_start_race: i8, // Qualifying driver must start race

    #[serde(rename = "NumCarClasses")]
    pub n_classes: u32, // Number of classes in the race
    #[serde(rename = "NumCarTypes")]
    pub n_car_types: u32, // Number of car types eligible for the race

    #[serde(rename = "WeekendOptions")]
    pub options: WeekendOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WeekendOptions {
    #[serde(rename = "NumStarters")]
    pub starters: u32, // Number of cars starting the session

    pub starting_grid: String,
    pub qualify_scoring: String,
    pub course_cautions: String,
    pub standing_start: i8,     // Standing or Rolling start
    pub restarts: String,       // Race restart rules
    pub weather_type: String,   // Weather type
    pub skies: String,          // Skies
    pub wind_direction: String, // Wind direction
    pub wind_speed: String,     // Wind Speed

    #[serde(rename = "WeatherTemp")]
    pub temperature: String, // Temperature

    pub relative_humidity: String, // RH (%)
    pub fog_level: String,
    pub unofficial: i8,          // Inverse of Official
    pub commercial_mode: String, // On if race is being run commercially (e.g. Professional race)
    pub night_mode: String,      // On if race is at night
    pub is_fixed_setup: i8,      // On if car setups are fixed by series rules
    pub strict_laps_checking: String,
    pub has_open_registration: i8, // On if anyone can register, off if registration requires a specific license or invitation.
    pub hardcore_level: i8,        // Hardcoreness
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SessionInfo {
    #[serde(rename = "NumSessions")]
    pub n_sessions: Option<u32>, // Number of sessions (possibly None)
    pub sessions: Vec<Session>, // Sessions
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Session {
    #[serde(rename = "SessionNum")]
    pub session_number: u64, // Session Number

    #[serde(rename = "SessionLaps")]
    laps: serde_yaml::Value, // Laps, may be string or number, use `max_laps()` to determine

    #[serde(rename = "SessionTime")]
    pub time: String, // Time limit

    pub session_type: String,

    #[serde(rename = "SessionTrackRubberState")]
    pub track_rubber_state: String,

    #[serde(rename = "ResultsPositions")]
    pub results: Option<Vec<SessionResult>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SessionResult {
    pub position: i32,
    pub class_position: i32,
    pub car_idx: i32,
    pub lap: i32,
    pub time: f32,
    pub fastest_lap: i32,
    pub fastest_time: f32,
    pub last_time: f32,
    pub laps_led: i32,
    pub laps_complete: i32,
    pub laps_driven: f32,
    pub incidents: i32,
    pub reason_out_id: i32,
    pub reason_out_str: String,
}

///
/// Details of Player driver, and other drivers.Deserialize
///
/// Struct contains player driver information, as well as a vector of
/// other drivers in the session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverInfo {
    #[serde(rename = "DriverCarIdx")]
    pub car_index: usize, // Drivers' Car Index

    #[serde(rename = "DriverHeadPosX")]
    pub head_position_x: f32, // Head Position (X)

    #[serde(rename = "DriverHeadPosY")]
    pub head_position_y: f32, // Head Position (X)

    #[serde(rename = "DriverHeadPosZ")]
    pub head_position_z: f32, // Head Position (Z)

    #[serde(rename = "DriverCarIdleRPM")]
    pub idle_rpm: f32,

    #[serde(rename = "DriverCarRedLine")]
    pub red_line_rpm: f32,

    #[serde(rename = "DriverCarFuelKgPerLtr")]
    pub fuel_density: f32, // Fuel density (kg/litre)

    #[serde(rename = "DriverCarFuelMaxLtr")]
    pub fuel_capacity: f32, // Fuel capacity (litres)

    #[serde(rename = "DriverCarMaxFuelPct")]
    pub fuel_max_fill_percent: f32, // Fuel Fill Percent

    #[serde(rename = "DriverCarSLFirstRPM")]
    pub shift_light_first_rpm: f32, // RPM at which the first shift-indicator light triggers

    #[serde(rename = "DriverCarSLShiftRPM")]
    pub shift_light_shift_rpm: f32, // RPM at which the shift indicator to actually shift triggers

    #[serde(rename = "DriverCarSLLastRPM")]
    pub shift_light_last_rpm: f32, // RPM at which the last shift-indicator light triggers

    #[serde(rename = "DriverCarSLBlinkRPM")]
    pub shift_light_blink_rpm: f32, // RPM at which the shift-indicators blink to remind the driver they really should upshift.

    #[serde(rename = "DriverPitTrkPct")]
    pub pit_track_percent: f32, // ???

    #[serde(rename = "DriverCarEstLapTime")]
    pub estimated_lap_time: f32, // Estimated laptime (s)

    #[serde(rename = "DriverSetupName")]
    pub setup_name: String,

    #[serde(rename = "DriverSetupIsModified")]
    pub setup_is_modified: u8, // Setup is modified from named setup

    #[serde(rename = "DriverSetupPassedTech")]
    pub setup_passed_tech: u8, // Setup has passed tech inspection and is valid to race

    #[serde(rename = "Drivers")]
    pub other_drivers: Vec<Driver>,
}

///
/// Details of all drivers (players) in the session, including the current driver.
///
/// Contains details of the user-profile of the driver, their License class, Safety Rating, and iRating.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Driver {
    #[serde(rename = "CarIdx")]
    pub index: usize,

    pub user_name: String,
    pub abbrev_name: String,
    pub initials: String,

    #[serde(rename = "UserID")]
    pub user_id: i64,

    #[serde(rename = "TeamID")]
    pub team_id: u64,

    pub team_name: String,

    #[serde(rename = "CarNumberRaw")]
    pub car_number: i64,

    pub car_path: String,

    #[serde(rename = "CarClassID")]
    pub car_class_id: u64,

    #[serde(rename = "CarID")]
    pub car_id: u64,

    pub car_screen_name: String,
    pub car_screen_name_short: String,
    pub car_class_short_name: String,

    #[serde(rename = "CarClassRelSpeed")]
    pub car_class_relative_speed: i64,

    pub car_class_license_level: i64,

    #[serde(rename = "CarClassMaxFuelPct")]
    pub car_class_max_fuel_percent: String,

    pub car_class_weight_penalty: String,

    pub car_class_color: String,

    pub i_rating: i64,

    #[serde(rename = "LicLevel")]
    pub license_level: i64,

    #[serde(rename = "LicSubLevel")]
    pub license_sub_level: i64,

    #[serde(rename = "LicString")]
    pub license: String,

    pub is_spectator: i8, // Is Specator?

    #[serde(rename = "CarDesignStr")]
    pub car_design: String,

    #[serde(rename = "CarSponsor_1")]
    pub car_sponsor1: i64,

    #[serde(rename = "CarSponsor_2")]
    pub car_sponsor2: i64,

    pub club_name: Option<String>, // User's club name - Not present for safety car.
    pub division_name: Option<String>, // User's disivision name - Not present for safety car.
}

impl Session {
    ///
    /// Get the maximum number of laps for the session.
    ///
    /// Returns an Some(u64) when there is a maximum number of laps.
    /// Returns None for unlimited laps.
    pub fn max_laps(&self) -> Option<u64> {
        self.laps.as_u64()
    }
}
