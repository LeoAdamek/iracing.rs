///
/// Track Surface Type

#[derive(Debug, Clone, Copy)]
pub enum TrackSurface {
    NotInWorld,
    Undefined,
    Asphalt(usize),
    Concrete(usize),
    RacingDirt(usize),
    Paint(usize),
    Rumble(usize),
    Grass(usize),
    Dirt(usize),
    Sand,
    Gravel(usize),
    Grasscrete,
    Astroturf,
    Unknown(usize),
}

impl From<i32> for TrackSurface {
    fn from(idx: i32) -> TrackSurface {
        let ix = idx as usize;
        match idx {
            -1 => TrackSurface::NotInWorld,
            0 => TrackSurface::Undefined,
            1..=4 => TrackSurface::Asphalt(ix),
            6 | 7 => TrackSurface::Concrete(ix - 4),
            8 | 9 => TrackSurface::RacingDirt(ix - 7),
            10 | 11 => TrackSurface::Paint(ix - 9),
            12..=15 => TrackSurface::Rumble(ix - 11),
            16..=19 => TrackSurface::Grass(ix - 15),
            20..=23 => TrackSurface::Dirt(ix - 19),
            24 => TrackSurface::Sand,
            25..=28 => TrackSurface::Gravel(ix - 24),
            29 => TrackSurface::Grasscrete,
            30 => TrackSurface::Astroturf,
            _ => TrackSurface::Unknown(ix),
        }
    }
}
