use std::rc::Rc;
use std::cell::RefCell;
use std::f64::NAN;
use tau::TAU;
use gtk;
use gtk::prelude::*;

use gui::common::*;
use gui::render;

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
        let state = $state.borrow();
        $spin_btn.set_value(state$(.$field)*);
        drop(state);
        $spin_btn.connect_value_changed(cloning!($state, $drawing, $spin_btn => move |_| {
            let mut state = $state.borrow_mut();
            state$(.$field)* = $spin_btn.get_value();
            drop(state);
            $drawing.queue_draw();
        }));
    }
}

const DRAG_TURN_RATE: f64 = 0.01; // rad/px

pub fn main(state: Rc<RefCell<State>>) {
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
                              // ma0_entry: gtk::SpinButton,
                              // ma1_entry: gtk::SpinButton);

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    drawing.connect_draw(cloning!(state => move |_, ctx| {
        let state = state.borrow();
        render::draw(ctx, &*state);
        drop(state);
        Inhibit(false)
    }));

    setup_spinbutton!(drawing;
                      pe_entry -> state.trajectory.t.periapsis);
    setup_spinbutton!(drawing;
                      ec_entry -> state.trajectory.t.eccentr);
    setup_spinbutton!(drawing, (-TAU) to (TAU) by (TAU/60.0);
                      ar_entry -> state.trajectory.p.arg_peri);
    setup_spinbutton!(drawing, (-TAU) to (TAU) by (TAU/60.0);
                      in_entry -> state.trajectory.p.inclination);
    setup_spinbutton!(drawing, (-TAU) to (TAU) by (TAU/60.0);
                      an_entry -> state.trajectory.p.lon_asc_node);
    // setup_spinbutton!(drawing, (-TAU) to (TAU) by (TAU/60.0);
    //                   ma0_entry -> state.trajectory.p.inclination);
    // setup_spinbutton!(drawing, (-TAU) to (TAU) by (TAU/60.0);
    //                   ma1_entry -> state.trajectory.p.lon_asc_node);

    let gest_drag = gtk::GestureDrag::new(&drawing);
    gest_drag.connect_drag_begin(cloning!(state => move |_, _, _| {
        let mut state = state.borrow_mut();
        state.p_eye_lat = state.eye_lat;
        state.p_eye_lon = state.eye_lon;
        drop(state);
    }));
    gest_drag.connect_drag_update(cloning!(state, drawing => move |_,dx,dy| {
        let mut state = state.borrow_mut();
        state.eye_lat = clamp(state.p_eye_lat + dy * DRAG_TURN_RATE, -TAU/4.0, TAU/4.0);
        state.eye_lon = state.p_eye_lon + dx * DRAG_TURN_RATE;
        drop(state);
        drawing.queue_draw();
    }));
    gest_drag.connect_drag_end(cloning!(state => move |_, _, _| {
        let mut state = state.borrow_mut();
        state.p_eye_lat = NAN;
        state.p_eye_lon = NAN;
        drop(state);
    }));

    window.show_all();
    gtk::main();

    fn clamp(x: f64, min: f64, max: f64) -> f64 {
        x.min(max).max(min)
    }
}
