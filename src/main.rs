extern crate gtk;
extern crate cairo;

use gtk::prelude::*;

struct State {
    x: f64,
    y: f64,
}

macro_rules! cloning {
    ($($n:ident),+ => $body:expr) => {{
        $( let $n = $n.clone(); )+
        $body
    }}
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

fn main() {
    gtk::init().expect("Failed to initialize GTK.");

    let builder = gtk::Builder::new_from_string(include_str!("layout.glade"));

    get_objects_from_builder!(
        builder,
        window:  gtk::Window,
        drawing: gtk::DrawingArea,
        x_scale: gtk::Scale,
        y_scale: gtk::Scale);

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    drawing.connect_draw(cloning!(
        x_scale, y_scale => move |_,ctx| {
            let st = State {
                x: x_scale.get_value(),
                y: y_scale.get_value(),
            };
            draw(ctx, st)
        }));
    x_scale.connect_value_changed(cloning!(drawing => move |_| drawing.queue_draw()));
    y_scale.connect_value_changed(cloning!(drawing => move |_| drawing.queue_draw()));

    window.show_all();
    gtk::main();
}

const CIRCLE_RADIUS: f64 = 50.0;
fn draw(ctx: &cairo::Context, st: State) -> Inhibit {
    let (ox, oy, ex, ey) = ctx.clip_extents();
    let xt = (st.x + 1.0) / 2.0;
    let yt = (st.y + 1.0) / 2.0;
    let cx = ox * (1.0 - xt) + ex * xt;
    let cy = oy * (1.0 - yt) + ey * yt;

    ctx.set_source_rgb(0.0, 0.0, 0.0);
    ctx.paint();

    ctx.set_source_rgb(0.0, 0.0, 0.75);
    ctx.new_path();
    ctx.move_to(cx, cy - CIRCLE_RADIUS);
    ctx.arc(cx, cy, CIRCLE_RADIUS, 0.0, 2.0 * std::f64::consts::PI);
    ctx.close_path();
    ctx.fill();

    Inhibit(false)
}
