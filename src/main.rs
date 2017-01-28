#![feature(slice_patterns)]
mod math;
mod conics;
mod render;

extern crate num_traits;
extern crate gtk;
extern crate cairo;
extern crate tau;
extern crate nalgebra as na;
#[macro_use]
extern crate lazy_static;

use std::sync::Mutex;
use gtk::prelude::*;
use std::f64::NAN;
use tau::TAU;

#[derive(Clone,Debug)]
pub struct State {
    eye_lat: f64, // rad
    eye_lon: f64, // rad
    p_eye_lat: f64, // rad (used during a drag)
    p_eye_lon: f64, // rad
    scale: f64, // km/px
    trajectory: Trajectory,
}

#[derive(Clone,Copy,Debug)]
struct Trajectory {
    p: Plane,
    t: PlanarTrajectory,
}

#[derive(Clone,Copy,Debug)]
struct Plane {
    lon_asc_node: f64,
    inclination: f64,
}

#[derive(Clone,Copy,Debug)]
struct PlanarTrajectory {
    arg_peri: f64,
    periapsis: f64, // km
    eccentr: f64,
}

const PLANET_RADIUS: f64 = 6371.0; // km
const DRAG_TURN_RATE: f64 = 0.01; // rad/px

impl Default for State {
    fn default() -> State {
        State {
            eye_lat: TAU / 16.0,
            eye_lon: 0.0,
            p_eye_lat: NAN,
            p_eye_lon: NAN,
            scale: 0.025,
            trajectory: Trajectory {
                p: Plane {
                    lon_asc_node: 0.0,
                    inclination: 0.0,
                },
                t: PlanarTrajectory {
                    arg_peri: 0.0,
                    periapsis: PLANET_RADIUS + 200.0,
                    eccentr: 0.0,
                },
            },
        }
    }
}

macro_rules! get_objects_from_builder {
    ($b:ident, $($n:ident : $t:ty),*) => {
        $(
            let $n : $t = $b.get_object(stringify!($n))
                .expect(concat!("Failed to get `", stringify!($n), "`",
                                " from `", stringify!($b), "`"));
        )*
    }
}

macro_rules! cloning {
    ($($n:ident),+ => $body:expr) => {{
        $( let $n = $n.clone(); )+
        $body
    }}
}

macro_rules! setup_spinbutton {
    ($drawing:ident,
     $min:tt to $max:tt by $incr:tt;
     $spin_btn:ident -> $state:ident $(.$field:ident)*
    ) => {
        $spin_btn.set_range($min, $max);
        $spin_btn.set_increments($incr, 0.0);
        setup_spinbutton!($drawing; $spin_btn -> $state$(.$field)*);
    };
    ($drawing:ident;
     $spin_btn:ident -> $state:ident $(.$field:ident)*
    ) => {
        $spin_btn.set_value($state.lock().unwrap()$(.$field)*);
        $spin_btn.connect_value_changed(cloning!($drawing, $spin_btn => move |_| {
            $state.lock().unwrap()$(.$field)* = $spin_btn.get_value();
            $drawing.queue_draw();
        }));
    }
}

lazy_static! {
    static ref STATE: Mutex<State> = Mutex::new(State::default());
}

fn main() {
    gtk::init().expect("Failed to initialize GTK.");

    let builder = gtk::Builder::new_from_string(include_str!("layout.glade"));

    get_objects_from_builder!(builder,
                              window: gtk::Window,
                              drawing: gtk::DrawingArea,
                              pe_entry: gtk::SpinButton,
                              ec_entry: gtk::SpinButton,
                              ar_entry: gtk::SpinButton,
                              in_entry: gtk::SpinButton,
                              an_entry: gtk::SpinButton);

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    drawing.connect_draw(|_, ctx| {
        render::draw(ctx, &*STATE.lock().unwrap());
        Inhibit(false)
    });

    setup_spinbutton!(drawing;
                      pe_entry -> STATE.trajectory.t.periapsis);
    setup_spinbutton!(drawing;
                      ec_entry -> STATE.trajectory.t.eccentr);
    setup_spinbutton!(drawing, (-TAU) to (TAU) by (TAU/50.0);
                      ar_entry -> STATE.trajectory.t.arg_peri);
    setup_spinbutton!(drawing, (-TAU) to (TAU) by (TAU/50.0);
                      in_entry -> STATE.trajectory.p.inclination);
    setup_spinbutton!(drawing, (-TAU) to (TAU) by (TAU/50.0);
                      an_entry -> STATE.trajectory.p.lon_asc_node);

    fn limit(x: f64, min: f64, max: f64) -> f64 {
        x.min(max).max(min)
    }

    let gest_drag = gtk::GestureDrag::new(&drawing);
    gest_drag.connect_drag_begin(|_, _, _| {
        let ref mut state = *STATE.lock().unwrap();
        state.p_eye_lat = state.eye_lat;
        state.p_eye_lon = state.eye_lon;
    });
    gest_drag.connect_drag_update(cloning!(drawing => move |_,dx,dy| {
        let ref mut state = *STATE.lock().unwrap();
        state.eye_lat = limit(state.p_eye_lat + dy * DRAG_TURN_RATE, -TAU/4.0, TAU/4.0);
        state.eye_lon = state.p_eye_lon + dx * DRAG_TURN_RATE;
        drawing.queue_draw();
    }));
    gest_drag.connect_drag_end(move |_, _, _| {
        let ref mut state = *STATE.lock().unwrap();
        state.p_eye_lat = NAN;
        state.p_eye_lon = NAN;
    });

    window.show_all();
    gtk::main();
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

// let render_plane =
//     |plane_lat: f64, mk_plane_path: &Fn() -> (), do_plane_render: &Fn() -> ()| {
//         let delta_lat = plane_lat - st.eye_lat;
//         let sin_dlat = delta_lat.sin();
//         if sin_dlat != 0.0 {
//             // inside clip
//             ctx.new_path();
//             draw_ellipse_arc(ctx,
//                              0.0,
//                              0.0,
//                              0.0,
//                              PLANET_RADIUS,
//                              PLANET_RADIUS * sin_dlat.abs(),
//                              0.0,
//                              TAU);
//             let inside_planet_path = ctx.copy_path();

//             // setup
//             ctx.push_group();
//             ctx.new_path();
//             ctx.scale(1.0, sin_dlat);
//             mk_plane_path();
//             ctx.save();
//             ctx.identity_matrix();
//             do_plane_render();
//             ctx.restore();
//             ctx.pop_group_to_source();

//             // things behind the planet are at 1/4 alpha; things inside are at 1/6 alpha

//             // internal
//             ctx.save();
//             ctx.append_path(&inside_planet_path);
//             ctx.clip();
//             ctx.paint_with_alpha(1.0 / 6.0);
//             ctx.restore();

//             // other parts
//             let draw_parts_in_planet = |front_half_path_inv: &cairo::Path,
//                                         back_half_path_inv: &cairo::Path| {
//                 // front
//                 ctx.save();
//                 ctx.append_path(&inside_planet_path);
//                 ctx.append_path(&front_half_path_inv);
//                 ctx.clip();
//                 ctx.append_path(&front_half_path_inv);
//                 ctx.clip();
//                 ctx.paint();
//                 ctx.restore();
//                 // back occluded
//                 ctx.save();
//                 ctx.append_path(&inside_planet_path);
//                 ctx.append_path(&planet_path_inv);
//                 ctx.clip();
//                 ctx.append_path(&back_half_path_inv);
//                 ctx.clip();
//                 ctx.paint_with_alpha(1.0 / 4.0);
//                 ctx.restore();
//                 // back outside
//                 ctx.save();
//                 ctx.append_path(&planet_path);
//                 ctx.append_path(&back_half_path_inv);
//                 ctx.clip();
//                 ctx.append_path(&back_half_path_inv);
//                 ctx.clip();
//                 ctx.paint();
//                 ctx.restore();
//             };
//             match delta_lat.tan().partial_cmp(&0.0).unwrap() {
//                 std::cmp::Ordering::Equal => {
//                     ctx.save();
//                     ctx.append_path(&planet_path);
//                     ctx.append_path(&everything_path_inv);
//                     ctx.clip();
//                     ctx.paint();
//                     ctx.restore();
//                 }
//                 std::cmp::Ordering::Greater => {
//                     draw_parts_in_planet(&upper_half_path_inv, &lower_half_path_inv)
//                 }
//                 std::cmp::Ordering::Less => {
//                     draw_parts_in_planet(&lower_half_path_inv, &upper_half_path_inv)
//                 }
//             }
//         }
//     };
