use cairo;
use math::*;
use conics::*;
use tau::TAU;
use na::{Matrix2, Matrix3, Rotation3, Vector3, Rotation};
use {PLANET_RADIUS, State, PlanarTrajectory, Trajectory, Plane};

struct RenderCommon<'a> {
    ctx: &'a cairo::Context,
    proj_mat: Matrix3<f64>,
    planet_path: cairo::Path,
    planet_path_inv: cairo::Path,
    everything_path_inv: cairo::Path,
}

const AXIS_LENGTH: f64 = PLANET_RADIUS + 1000.0; // km
pub fn draw(ctx: &cairo::Context, st: &State) {
    ctx.set_antialias(cairo::Antialias::Best);
    ctx.set_fill_rule(cairo::FillRule::Winding);
    let (ox, oy, ex, ey) = ctx.clip_extents();
    // println!("{:?}", (st.eye_lat / TAU));

    ctx.translate((ox + ex) / 2.0, (oy + ey) / 2.0);
    ctx.scale(st.scale, -st.scale);
    // the center of our canvas is at the origin and y-axis points up
    let (_, _, ex, ey) = ctx.clip_extents();

    // converts space coordinates to screen coordinates via a rotation
    let proj_mat = *Rotation3::new(Vector3::new(st.eye_lat - TAU / 4.0, 0.0, 0.0))
        .prepend_rotation(&Vector3::new(0.0, 0.0, st.eye_lon - TAU / 4.0))
        .submatrix();

    // setup some clips
    ctx.new_path();
    ctx.arc(0.0, 0.0, PLANET_RADIUS, 0.0, TAU);
    let planet_path = ctx.copy_path();
    ctx.new_path();
    ctx.arc_negative(0.0, 0.0, PLANET_RADIUS, TAU, 0.0);
    let planet_path_inv = ctx.copy_path();
    ctx.new_path();
    ctx.rectangle(-ex, ey, 2.0 * ex, -2.0 * ey);
    let everything_path_inv = ctx.copy_path();

    let rc = RenderCommon {
        ctx: ctx,
        proj_mat: proj_mat,
        planet_path: planet_path,
        planet_path_inv: planet_path_inv,
        everything_path_inv: everything_path_inv,
    };

    // clear
    ctx.set_source_rgb(0.0, 0.0, 0.0);
    ctx.paint();
    ctx.new_path();

    // Planet
    ctx.append_path(&rc.planet_path);
    ctx.set_source_rgb(0.0, 0.0, 0.75);
    ctx.fill();

    // hemisphere lines
    // sorta abusing `render_trajectory`
    let great_circle = PlanarTrajectory {
        arg_peri: 0.0,
        periapsis: PLANET_RADIUS,
        eccentr: 0.0,
    };
    let hemisphere_renderer = || {
        ctx.set_line_width(4.0);
        ctx.set_source_rgb(0.0, 1.0, 0.0);
        ctx.stroke();
    };
    render_trajctory(&rc,
                     Trajectory {
                         p: Plane {
                             lon_asc_node: 0.0,
                             inclination: 0.0,
                         },
                         t: great_circle,
                     },
                     &hemisphere_renderer);
    render_trajctory(&rc,
                     Trajectory {
                         p: Plane {
                             lon_asc_node: 0.0,
                             inclination: TAU / 4.0,
                         },
                         t: great_circle,
                     },
                     &hemisphere_renderer);
    render_trajctory(&rc,
                     Trajectory {
                         p: Plane {
                             lon_asc_node: TAU / 4.0,
                             inclination: TAU / 4.0,
                         },
                         t: great_circle,
                     },
                     &hemisphere_renderer);

    // actual trajectory
    render_trajctory(&rc,
                     st.trajectory,
                     &|| {
                         ctx.set_source_rgb(1.0, 0.0, 0.0);
                         ctx.set_line_width(5.0);
                         ctx.stroke();
                     });
}

fn render_trajctory<F: FnOnce() -> ()>(rc: &RenderCommon,
                                       traj: Trajectory,
                                       renderer: F) {
    let ctx = rc.ctx;
    let Trajectory { p: Plane { lon_asc_node: l, inclination: i },
                     t: PlanarTrajectory { arg_peri: o, periapsis: pe, eccentr: ec } } = traj;
    let tm = *Rotation3::new(Vector3::new(0.0, 0.0, l))
        .prepend_rotation(&Vector3::new(i, 0.0, 0.0))
        .prepend_rotation(&Vector3::new(0.0, 0.0, o))
        .submatrix();
    let m = rc.proj_mat * tm;
    let m2 = Matrix2::new(m.m11, m.m12, m.m21, m.m22);
    let aff = Affine::new(m2, Zero::zero());
    let traj = Ellipse::from_orbital(pe, ec).transform(aff);

    let planet_in_plane = Ellipse::from_orbital(PLANET_RADIUS, 0.0).transform(aff);
    ctx.new_path();
    draw_ellipse_arc(ctx, planet_in_plane, 0.0, TAU);
    let inside_planet_path = ctx.copy_path();

    ctx.push_group();
    ctx.new_path();
    draw_ellipse_arc(ctx, traj, 0.0, TAU);
    ctx.save();
    ctx.identity_matrix();
    renderer();
    ctx.restore();
    ctx.pop_group_to_source();

    // internal
    ctx.save();
    ctx.append_path(&inside_planet_path);
    ctx.clip();
    ctx.paint_with_alpha(1.0 / 3.0);
    ctx.restore();

    // FIXME behind stuff should be behind stuff
    ctx.save();
    ctx.append_path(&rc.everything_path_inv);
    ctx.append_path(&inside_planet_path);
    ctx.clip();
    ctx.paint();
    ctx.restore();
}

fn draw_ellipse_arc(ctx: &cairo::Context, ellipse: Ellipse, eta1: f64, eta2: f64) -> () {
    let el = ellipse.to_canonical();
    ctx.save();
    ctx.translate(el.center.x, el.center.y);
    ctx.rotate(el.rotation);
    // println!("ellipse: {:?}", el);
    if el.semi_axes.x != 0.0 && el.semi_axes.y != 0.0 {
        ctx.scale(el.semi_axes.x, el.semi_axes.y);
        ctx.arc(0.0, 0.0, 1.0, eta1, eta2);
    } else {
        // println!("line ellipse: {:?}", el);
    }
    ctx.restore();
}
