use super::{
    random::{Random, RandomPlugin},
    AngularVelocity, BoundaryWrap, Bounding, Debug, ShapeBundle, SpeedLimit, Velocity, DARK, LIGHT,
    POLY_LINE_WIDTH,
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
use std::{collections::btree_map::Range, ops::RangeInclusive};
pub const ASTEROID_SIZES: (
    RangeInclusive<f32>,
    RangeInclusive<f32>,
    RangeInclusive<f32>,
) = (60.0..=80.0, 30.0..=50.00, 10.0..=20.0);

pub const ASTEROID_LINE_WIDTH: f32 = 3.0;

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

#[derive(Bundle)]
pub struct AsteroidBundle {
    pub bound: Bounding,
    pub wrap: BoundaryWrap,
    pub vel: Velocity,
    pub health: Health,
    pub vel_limit: SpeedLimit,
    pub ang_vel: AngularVelocity,
    pub marker: Asteroid,
    #[bundle]
    pub shape: ShapeBundle,
}

pub fn asteroid_spawn_system(
    window: Res<WindowDescriptor>,
    mut rng: Local<Random>,
    mut ev_asteroid_spawn: EventWriter<AsteroidSpawnEvent>,
    asteroids: Query<(&Transform, &Bounding, With<Asteroid>)>,
) {
    if !rng.gen_bool(1.0 / 4.0) {
        return;
    }

    let h = window.height / 2.0;
    let w = window.width / 2.0;

    let size = rng.gen_range(0..=10);
    let radius = match size {
        0..=3 => rng.gen_range(ASTEROID_SIZES.0),
        4..=6 => rng.gen_range(ASTEROID_SIZES.1),
        7..=9 => rng.gen_range(ASTEROID_SIZES.2),
        _ => rng.gen_range(ASTEROID_SIZES.0),
    };

    let side = rng.gen_range(0..=3);
    let pos = match side {
        0 => vec2(-w, rng.gen_range(-h..h)),
        1 => vec2(w, rng.gen_range(-h..h)),
        2 => vec2(rng.gen_range(-w..w), -h),
        _ => vec2(rng.gen_range(-w..w), h),
    };

    let Vec2 { x: x1, y: y1 } = pos;
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
    ev_asteroid_spawn.send(AsteroidSpawnEvent {
        amount,
        pos,
        radius,
    });
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
            let pos = if *amount > 0 {
                let angle = ((360 / *amount * i) as f32).to_radians();
                vec2(
                    pos.x + *radius * 1.25 * angle.sin(),
                    pos.y + *radius * 1.25 * angle.cos(),
                )
            } else {
                *pos
            };

            let edges = rng.gen_range(9..15);

            let mut points = Vec::new();
            let angle_inc = 360.0 / edges as f32;

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
                .clone()
                .iter()
                .map(|p| ((p.x).powi(2) + (p.y).powi(2)).sqrt())
                .sum::<f32>()
                / p_len as f32;
            let bounding = average;
            let shape = shapes::Polygon {
                points,
                closed: true,
            };

            let center = vec3(pos.x, pos.y, 1.0);
            let health = match *radius as usize {
                60..=80 => 30.0,
                30..=50 => 20.0,
                _ => 1.0,
            };
            let vel = match *radius as usize {
                60..=80 => {
                    let dest = vec3(1.0, 1.0, 1.0);
                    let angle = center.angle_between(dest);
                    let direction = Quat::from_rotation_z(angle) * -Vec3::Y; //TODO: find out why this works
                    let force = rng.gen_range(10.0..50.00);
                    vec2(force * direction.x, force * direction.y)
                }
                30..=50 => {
                    let direction =
                        Quat::from_rotation_z((rng.gen_range(0..360) as f32).to_radians())
                            * -Vec3::Y; //TODO: find out why this works
                    let force = rng.gen_range(20.0..60.00);
                    vec2(force * direction.x, force * direction.y)
                }
                _ => {
                    let direction =
                        Quat::from_rotation_z((rng.gen_range(0..360) as f32).to_radians())
                            * -Vec3::Y; //TODO: find out why this works
                    let force = rng.gen_range(30.0..70.00);
                    vec2(force * direction.x, force * direction.y)
                }
            };

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
                    vel: Velocity::from(vel),
                    vel_limit: SpeedLimit::from(200.0),
                    ang_vel: AngularVelocity::from(rng.gen_range(0.1..1.0)),
                    marker: Asteroid,
                    health: Health(health),
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
