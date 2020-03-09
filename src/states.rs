
#[derive(Debug, Copy, Clone)]
pub enum SessionState {
    Invalid(i32),
    GetInCar,
    Warmup,
    ParadeLaps,
    Racing,
    Checkered,
    Cooldown
}

impl From<i32> for SessionState {
    fn from(idx: i32) -> SessionState {
        match idx {
            1 => Self::GetInCar,
            2 => Self::Warmup,
            3 => Self::ParadeLaps,
            4 => Self::Racing,
            5 => Self::Checkered,
            6 => Self::Cooldown,
            _ => Self::Invalid(idx)
        }
    }
}

bitflags! {
    ///
    /// Current warnings / status flags of the player's engine.
    ///
    #[derive(Default)]
    pub struct EngineWarnings: u32 {
        /// Water Temperature too high
        const WATER_TEMPERATURE = 0x01;

        /// Fuel pressure too low (low fuel)
        const FUEL_PRESSURE = 0x02;
        
        /// Oil pressure too low (low oil)
        const OIL_PRESSURE = 0x04;

        /// Engine stalled
        const ENGINE_STALLED = 0x08;

        /// Status: Pit speed limiter is on.
        const PIT_SPEED_LIMITER = 0x10;

        /// Status: rev limiter is active.
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

bitflags! {
    #[derive(Default)]
    pub struct Flags: u32 {
        const CHECKERED_FLAG = 0x01;
        const WHITE_FLAG = 1 << 1;
        const GREEN_FLAG = 0x04;
        const YELLOW_FLAG = 1 << 3;
        const RED_FLAG = 1 << 4;
        const BLUE_FLAG = 1 << 5;
        const DEBRIS = 1 << 6;
        const CROSSED = 1 << 7;
        const YELLOW_WAVING_FLAG = 1 << 8;
        const ONE_LAP_TO_GREEN = 1 << 9;
        const GREEN_HELD = 1 << 10;
        const TEN_LAPS_TO_GO = 1 << 11;
        const FIVE_LAPS_TO_GO = 1 << 12;
        const RANDOM_WAVING = 1 << 13;
        const CAUTION = 1 << 14;
        const CAUTION_WAVING = 1 << 15;
        
        const BLACK_FLAG = 1 << 16;
        const DISQUALIFIED_FLAG = 1 << 17;
        const CAN_SERVICE = 1 << 18;
        const FURLED_FLAG = 1 << 19;
        const REPAIR_FLAG = 1 << 20;

        const START_HIDDEN = 1 << 21;
        const START_READY = 1 << 22;
        const START_SET = 1 << 23;
        const START_GO = 1 << 24;
    }
}

/**
 * Action which will be initiated by the "RESET" button
 */
#[derive(Debug, Copy, Clone)]
pub enum ResetAction {
    Enter,
    Exit,
    Reset,
}

impl Default for ResetAction {
    fn default() -> Self {
        Self::Enter
    }
}

/**
 * Current units being displayed
 */
#[derive(Debug, Copy, Clone)]
pub enum Units {
    Imperial,
    Metric,
}

impl Default for Units {
    fn default() -> Self {
        Self::Metric
    }
}

impl From<i32> for Units {
    fn from(v: i32) -> Units {
        if v > 0 {
            Self::Metric
        } else {
            Self::Imperial 
        }
    }
}