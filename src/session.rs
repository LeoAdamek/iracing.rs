mod session {
    pub struct SessionDetails {
        pub track_name: &'static str,               // Track Name
        pub track_id: u32,                          // iRacing Track ID
        pub track_length: f32,                      // Track length (km)
        pub track_display_name: &'static str,       // Track display name
        pub track_display_short_name: &'static str, // Track short display name
        pub track_config_name: &'static str,        // Track layout/configuration name
        pub track_city: &'static str,               // Track Location, City
        pub track_country: &'static str,            // Track Location: Country
        pub track_altitude: f32,                    // Track Altitude (m)
        pub track_latitude: f64,                    // Track Latitude (deg)
        pub track_longitude: f64,                   // Track Longitude (deg)
        pub track_north_offset: f32,                // Track rotation relative to true north (rad)
        pub track_turns: u32,                       // Number of turns
        pub track_pit_speed_limit: f32,             // Pit speed limit (km/h)
        pub track_type: &'static str,               // Track type (Road, Oval, Dirt, DOval)
        pub track_weather: &'static str,            // Track Weather
        pub track_skies: &'static str,              // Sky state
        pub track_surface_temperature: f32,         // Track surface temperature (degC)
        pub track_air_tempearture: f32,             // Track air temperature (degC)
        pub track_air_pressure: f32,                // Track air pressure (Hg)
        pub track_wind_speed: f32,                  // Track wind speed (km/h)
        pub track_wind_direction: f32,              // Track wind direction relative to north (rad)
        pub track_fog: f32,                         // Track fogginess
        pub track_cleanup: i32,                     // Track cleanup
        pub track_dynamic: i32,                     // Track Dynamic
        pub series_id: i32,                         // iRacing series ID
        pub season_id: i32,                         // iRacing season ID
        pub session_id: i32,                        // iRacing session Id
        pub sub_session_id: i32,                    // iRacing subsession (split) ID
        pub league_id: i32,                         // iRacing League ID
        pub official: i8,                           // Official Race
        pub race_week: i32,                         // Race  Week Number
        pub event_type: &'static str,               // Event Type
        pub category: &'static str,                  // Category
        pub sim_mode: &'static str,                          // Sim Mode
        pub team_race: i8,                          // Is Team Race
        pub min_drivers: i8,                        // Minimum drivers per team
        pub max_drivers: i8,                        // Maximum drivers per team
        pub dc_rule_set: str,                       // Driver change rules
    }
}
