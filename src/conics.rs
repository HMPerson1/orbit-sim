use math::*;
use na::{Matrix3, Point2, Vector2, Inverse, Transpose};

#[derive(Clone,Copy,Debug)]
pub enum Ellipse {
    Canonical(CanonicalEllipseRepr),
    Implicit(ImplicitConicSectionRepr),
}

impl Ellipse {
    pub fn from_orbital(periapsis: f64, eccentricity: f64) -> Ellipse {
        let a = periapsis / (1.0 - eccentricity);
        let b = a * (1.0 - eccentricity.powi(2)).sqrt();
        Ellipse::Canonical(CanonicalEllipseRepr {
            semi_axes: Vector2::new(a, b),
            center: Point2::new(periapsis - a, 0.0),
            rotation: 0.0,
        })
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn to_canonical(self) -> CanonicalEllipseRepr {
        match self {
            Ellipse::Canonical(x) => x,
            Ellipse::Implicit(x)  => x.to_canonical(),
        }
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn to_implicit(self) -> ImplicitConicSectionRepr {
        match self {
            Ellipse::Canonical(x) => x.to_implicit(),
            Ellipse::Implicit(x)  => x,
        }
    }

    pub fn transform(self, a: Affine2<f64>) -> Ellipse {
        match a.inverse() {
            Some(a) => {
                let m = self.to_implicit().to_matrix();
                let n = a.transpose() * m * a;
                Ellipse::Implicit(ImplicitConicSectionRepr::from_matrix(n))
            }
            None => {
                // TODO: do better
                Ellipse::Canonical(CanonicalEllipseRepr {
                    semi_axes: Vector2::new(0.0, 0.0),
                    center: Point2::new(0.0, 0.0),
                    rotation: 0.0,
                })
            }
        }
    }
}

#[derive(Clone,Copy,Debug)]
pub struct ImplicitConicSectionRepr(pub f64, pub f64, pub f64, pub f64, pub f64, pub f64);

impl ImplicitConicSectionRepr {
    // TODO: Make this work for other conic sections
    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn to_canonical(self) -> CanonicalEllipseRepr {
        let ImplicitConicSectionRepr(pa, pb, pc, pd, pe, pf) = self;
        let discr = pb*pb - 4.0*pa*pc;
        let ec = Point2::new(2.0*pc*pd - pb*pe, 2.0*pa*pe - pb*pd) / discr;
        let tmp1 = 2.0*(pa*pe*pe + pc*pd*pd - pb*pd*pe + discr*pf);
        let tmp2 = ((pa-pc).powi(2) + pb*pb).sqrt();
        let ea = -Vector2::new((tmp1*(pa+pc+tmp2)).sqrt(), (tmp1*(pa+pc-tmp2)).sqrt()) / discr;
        let theta = (-pb).atan2(pc-pa)/2.0;
        let theta = if theta.is_nan() { 0.0 } else { theta };
        CanonicalEllipseRepr {
            semi_axes: ea,
            center: ec,
            rotation: theta,
        }
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn to_matrix(self) -> Matrix3<f64> {
        let ImplicitConicSectionRepr(pa, pb, pc, pd, pe, pf) = self;
        Matrix3::new(  pa  , pb/2.0, pd/2.0,
                     pb/2.0,   pc  , pe/2.0,
                     pd/2.0, pe/2.0,   pf  )
    }

    fn from_matrix(m: Matrix3<f64>) -> Self {
        ImplicitConicSectionRepr(m.m11,
                                 m.m12 + m.m21,
                                 m.m22,
                                 m.m13 + m.m31,
                                 m.m23 + m.m32,
                                 m.m33)
    }
}

#[derive(Clone,Copy,Debug)]
pub struct CanonicalEllipseRepr {
    pub semi_axes: Vector2<f64>,
    pub center: Point2<f64>,
    pub rotation: f64,
}

impl CanonicalEllipseRepr {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn to_implicit(self) -> ImplicitConicSectionRepr {
        let CanonicalEllipseRepr { semi_axes: ea, center: ec, rotation } = self;
        let (sint, cost) = rotation.sin_cos();
        let pa = (ea.x*sint).powi(2) + (ea.y*cost).powi(2);
        let pc = (ea.x*cost).powi(2) + (ea.y*sint).powi(2);
        let pb = 2.0*(ea.y*ea.y - ea.x*ea.x)*sint*cost;
        let pd = -2.0*pa*ec.x - pb*ec.y;
        let pe = -pb*ec.x - 2.0*pc*ec.y;
        let pf = pa*ec.x*ec.x + pb*ec.x*ec.y + pc*ec.y*ec.y - ea.x*ea.x*ea.y*ea.y;
        ImplicitConicSectionRepr(pa, pb, pc, pd, pe, pf)
    }
}
