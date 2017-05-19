use tau::TAU;
use na::{Matrix3, Point2, Vector2, Vector3, Rotation3, Unit};

use conics::*;

#[derive(Clone,Copy,Debug,Default)]
pub struct Trajectory {
    pub p: Plane,
    pub t: PlanarTrajectory,
}

#[derive(Clone,Copy,Debug,Default)]
pub struct Plane {
    pub lon_asc_node: f64,
    pub inclination: f64,
    pub arg_peri: f64,
}

impl Plane {
    /// Computes a transformation matrix.
    /// It transforms the x-y plane to the orbital plane
    /// where the x-axis points towards the periapsis.
    pub fn to_matrix(&self) -> Rotation3<f64> {
        Rotation3::from_axis_angle(&Vector3::z_axis(), self.lon_asc_node) *
        Rotation3::from_axis_angle(&Vector3::x_axis(), self.inclination) *
        Rotation3::from_axis_angle(&Vector3::z_axis(), self.arg_peri)
    }
}

#[derive(Clone,Copy,Debug)]
pub struct PlanarTrajectory {
    pub periapsis: f64, // km
    pub eccentr: f64,
    pub mean_anom0: f64,
    pub mean_anom1: f64,
}

impl PlanarTrajectory {
    /// Computes the period of this trajectory given a standard gravitational parameter `mu`.
    pub fn period(&self, mu: f64) -> f64 {
        let r = self.periapsis / (1.0 - self.eccentr);
        TAU * (r.powi(3) / mu).sqrt()
    }

    /// Computes the apoapsis location (if it exists) in this coordinate system.
    pub fn apoapsis(&self) -> Option<Vector2<f64>> {
        if self.eccentr < 1.0 {
            let a = self.periapsis / (1.0 - self.eccentr);
            Some(Vector2::new(self.periapsis - 2.0 * a, 0.0))
        } else {
            None
        }
    }

    /// Computes the periapsis location in this coordinate system.
    pub fn periapsis(&self) -> Vector2<f64> {
        Vector2::new(self.periapsis, 0.0)
    }

    pub fn to_ellipse(&self) -> Ellipse {
        assert!(self.eccentr < 1.0);
        let a = self.periapsis / (1.0 - self.eccentr);
        let b = a * (1.0 - self.eccentr.powi(2)).sqrt();
        Ellipse::Canonical(CanonicalEllipseRepr {
            semi_axes: Vector2::new(a, b),
            center: Point2::new(self.periapsis - a, 0.0),
            rotation: 0.0,
        })
    }
}

const MAX_ITERATIONS: u8 = 20;
const ACCURACY: f64 = 1e-15;
pub fn approx_inv_kepler(ecc: f64, mean_anom: f64) -> f64 {
    assert!(-TAU <= mean_anom && mean_anom <= TAU,
            "{:?} must be in the range [-TAU,TAU]",
            mean_anom);

    let mut ec_an = if ecc < 0.8 {
        mean_anom
    } else {
        mean_anom.signum() * TAU / 2.0
    };
    for _ in 0..MAX_ITERATIONS {
        let f = ec_an - ecc * ec_an.sin() - mean_anom;
        if f.abs() < ACCURACY {
            return ec_an;
        }
        let prev_ec_an = ec_an;
        ec_an -= f / (1.0 - ecc * ec_an.cos());
        if prev_ec_an == ec_an {
            return ec_an;
        }
    }
    panic!("newton's failed to converge after {:?} iterations: \
            approx_inv_kepler({:?}, {:?})",
           MAX_ITERATIONS,
           ecc,
           mean_anom);
}

pub fn kepler(ecc: f64, ecc_anom: f64) -> f64 {
    ecc_anom - ecc * ecc_anom.sin()
}
