use std::f64::NAN;
use tau::TAU;

use orbits::{Trajectory, PlanarTrajectory};

pub const PLANET_RADIUS: f64 = 6371.0; // km

#[derive(Debug)]
pub struct State {
    pub eye_lat: f64, // rad
    pub eye_lon: f64, // rad
    pub p_eye_lat: f64, // rad (used during a drag)
    pub p_eye_lon: f64, // rad
    pub scale: f64, // km/px
    pub trajectory: Trajectory,
}

impl Default for State {
    fn default() -> State {
        State {
            eye_lat: TAU / 16.0,
            eye_lon: 0.0,
            p_eye_lat: NAN,
            p_eye_lon: NAN,
            scale: 0.025,
            trajectory: Trajectory::default(),
        }
    }
}

impl Default for PlanarTrajectory {
    fn default() -> Self {
        PlanarTrajectory {
            periapsis: PLANET_RADIUS + 200.0,
            eccentr: 0.0,
            mean_anom0: 0.0,
            mean_anom1: TAU,
        }
    }
}
