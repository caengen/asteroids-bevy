use crate::{GAME_BORDER_OFFSET, GAME_FRAME_HEIGHT, GAME_FRAME_WIDTH};

use super::{
    random::Random, AngularVelocity, BoundaryWrap, Bounding, Debug, ShapeBundle, SpeedLimit,
    Velocity, DARK, LIGHT, POLY_LINE_WIDTH,
};
use bevy::{
    math::{vec2, vec3},
    prelude::*,
};
use bevy_prototype_lyon::{
    prelude::{DrawMode, FillMode, GeometryBuilder, StrokeMode},
    shapes,
};
use rand::Rng;
use std::ops::RangeInclusive;
pub const ASTEROID_SIZES: (
    RangeInclusive<f32>,
    RangeInclusive<f32>,
    RangeInclusive<f32>,
) = (60.0..=80.0, 30.0..=50.00, 10.0..=20.0);

#[derive(Debug, Component)]
pub struct Asteroid;
#[derive(Debug, Component)]
pub struct Health(pub f32);
#[derive(Debug, Component)]
pub struct Damage(pub f32);

pub struct AsteroidSpawnEvent {
    pub pos: Vec2,
    pub radius: f32,
    pub amount: i32,
}

pub struct AsteroidSplitEvent {
    pub parent_points: Vec<Vec2>,
    pub pos: Vec2,
    pub radius: f32,
    pub amount: f32,
}

#[derive(Debug, Component, Clone, Deref)]
pub struct Points(pub Vec<Vec2>);

#[derive(Bundle)]
pub struct AsteroidBundle {
    pub bound: Bounding,
    pub wrap: BoundaryWrap,
    pub vel: Velocity,
    pub health: Health,
    pub vel_limit: SpeedLimit,
    pub ang_vel: AngularVelocity,
    pub marker: Asteroid,
    pub points: Points,
    #[bundle]
    pub shape: ShapeBundle,
}

pub fn velocity(center: &Vec2, radius: &f32, rng: &mut Random) -> Velocity {
    let v = match *radius as usize {
        60..=80 => {
            let dest = vec2(1.0, 1.0);
            let angle = center.angle_between(dest);
            let direction = Quat::from_rotation_z(angle) * -Vec3::Y; //TODO: find out why this works
            let force = rng.gen_range(10.0..50.00);
            vec2(force * direction.x, force * direction.y)
        }
        30..=50 => {
            let direction =
                Quat::from_rotation_z((rng.gen_range(0..360) as f32).to_radians()) * -Vec3::Y; //TODO: find out why this works
            let force = rng.gen_range(20.0..60.00);
            vec2(force * direction.x, force * direction.y)
        }
        _ => {
            let direction =
                Quat::from_rotation_z((rng.gen_range(0..360) as f32).to_radians()) * -Vec3::Y; //TODO: find out why this works
            let force = rng.gen_range(30.0..70.00);
            vec2(force * direction.x, force * direction.y)
        }
    };

    Velocity::from(v)
}

pub fn health(radius: &f32) -> Health {
    let h = match *radius as usize {
        60..=80 => 30.0,
        30..=50 => 20.0,
        _ => 1.0,
    };

    Health(h)
}

pub fn asteroid_spawn_system(
    mut rng: Local<Random>,
    mut ev_asteroid_spawn: EventWriter<AsteroidSpawnEvent>,
    asteroids: Query<(&Transform, &Bounding, With<Asteroid>)>,
) {
    if !rng.gen_bool(1.0 / 6.0) {
        return;
    }

    let h = (GAME_FRAME_HEIGHT - GAME_BORDER_OFFSET) / 2.0;
    let w = (GAME_FRAME_WIDTH - GAME_BORDER_OFFSET) / 2.0;

    let size = rng.gen_range(0..=10);
    let radius = match size {
        0..=3 => rng.gen_range(ASTEROID_SIZES.0),
        4..=6 => rng.gen_range(ASTEROID_SIZES.1),
        7..=9 => rng.gen_range(ASTEROID_SIZES.2),
        _ => rng.gen_range(ASTEROID_SIZES.0),
    };

    let side = rng.gen_range(0..=3);
    let pos = match side {
        0 => vec3(-w, rng.gen_range(-h..h), 1.0),
        1 => vec3(w, rng.gen_range(-h..h), 1.0),
        2 => vec3(rng.gen_range(-w..w), -h, 1.0),
        _ => vec3(rng.gen_range(-w..w), h, 1.0),
    };

    let Vec3 { x: x1, y: y1, z: _ } = pos;
    let r1 = radius;
    for (transform, bounding, _) in asteroids.iter() {
        let Vec3 { x: x2, y: y2, z: _ } = transform.translation;
        let r2 = bounding.0;
        let d = ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt();
        if d < r1 + r2 {
            // spawn collides with existing asteroid
            return;
        }
    }

    let amount = 1;
    // må fikse til å kun bruke vec3..
    ev_asteroid_spawn.send(AsteroidSpawnEvent {
        amount,
        pos: vec2(pos.x, pos.y),
        radius,
    });
}

pub fn asteroid_split_system(
    mut commands: Commands,
    mut rng: Local<Random>,
    mut ev_asteroid_split: EventReader<AsteroidSplitEvent>,
    debug: Res<Debug>,
) {
    for AsteroidSplitEvent {
        parent_points: pp,
        pos,
        radius,
        amount,
    } in ev_asteroid_split.iter()
    {
        let len = pp.len() as f32;
        let num = (len / amount).floor() as usize;
        let a = &pp[..=num];
        let b = &pp[num..=(num * 2)];
        let c = &pp[(num * 2)..];
        let center_point = vec2(
            pp.iter().map(|p| p.x).sum::<f32>() / len,
            pp.iter().map(|p| p.y).sum::<f32>() / len,
        );

        //[&c[0], a.to_vec(), &b[0]].concat()

        // let z = c[0].clone();
        let mut a_points = vec![center_point.clone(), c[c.len() - 1]];
        a_points.extend_from_slice(a);
        a_points.push(b[0]);
        let a_shape = shapes::Polygon {
            points: a_points.clone(),
            closed: true,
        };

        let mut b_points = vec![center_point.clone(), a[a.len() - 1]];
        b_points.extend_from_slice(b);
        b_points.push(c[0]);
        let b_shape = shapes::Polygon {
            points: b_points.clone(),
            closed: true,
        };

        let mut c_points = vec![center_point.clone(), b[b.len() - 1]];
        c_points.extend_from_slice(c);
        c_points.push(a[0]);
        let c_shape = shapes::Polygon {
            points: c_points.clone(),
            closed: true,
        };

        let shapes = [
            (a_shape, a_points),
            (b_shape, b_points),
            (c_shape, c_points),
        ];

        for (shape, points) in shapes.iter() {
            let centroid = vec2(
                points.iter().map(|p| p.x).sum::<f32>() / len,
                points.iter().map(|p| p.y).sum::<f32>() / len,
            );

            let bounding = points
                .iter()
                .map(|p| ((p.x).powi(2) + (p.y).powi(2)).sqrt())
                .sum::<f32>()
                / points.len() as f32;

            let asteroid = commands
                .spawn_bundle(AsteroidBundle {
                    shape: (GeometryBuilder::build_as(
                        shape,
                        DrawMode::Outlined {
                            outline_mode: StrokeMode::new(LIGHT, POLY_LINE_WIDTH * 1.5),
                            fill_mode: FillMode::color(DARK),
                        },
                        Transform::default().with_translation(vec3(
                            if centroid.x.is_sign_positive() {
                                pos.x - centroid.x
                            } else {
                                pos.x + centroid.x
                            },
                            if centroid.y.is_sign_positive() {
                                pos.y - centroid.y
                            } else {
                                pos.y + centroid.y
                            },
                            1.0,
                        )),
                    )),
                    bound: Bounding::from(bounding),
                    wrap: BoundaryWrap,
                    vel: velocity(&centroid, &bounding, &mut rng),
                    vel_limit: SpeedLimit::from(200.0),
                    ang_vel: AngularVelocity::from(rng.gen_range(0.1..1.0)),
                    marker: Asteroid,
                    points: Points(points.clone()),
                    health: health(&bounding),
                })
                .id();

            if debug.0 {
                let d_circle = shapes::Circle {
                    radius: bounding,
                    ..Default::default()
                };
                let debug_bound = commands
                    .spawn()
                    .insert_bundle(
                        (GeometryBuilder::build_as(
                            &d_circle,
                            DrawMode::Outlined {
                                outline_mode: StrokeMode::new(Color::RED, POLY_LINE_WIDTH * 1.5),
                                fill_mode: FillMode::color(DARK),
                            },
                            Transform::default(),
                        )),
                    )
                    .id();
                commands.entity(asteroid).insert_children(0, &[debug_bound]);
            }
        }
    }
}

pub fn asteroid_generation_system(
    mut commands: Commands,
    mut rng: Local<Random>,
    mut ev_asteroid_spawn: EventReader<AsteroidSpawnEvent>,
    debug: Res<Debug>,
) {
    for AsteroidSpawnEvent {
        amount,
        pos,
        radius,
    } in ev_asteroid_spawn.iter()
    {
        for i in 0..*amount {
            let mut points = Vec::new();
            let angle = ((360 / *amount * i) as f32).to_radians();
            let pos = vec2(
                pos.x + *radius * 1.25 * angle.sin(),
                pos.y + *radius * 1.25 * angle.cos(),
            );

            let edges = rng.gen_range(9..15);

            let angle_inc = 360.0 / edges as f32;

            let (min, max) = match *radius as usize {
                60..=80 => (60.0, 80.0),
                30..=50 => (30.0, 50.0),
                _ => (10.0, 20.0),
            };

            for i in 1..=edges {
                let r = match *radius as usize {
                    60..=80 => rng.gen_range((*radius as i32 - 30)..=(*radius as i32)),
                    30..=50 => rng.gen_range((*radius as i32 - 15)..=(*radius as i32)),
                    _ => rng.gen_range((*radius as i32 - 3)..=(*radius as i32)),
                } as f32;

                let angle = (angle_inc * i as f32).to_radians();
                points.push(vec2(r * angle.sin(), r * angle.cos()));
            }
            let p_len = points.len();
            let average = points
                .iter()
                .map(|p| ((p.x).powi(2) + (p.y).powi(2)).sqrt())
                .sum::<f32>()
                / p_len as f32;
            let bounding = average.clamp(min, max);
            let shape = shapes::Polygon {
                points: points.clone(),
                closed: true,
            };

            let center = vec3(pos.x, pos.y, 1.0);

            let asteroid = commands
                .spawn_bundle(AsteroidBundle {
                    shape: (GeometryBuilder::build_as(
                        &shape,
                        DrawMode::Outlined {
                            outline_mode: StrokeMode::new(LIGHT, POLY_LINE_WIDTH * 1.5),
                            fill_mode: FillMode::color(DARK),
                        },
                        Transform::default().with_translation(center),
                    )),
                    bound: Bounding::from(bounding),
                    wrap: BoundaryWrap,
                    vel: velocity(&vec2(center.x, center.y), radius, &mut rng),
                    vel_limit: SpeedLimit::from(200.0),
                    ang_vel: AngularVelocity::from(rng.gen_range(0.1..1.0)),
                    marker: Asteroid,
                    points: Points(points),
                    health: health(radius),
                })
                .id();

            if debug.0 {
                let d_circle = shapes::Circle {
                    radius: average,
                    ..Default::default()
                };
                let debug_bound = commands
                    .spawn()
                    .insert_bundle(
                        (GeometryBuilder::build_as(
                            &d_circle,
                            DrawMode::Outlined {
                                outline_mode: StrokeMode::new(Color::RED, POLY_LINE_WIDTH * 1.5),
                                fill_mode: FillMode::color(DARK),
                            },
                            Transform::default(),
                        )),
                    )
                    .id();
                commands.entity(asteroid).insert_children(0, &[debug_bound]);
            }
        }
    }
}
