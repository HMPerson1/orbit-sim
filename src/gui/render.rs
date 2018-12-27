use cairo;
use na::{Affine2, Matrix3, Rotation2, Rotation3, Transform2, Vector2, Vector3, U2};
use tau::TAU;

use conics::*;
use gui::common::*;
use orbits::*;

enum Void {}
impl FnOnce<(f64, f64)> for Void {
    type Output = ();
    extern "rust-call" fn call_once(self, _: (f64, f64)) -> () {
        match self {}
    }
}

struct RenderCommon<'a> {
    ctx: &'a cairo::Context,
    screen_extent: f64,
    proj_mat: Rotation3<f64>,
    planet: Ellipse,
    planet_path: cairo::Path,
    planet_path_inv: cairo::Path,
}

const AXIS_LENGTH: f64 = PLANET_RADIUS + 1000.0; // km
pub fn draw(ctx: &cairo::Context, st: &State) {
    ctx.set_antialias(cairo::Antialias::Best);
    ctx.set_fill_rule(cairo::FillRule::Winding);
    let (ox, oy, ex, ey) = ctx.clip_extents();

    // the center of our canvas should be at the origin and y-axis should point up
    ctx.translate((ox + ex) / 2.0, (oy + ey) / 2.0);
    ctx.scale(st.scale, -st.scale);

    // precompute a bunch of stuff
    let rc = {
        ctx.arc(0.0, 0.0, PLANET_RADIUS, 0.0, TAU);
        let planet_path = ctx.copy_path();
        ctx.new_path();
        ctx.arc_negative(0.0, 0.0, PLANET_RADIUS, TAU, 0.0);
        let planet_path_inv = ctx.copy_path();
        ctx.new_path();
        let proj_mat = Rotation3::from_axis_angle(&Vector3::x_axis(), st.eye_lat - TAU / 4.0)
            * Rotation3::from_axis_angle(&Vector3::z_axis(), st.eye_lon - TAU / 4.0);
        let (_, _, ex, ey) = ctx.clip_extents();
        RenderCommon {
            ctx: ctx,
            screen_extent: ex.hypot(ey),
            proj_mat: proj_mat,
            planet: Ellipse::new_circle(PLANET_RADIUS),
            planet_path: planet_path,
            planet_path_inv: planet_path_inv,
        }
    };

    // clear
    ctx.set_source_rgb(0.0, 0.0, 0.0);
    ctx.paint();

    // // Planet
    // ctx.append_path(&rc.planet_path);
    // ctx.set_source_rgb(0.0, 0.0, 0.75);
    // ctx.fill();

    // axis
    let north_pole = (rc.proj_mat * Vector3::z()).fixed_rows::<U2>(0) * AXIS_LENGTH;
    ctx.move_to(north_pole.x, north_pole.y);
    ctx.line_to(north_pole.x, -north_pole.y);
    ctx.save();
    ctx.identity_matrix();
    ctx.set_line_width(2.0);
    ctx.set_source_rgba(0.0, 1.0, 1.0, 0.5);
    ctx.stroke();
    ctx.restore();

    // // hemisphere lines
    // // sorta abusing `render_trajectory`
    // let great_circle = PlanarTrajectory { periapsis: PLANET_RADIUS, ..Default::default() };
    // let hemisphere_renderer = || {
    //     ctx.set_line_width(4.0);
    //     ctx.set_source_rgb(0.0, 1.0, 0.0);
    //     ctx.stroke();
    // };
    // println!("1");
    // render_trajctory(&rc,
    //                  Default::default(),
    //                  Trajectory {
    //                      p: Plane {
    //                          arg_peri: 0.0,
    //                          lon_asc_node: 0.0,
    //                          inclination: 0.0,
    //                      },
    //                      t: great_circle,
    //                  },
    //                  &hemisphere_renderer);
    // println!("2");
    // render_trajctory(&rc,
    //                  Default::default(),
    //                  Trajectory {
    //                      p: Plane {
    //                          arg_peri: 0.0,
    //                          lon_asc_node: 0.0,
    //                          inclination: TAU / 4.0,
    //                      },
    //                      t: great_circle,
    //                  },
    //                  &hemisphere_renderer);
    // println!("3");
    // render_trajctory(&rc,
    //                  Default::default(),
    //                  Trajectory {
    //                      p: Plane {
    //                          arg_peri: 0.0,
    //                          lon_asc_node: TAU / 4.0,
    //                          inclination: TAU / 4.0,
    //                      },
    //                      t: great_circle,
    //                  },
    //                  &hemisphere_renderer);

    // actual trajectory
    let rp = render_trajctory::<_, _, Void, Void, _>(
        &rc,
        InterestingPoints {
            apoapsis: Some(|x, y| {
                ctx.set_source_rgb(1.0, 0.0, 0.0);
                ctx.arc(x, y, 400.0, 0.0, TAU);
                ctx.fill();
            }),
            periapsis: Some(|x, y| {
                ctx.set_source_rgb(1.0, 0.0, 0.0);
                ctx.arc(x, y, 400.0, 0.0, TAU);
                ctx.fill();
            }),
            ascending_node: None,
            descending_node: None,
        },
        st.trajectory,
        || {
            ctx.set_source_rgb(1.0, 0.0, 0.0);
            ctx.set_line_width(5.0);
            ctx.stroke();
        },
    );

    composite_planes_and_planet(&rc, &[rp]);
}

struct InterestingPoints<F1, F2, F3, F4>
where
    F1: FnOnce(f64, f64) -> (),
    F2: FnOnce(f64, f64) -> (),
    F3: FnOnce(f64, f64) -> (),
    F4: FnOnce(f64, f64) -> (),
{
    apoapsis: Option<F1>,
    periapsis: Option<F2>,
    ascending_node: Option<F3>,
    descending_node: Option<F4>,
}

impl Default for InterestingPoints<Void, Void, Void, Void> {
    fn default() -> Self {
        InterestingPoints {
            apoapsis: None,
            periapsis: None,
            ascending_node: None,
            descending_node: None,
        }
    }
}

struct RenderedPlane {
    pattern: cairo::Pattern,
    inside_planet_path: cairo::Path,
    back_path_inv: cairo::Path,
    front_path_inv: cairo::Path,
}

// TODO: dynamic dispatch might be better here
fn render_trajctory<F1, F2, F3, F4, Fr>(
    rc: &RenderCommon,
    pts: InterestingPoints<F1, F2, F3, F4>,
    traj: Trajectory,
    tr_renderer: Fr,
) -> RenderedPlane
where
    F1: FnOnce(f64, f64) -> (),
    F2: FnOnce(f64, f64) -> (),
    F3: FnOnce(f64, f64) -> (),
    F4: FnOnce(f64, f64) -> (),
    Fr: FnOnce() -> (),
{
    let ctx = rc.ctx;

    // compute some stuff using `traj`
    let mat3 = rc.proj_mat * traj.p.to_matrix();
    let mat2 = mat3.matrix().fixed_slice::<U2, U2>(0, 0);
    let mut aff: Transform2<f64> = Transform2::identity();
    {
        let mut aff_set = aff.matrix_mut_unchecked().fixed_slice_mut::<U2, U2>(0, 0);
        for (a, m) in aff_set.iter_mut().zip(mat2.iter()) {
            *a = *m;
        }
    }
    let aff = aff;

    rc.planet
        .transform(&aff)
        .map(|e| draw_ellipse_arc(ctx, e, 0.0, TAU));
    let inside_planet_path = ctx.copy_path();
    ctx.new_path();

    // start rendering to a pattern
    ctx.push_group();

    // interesting points
    pts.apoapsis.map(|f| {
        traj.t.apoapsis().map(|ap| {
            let ap = mat2 * ap;
            f(ap.x, ap.y);
        })
    });
    pts.periapsis.map(|f| {
        let pe = mat2 * traj.t.periapsis();
        f(pe.x, pe.y);
    });

    if let Some(ell) = traj.t.to_ellipse().transform(&aff) {
        draw_ellipse_arc(ctx, ell, 0.0, TAU);
        ctx.save();
        ctx.identity_matrix();
        tr_renderer();
        ctx.restore();
    }

    let pattern = ctx.pop_group();

    // figure out which parts are in front of the planet and behind the planet
    let (back_path_inv, front_path_inv) = {
        let orbit_norm = mat3 * Vector3::z_axis();
        let cut_dir = orbit_norm.cross(&Vector3::z_axis());
        let cut_angle = cut_dir.y.atan2(cut_dir.x);

        ctx.arc_negative(0.0, 0.0, rc.screen_extent, cut_angle + TAU / 2.0, cut_angle);
        let left_path_inv = ctx.copy_path();
        ctx.new_path();
        ctx.arc_negative(0.0, 0.0, rc.screen_extent, cut_angle, cut_angle - TAU / 2.0);
        let right_path_inv = ctx.copy_path();
        ctx.new_path();

        if orbit_norm.z > 0.0 {
            (left_path_inv, right_path_inv)
        } else {
            (right_path_inv, left_path_inv)
        }
    };

    RenderedPlane {
        pattern,
        inside_planet_path,
        back_path_inv,
        front_path_inv,
    }
}

fn draw_ellipse_arc(ctx: &cairo::Context, ellipse: Ellipse, eta1: f64, eta2: f64) -> () {
    let el = ellipse.to_canonical();
    ctx.save();
    ctx.translate(el.center.x, el.center.y);
    ctx.rotate(el.rotation);
    if el.semi_axes.x != 0.0 && el.semi_axes.y != 0.0 {
        ctx.scale(el.semi_axes.x, el.semi_axes.y);
        ctx.arc(0.0, 0.0, 1.0, eta1, eta2);
    } else {
        // println!("line ellipse: {:?}", el);
    }
    ctx.restore();
}

fn composite_planes_and_planet(rc: &RenderCommon<'_>, planes: &[RenderedPlane]) -> () {
    let ctx = rc.ctx;
    // compose in z-order

    // back
    for rp in planes {
        ctx.save();
        ctx.append_path(&rp.inside_planet_path);
        ctx.append_path(&rp.back_path_inv);
        ctx.clip();
        ctx.append_path(&rp.back_path_inv);
        ctx.clip();
        ctx.set_source(&rp.pattern);
        ctx.paint();
        ctx.restore();
    }

    // planet back
    ctx.append_path(&rc.planet_path);
    ctx.set_source_rgba(0.0, 0.0, 0.75, 0.5);
    ctx.fill();

    // inside
    for rp in planes {
        ctx.save();
        ctx.append_path(&rp.inside_planet_path);
        ctx.clip();
        ctx.set_source(&rp.pattern);
        ctx.paint();
        ctx.restore();
    }

    // planet front
    ctx.append_path(&rc.planet_path);
    ctx.set_source_rgba(0.0, 0.0, 0.75, 0.5);
    ctx.fill();

    // front
    for rp in planes {
        ctx.save();
        ctx.append_path(&rp.inside_planet_path);
        ctx.append_path(&rp.front_path_inv);
        ctx.clip();
        ctx.append_path(&rp.front_path_inv);
        ctx.clip();
        ctx.set_source(&rp.pattern);
        ctx.paint();
        ctx.restore();
    }
}
