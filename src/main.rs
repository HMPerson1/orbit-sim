#![feature(unboxed_closures,fn_traits)]

#[macro_use]
extern crate gtk;
extern crate cairo;
extern crate tau;
extern crate nalgebra as na;

use std::rc::Rc;

mod orbits;
mod conics;
mod gui;

fn main() -> () {
    // use rand::distributions::{IndependentSample, Range};
    // use tau::TAU;
    // let range = Range::new(-TAU, TAU);
    // let mut rng = rand::thread_rng();
    // for e in 0..100 {
    //     for _ in 0..2000 {
    //         let m = range.ind_sample(&mut rng);
    //         let ea = orbits::approx_inv_kepler(e as f64 / 100.0, m);
    //         // println!("m: {:?} ({:?}), ea: :{:?}", m, orbits::kepler(e, ea), ea);
    //     }
    // }
    gui::main(Rc::default());
}

// let e1 = Ellipse::Canonical(CanonicalEllipseRepr {
//     semi_axes: Vector2::new(1.0, 3.0),
//     center: Point2::new(3.0, 1.0),
//     rotation: 1.0 * TAU / 3.0,
// });
// println!("{:?}",
//          Ellipse::Implicit(e1.clone().to_implicit()).to_canonical());
// let t = Affine2::rotate(TAU / 6.0);
// let e2 = e1.transform(t).to_canonical();
// println!("{:?}", e2);
// println!("{:?}",
//          ImplicitConicSectionRepr(0.0, 1.0, 0.0, 0.0, 0.0, -1.0).to_canonical());
