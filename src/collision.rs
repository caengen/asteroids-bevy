use std::{f32::consts::PI, ops::Range, time::Duration};

use crate::{
    asteroid::{AsteroidSpawnEvent, AsteroidSplitEvent, Damage, Health, Points},
    weapons::Bullet,
    Flick,
};

use super::{
    random::Random, Asteroid, DestructionEvent, GrainParticleSpawnEvent, PlayerDeathEvent, Ship,
    ShipState, Velocity, ASTEROID_SIZES,
};
use bevy::math::vec2;
use bevy::prelude::*;
use bevy::{ecs::component::Component, math::vec3};
use bevy_prototype_lyon::prelude::Path;
use derive_more::From;
use rand::Rng;

fn distance_between(a: &Vec3, b: &Vec3) -> f32 {
    a.distance(*b)
}

fn circles_touching(a: &Transform, ar: &Bounding, b: &Transform, br: &Bounding) -> bool {
    distance_between(&a.translation, &b.translation) < (ar.0 + br.0)
}

fn distance_to_move(a: &Vec3, ar: f32, b: &Vec3, br: f32) -> f32 {
    ar + br - distance_between(a, b)
}

// c: contact angle
fn x_vel_comp_after_collision(v1: f32, m1: f32, a1: f32, v2: f32, m2: f32, a2: f32, c: f32) -> f32 {
    ((v1 * f32::cos(a1 - c) * (m1 - m2) + 2.0 * m2 * v2 * f32::cos(a2 - c)) / (m1 + m2))
        * f32::cos(c)
        + v1 * f32::sin(a1 - c) * f32::cos(c + PI / 2.0)
}

fn y_vel_comp_after_collision(v1: f32, m1: f32, a1: f32, v2: f32, m2: f32, a2: f32, c: f32) -> f32 {
    ((v1 * f32::cos(a1 - c) * (m1 - m2) + 2.0 * m2 * v2 * f32::cos(a2 - c)) / (m1 + m2))
        * f32::sin(c)
        + v1 * f32::sin(a1 - c) * f32::sin(c + PI / 2.0)
}

fn vel_after_collision(v1: f32, m1: f32, a1: f32, v2: f32, m2: f32, a2: f32, c: f32) -> Vec2 {
    vec2(
        x_vel_comp_after_collision(v1, m1, a1, v2, m2, a2, c),
        y_vel_comp_after_collision(v1, m1, a1, v2, m2, a2, c),
    )
}

fn rotate_point(fp: Vec2, pt: Vec2, a: f32) -> Vec2 {
    let x = pt.x - fp.x;
    let y = pt.y - fp.y;
    let x_rot = x * a.cos() + y * a.sin();
    let y_rot = y * a.cos() - x * a.sin();

    vec2(fp.x + x_rot, fp.y + y_rot)
}

// source: https://github.com/williamfiset/Algorithms/blob/master/src/main/java/com/williamfiset/algorithms/geometry/CircleCircleIntersectionPoints.js
fn circle_impact_position(a: &Transform, b: &Transform, ar: f32, br: f32) -> Option<Vec3> {
    let mut r: f32 = 0.0;
    let mut R: f32 = 0.0;
    let mut cx: f32 = 0.0;
    let mut Cx: f32 = 0.0;
    let mut cy: f32 = 0.0;
    let mut Cy: f32 = 0.0;
    if ar < br {
        r = ar;
        R = br;
        cx = a.translation.x;
        cy = a.translation.y;
        Cx = b.translation.x;
        Cy = b.translation.y;
    } else {
        r = ar;
        R = br;
        Cx = b.translation.x;
        Cy = b.translation.y;
        cx = a.translation.x;
        cy = a.translation.y;
    }

    let d = distance_between(&a.translation, &b.translation);

    if d < f32::EPSILON && (R - r).abs() < f32::EPSILON {
        return None;
    }
    // No intersection (circles centered at the
    // same place with different size)
    else if d < f32::EPSILON {
        return None;
    }

    let dx = cx - Cx;
    let dy = cy - Cy;
    let x = (dx / d) * R + Cx;
    let y = (dy / d) * R + Cy;
    let P = vec2(x, y);

    if (ar - br).abs() - d < f32::EPSILON || (ar - br + d).abs() < f32::EPSILON {
        return Some(vec3(P.x, P.y, 1.0));
    }

    // No intersection. Either the small circle contained within
    // big circle or circles are simply disjoint.
    if (d + r) < R || (R + r < d) {
        return None;
    };

    let C = vec2(Cx, Cy);
    let angle = ((r * r - d * d - R * R) / (-2.0 * d * R)).acos();
    let pt1 = rotate_point(C, P, angle.abs());
    let pt2 = rotate_point(C, P, -angle);
    let avg = vec3((pt1.x + pt2.x) / 2.0, (pt1.y + pt2.y) / 2.0, 1.0);

    return Some(avg);
}

#[derive(Debug, Component, Default, Deref, DerefMut, From)]
pub struct Bounding(pub f32);

const IMPACT_VEL_PARTICLE_TRIGGER: f32 = 25.0;
const IMPACT_SPAWN_RADIUS: f32 = 1.0;
const IMPCAT_PARTICLE_RANGE: Range<i32> = 1..5;
// Temporarily Radius will act as Mass for momentum calculation
pub fn self_collision_system<A: Component>(
    mut colliders: Query<(&mut Transform, &Bounding, &mut Velocity, With<A>)>,
    mut ev_grain: EventWriter<GrainParticleSpawnEvent>,
) {
    let mut combinations = colliders.iter_combinations_mut();
    while let Some([a, b]) = combinations.fetch_next() {
        let (at, ab, mut av, _) = a;
        let Vec3 { x: ax, y: ay, z: _ } = at.translation;
        let ar = ab.0; // radius
        let (mut bt, bb, mut bv, _) = b;
        let Vec3 { x: bx, y: by, z: _ } = bt.translation;
        let br = bb.0; // radius

        if circles_touching(&at, ab, &bt, bb) {
            let contact_angle = f32::atan2(by - ay, bx - ax);
            let distance_to_move = distance_to_move(&at.translation, ar, &bt.translation, br);

            // velocities
            let v1 = av.length();
            let v2 = bv.length();

            if v1 > IMPACT_VEL_PARTICLE_TRIGGER || v2 > IMPACT_VEL_PARTICLE_TRIGGER {
                if let Some(impact_pos) = circle_impact_position(&at, &bt, ab.0, bb.0) {
                    ev_grain.send(GrainParticleSpawnEvent {
                        pos: impact_pos,
                        spawn_radius: IMPACT_SPAWN_RADIUS,
                        particles: IMPCAT_PARTICLE_RANGE,
                        impact_vel: vec2(0.0, 0.0),
                    });
                }
            }

            // move the second circle
            bt.translation.x += f32::cos(contact_angle) * distance_to_move;
            bt.translation.y += f32::sin(contact_angle) * distance_to_move;

            // masses
            let m1 = PI * ar.powi(2);
            let m2 = PI * br.powi(2);
            // angles
            let a1 = av.angle_between(vec2(ax, ay));
            let a2 = bv.angle_between(vec2(bx, by));

            av.0 = vel_after_collision(v1, m1, a1, v2, m2, a2, contact_angle) * 0.992;
            bv.0 = vel_after_collision(v2, m2, a2, v1, m1, a1, contact_angle) * 0.992;
        }
    }
}

pub fn damage_transfer_system<Dealer: Component, Victim: Component>(
    mut ev_grain: EventWriter<GrainParticleSpawnEvent>,
    // mut ev_ball_particles: EventWriter<BallParticleSpawnEvent>,
    mut ev_destruction: EventWriter<DestructionEvent>,
    mut ev_asteroid_spawn: EventWriter<AsteroidSpawnEvent>,
    mut victims: Query<(
        Entity,
        &Velocity,
        &Transform,
        &Bounding,
        &mut Health,
        Option<&Asteroid>,
        Option<&Points>,
        With<Victim>,
    )>,
    mut dealers: Query<(
        Entity,
        &Velocity,
        &Transform,
        &Bounding,
        &Damage,
        Option<&Bullet>,
        With<Dealer>,
    )>,
    mut rng: Local<Random>,
    mut commands: Commands,
) {
    for (victim, vv, vt, vb, mut health, asteroid, points, _) in victims.iter_mut() {
        let Vec3 { x: x1, y: y1, z: _ } = vt.translation;
        for (dealer, dv, dt, db, damage, bullet, _) in dealers.iter_mut() {
            let Vec3 { x: x2, y: y2, z: _ } = dt.translation;
            if circles_touching(&vt, vb, &dt, db) {
                let new_health = health.0 - damage.0;
                if new_health < 0.0 {
                    ev_destruction.send(DestructionEvent { entity: victim });
                    if let Some(Asteroid) = asteroid {
                        // let points = points.unwrap();
                        match vb.0 as usize {
                            60..=80 => {
                                // ev_ball_particles.send(BallParticleSpawnEvent {
                                //     pos: vt.translation,
                                //     spawn_radius: vb.0 * 1.25,
                                //     ball_radius: 30.0,
                                //     particles: 20..30,
                                //     delay: 30,
                                //     dir_vel: dv.0,
                                // });
                                ev_asteroid_spawn.send(AsteroidSpawnEvent {
                                    // parent_points: points.0.clone(),
                                    amount: 3,
                                    pos: vec2(vt.translation.x, vt.translation.y),
                                    radius: rng.gen_range(ASTEROID_SIZES.1),
                                });
                                ev_grain.send(GrainParticleSpawnEvent {
                                    pos: vt.translation,
                                    spawn_radius: vb.0 / 1.5,
                                    particles: 100..200,
                                    impact_vel: vec2(0.0, 0.0),
                                });
                            }
                            30..=50 => {
                                // ev_ball_particles.send(BallParticleSpawnEvent {
                                //     pos: vt.translation,
                                //     spawn_radius: vb.0 * 1.25,
                                //     ball_radius: 18.0,
                                //     particles: 10..20,
                                //     delay: 30,
                                //     dir_vel: dv.0,
                                // });
                                ev_asteroid_spawn.send(AsteroidSpawnEvent {
                                    // parent_points: points.0.clone(),
                                    amount: 3,
                                    pos: vec2(vt.translation.x, vt.translation.y),
                                    radius: rng.gen_range(ASTEROID_SIZES.2),
                                });
                                ev_grain.send(GrainParticleSpawnEvent {
                                    pos: vt.translation,
                                    spawn_radius: vb.0 / 1.5,
                                    particles: 100..200,
                                    impact_vel: vec2(0.0, 0.0),
                                });
                            }
                            _ => {
                                ev_grain.send(GrainParticleSpawnEvent {
                                    pos: dt.translation,
                                    spawn_radius: db.0,
                                    particles: 3..15,
                                    impact_vel: -(dv.0 / 4.0),
                                });
                                // hack. need to add weight to impacters
                                ev_grain.send(GrainParticleSpawnEvent {
                                    pos: dt.translation,
                                    spawn_radius: db.0,
                                    particles: 30..70,
                                    impact_vel: vec2((dv.x / 3.0), (dv.y / 3.0)),
                                });
                            }
                        }
                    }
                } else {
                    health.0 = new_health;

                    ev_grain.send(GrainParticleSpawnEvent {
                        pos: dt.translation,
                        spawn_radius: db.0,
                        particles: 3..15,
                        impact_vel: -(dv.0 / 4.0),
                    });
                    commands.entity(victim).insert(Flick {
                        duration: Timer::new(Duration::from_millis(75), false),
                        switch_timer: Timer::new(Duration::from_millis(1), false),
                    });
                }

                if let Some(Bullet(_)) = bullet {
                    commands.entity(dealer).despawn();
                }
            }
        }
    }
}

pub fn elastic_collision_system<A: Component, B: Component>(
    mut colliders: Query<(&mut Transform, &Bounding, &mut Velocity, With<A>)>,
    mut victims: Query<(
        &mut Transform,
        &Bounding,
        &mut Velocity,
        With<B>,
        Without<A>,
    )>,
    mut ev_grain: EventWriter<GrainParticleSpawnEvent>,
) {
    for (at, ab, mut av, _) in colliders.iter_mut() {
        for (mut bt, bb, mut bv, _, _) in victims.iter_mut() {
            let Vec3 { x: ax, y: ay, z: _ } = at.translation;
            let ar = ab.0; // radius
            let Vec3 { x: bx, y: by, z: _ } = bt.translation;
            let br = bb.0; // radius

            if circles_touching(&at, ab, &bt, bb) {
                let contact_angle = f32::atan2(by - ay, bx - ax);
                let distance_to_move = distance_to_move(&at.translation, ar, &bt.translation, br);

                // velocities
                let v1 = av.length();
                let v2 = bv.length();

                if v1 > IMPACT_VEL_PARTICLE_TRIGGER || v2 > IMPACT_VEL_PARTICLE_TRIGGER {
                    if let Some(impact_pos) = circle_impact_position(&at, &bt, ab.0, bb.0) {
                        ev_grain.send(GrainParticleSpawnEvent {
                            pos: impact_pos,
                            spawn_radius: IMPACT_SPAWN_RADIUS,
                            particles: IMPCAT_PARTICLE_RANGE,
                            impact_vel: vec2(0.0, 0.0),
                        });
                    }
                }

                // move the second circle
                bt.translation.x += f32::cos(contact_angle) * distance_to_move;
                bt.translation.y += f32::sin(contact_angle) * distance_to_move;

                // masses
                let m1 = PI * ar.powi(2);
                let m2 = PI * br.powi(2);
                // angles
                let a1 = av.angle_between(vec2(ax, ay));
                let a2 = bv.angle_between(vec2(bx, by));

                av.0 = vel_after_collision(v1, m1, a1, v2, m2, a2, contact_angle) * 0.992;
                bv.0 = vel_after_collision(v2, m2, a2, v1, m1, a1, contact_angle) * 0.992;
            }
        }
    }
}

pub fn kill_collision_system<A: Component, B: Component>(
    mut ev_explode: EventWriter<GrainParticleSpawnEvent>,
    mut ev_player_death: EventWriter<PlayerDeathEvent>,
    colliders: Query<(Entity, &Transform, &Bounding, &Velocity, With<A>)>,
    mut victims: Query<(&Transform, &Bounding, With<B>, Option<&mut Ship>)>,
) {
    for (_collider, at, ab, avel, _) in colliders.iter() {
        for (bt, bb, _, ship) in victims.iter_mut() {
            let r2 = bb.0;
            if circles_touching(&at, ab, &bt, bb) {
                if let Some(ship) = ship {
                    if matches!(ship.state, ShipState::Alive) {
                        ev_explode.send(GrainParticleSpawnEvent {
                            pos: bt.translation,
                            spawn_radius: r2,
                            particles: 150..200,
                            impact_vel: vec2(avel.x, avel.y),
                        });
                        ev_player_death.send(PlayerDeathEvent {});
                    }
                }
            }
        }
    }
}
