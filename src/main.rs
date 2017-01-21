extern crate gtk;
extern crate cairo;
extern crate tau;
#[macro_use]
extern crate lazy_static;

use std::sync::Mutex;
use gtk::prelude::*;
use tau::TAU;

#[derive(Clone,Debug)]
struct State {
    eye_lat: f64, // rad
    eye_lon: f64, // rad
    p_eye_lat: f64, // rad (used during a drag)
    p_eye_lon: f64, // rad
    scale: f64, // km/px
    orbit: Orbit,
}

#[derive(Clone,Debug)]
enum Orbit {
    Equitorial(PlanarPath),
}

#[derive(Clone,Debug)]
enum PlanarPath {
    Circle(f64),
}

const DRAG_TURN_RATE: f64 = 0.01; // rad/px

impl Default for State {
    fn default() -> State {
        State {
            eye_lat: 0.0,
            eye_lon: 0.0,
            p_eye_lat: std::f64::NAN,
            p_eye_lon: std::f64::NAN,
            scale: 0.025,
            orbit: Orbit::Equitorial(PlanarPath::Circle(6871.0)),
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

macro_rules! connect_spinbutton_state {
    ($drawing:ident; $scale:ident => $state:ident . $field:ident <- $conv:expr) => {
        $scale.connect_value_changed(cloning!($drawing, $scale => move |_| {
            let $scale = $scale.get_value();
            $state.lock().unwrap().$field = $conv;
            $drawing.queue_draw();
        }));
        let $scale = $scale.get_value();
        $state.lock().unwrap().$field = $conv;
    }
}

lazy_static! {
    static ref STATE: Mutex<State> = Mutex::new(Default::default());
}

fn main() {
    gtk::init().expect("Failed to initialize GTK.");

    let builder = gtk::Builder::new_from_string(include_str!("layout.glade"));

    get_objects_from_builder!(builder,
                              window: gtk::Window,
                              drawing: gtk::DrawingArea,
                              r_entry: gtk::SpinButton);

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    drawing.connect_draw(|_, ctx| draw(ctx, &*STATE.lock().unwrap()));
    connect_spinbutton_state!(
        drawing;
        r_entry =>
        STATE.orbit <- Orbit::Equitorial(PlanarPath::Circle(r_entry))
    );

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
        state.p_eye_lat = std::f64::NAN;
        state.p_eye_lon = std::f64::NAN;
    });

    window.show_all();
    gtk::main();
}

fn limit(x: f64, min: f64, max: f64) -> f64 {
    x.min(max).max(min)
}

const PLANET_RADIUS: f64 = 6371.0; // km
const AXIS_LENGTH: f64 = PLANET_RADIUS + 1000.0; // km
fn draw(ctx: &cairo::Context, st: &State) -> Inhibit {
    ctx.set_antialias(cairo::Antialias::Best);
    ctx.set_fill_rule(cairo::FillRule::Winding);
    let (ox, oy, ex, ey) = ctx.clip_extents();
    // println!("{:?}", (st.eye_lat / TAU));

    ctx.translate((ox + ex) / 2.0, (oy + ey) / 2.0);
    ctx.scale(st.scale, -st.scale);
    // the center of our canvas is at the origin and y-axis points up
    let (_, _, ex, ey) = ctx.clip_extents();

    // setup some clips
    ctx.new_path();
    ctx.arc(0.0, 0.0, PLANET_RADIUS, 0.0, TAU);
    let planet_path = ctx.copy_path();
    ctx.new_path();
    ctx.arc_negative(0.0, 0.0, PLANET_RADIUS, TAU, 0.0);
    let planet_path_inv = ctx.copy_path();
    ctx.new_path();
    ctx.rectangle(-ex, ey, 2.0 * ex, -ey);
    let upper_half_path_inv = ctx.copy_path();
    ctx.new_path();
    ctx.rectangle(-ex, 0.0, 2.0 * ex, -ey);
    let lower_half_path_inv = ctx.copy_path();
    ctx.new_path();
    ctx.rectangle(-ex, ey, 2.0 * ex, -2.0 * ey);
    let everything_path_inv = ctx.copy_path();

    let render_plane =
        |plane_lat: f64, mk_plane_path: &Fn() -> (), do_plane_render: &Fn() -> ()| {
            let delta_lat = plane_lat - st.eye_lat;
            let sin_dlat = delta_lat.sin();
            if sin_dlat != 0.0 {
                // inside clip
                ctx.new_path();
                draw_ellipse_arc(ctx,
                                 0.0,
                                 0.0,
                                 0.0,
                                 PLANET_RADIUS,
                                 PLANET_RADIUS * sin_dlat.abs(),
                                 0.0,
                                 TAU);
                let inside_planet_path = ctx.copy_path();

                // setup
                ctx.push_group();
                ctx.new_path();
                ctx.scale(1.0, sin_dlat);
                mk_plane_path();
                ctx.save();
                ctx.identity_matrix();
                do_plane_render();
                ctx.restore();
                ctx.pop_group_to_source();

                // things behind the planet are at 1/3 alpha; things inside are at 1/5 alpha

                // internal
                ctx.save();
                ctx.append_path(&inside_planet_path);
                ctx.clip();
                ctx.paint_with_alpha(1.0 / 6.0);
                ctx.restore();

                // other parts
                let draw_parts_in_planet = |front_half_path_inv: &cairo::Path,
                                            back_half_path_inv: &cairo::Path| {
                    // front
                    ctx.save();
                    ctx.append_path(&inside_planet_path);
                    ctx.append_path(&front_half_path_inv);
                    ctx.clip();
                    ctx.append_path(&front_half_path_inv);
                    ctx.clip();
                    ctx.paint();
                    ctx.restore();
                    // back occluded
                    ctx.save();
                    ctx.append_path(&inside_planet_path);
                    ctx.append_path(&planet_path_inv);
                    ctx.clip();
                    ctx.append_path(&back_half_path_inv);
                    ctx.clip();
                    ctx.paint_with_alpha(1.0 / 4.0);
                    ctx.restore();
                    // back outside
                    ctx.save();
                    ctx.append_path(&planet_path);
                    ctx.append_path(&back_half_path_inv);
                    ctx.clip();
                    ctx.append_path(&back_half_path_inv);
                    ctx.clip();
                    ctx.paint();
                    ctx.restore();
                };
                match delta_lat.tan().partial_cmp(&0.0).unwrap() {
                    std::cmp::Ordering::Equal => {
                        ctx.save();
                        ctx.append_path(&planet_path);
                        ctx.append_path(&everything_path_inv);
                        ctx.clip();
                        ctx.paint();
                        ctx.restore();
                    }
                    std::cmp::Ordering::Greater => {
                        draw_parts_in_planet(&upper_half_path_inv, &lower_half_path_inv)
                    }
                    std::cmp::Ordering::Less => {
                        draw_parts_in_planet(&lower_half_path_inv, &upper_half_path_inv)
                    }
                }
            }
        };

    // clear
    ctx.set_source_rgb(0.0, 0.0, 0.0);
    ctx.paint();
    ctx.new_path();

    // Planet
    ctx.arc(0.0, 0.0, PLANET_RADIUS, 0.0, TAU);
    ctx.close_path();
    ctx.set_source_rgb(0.0, 0.0, 0.75);
    ctx.fill();

    // Axis
    render_plane(TAU / 4.0,
                 &|| {
                     ctx.move_to(0.0, AXIS_LENGTH);
                     ctx.line_to(0.0, -AXIS_LENGTH);
                 },
                 &|| {
                     ctx.set_line_width(4.0);
                     ctx.set_line_cap(cairo::LineCap::Round);
                     ctx.set_source_rgb(0.0, 0.75, 0.75);
                     ctx.stroke();
                 });

    // hemisphere lines
    let hemisphere_renderer = || {
        ctx.set_line_width(4.0);
        ctx.set_source_rgb(0.0, 1.0, 0.0);
        ctx.stroke();
    };
    render_plane(TAU / 4.0,
                 &|| {
                     ctx.append_path(&planet_path);
                 },
                 &hemisphere_renderer);
    render_plane(0.0,
                 &|| {
                     ctx.append_path(&planet_path);
                 },
                 &hemisphere_renderer);

    // Orbit
    let Orbit::Equitorial(PlanarPath::Circle(r)) = st.orbit;
    render_plane(0.0,
                 &|| {
                     ctx.arc(0.0, 0.0, r, 0.0, TAU);
                 },
                 &|| {
                     ctx.set_source_rgb(1.0, 0.0, 0.0);
                     ctx.set_line_width(5.0);
                     ctx.stroke();
                 });

    Inhibit(false)
}

fn draw_ellipse_arc(ctx: &cairo::Context,
                    cx: f64,
                    cy: f64,
                    theta: f64,
                    a: f64,
                    b: f64,
                    eta1: f64,
                    eta2: f64)
                    -> () {
    ctx.save();
    ctx.translate(cx, cy);
    ctx.rotate(theta);
    ctx.scale(a, b);
    ctx.arc(0.0, 0.0, 1.0, eta1, eta2);
    ctx.restore();
}
