use std::f32::consts::PI;

use super::{
    random::{Random, RandomPlugin},
    Asteroid, AsteroidSpawnEvent, ExplosionEvent, HitEvent, PlayerDeathEvent, Ship, ShipState,
    Velocity, ASTEROID_SIZES,
};
use bevy::ecs::component::Component;
use bevy::math::vec2;
use bevy::prelude::*;
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
#[derive(Debug, Component, Default, Deref, DerefMut, From)]
pub struct Bounding(pub f32);

// Temporarily Radius will act as Mass for momentum calculation
pub fn self_collision_system<A: Component>(
    mut colliders: Query<(Entity, &mut Transform, &Bounding, &mut Velocity, With<A>)>,
    mut rng: Local<Random>,
) {
    let mut combinations = colliders.iter_combinations_mut();
    while let Some([mut a, mut b]) = combinations.fetch_next() {
        let (_, mut at, ab, mut av, _) = a;
        let Vec3 { x: ax, y: ay, z: _ } = at.translation;
        let ar = ab.0; // radius
        let (_, mut bt, bb, mut bv, _) = b;
        let Vec3 { x: bx, y: by, z: _ } = bt.translation;
        let br = bb.0; // radius

        if circles_touching(&at, ab, &bt, bb) {
            let contact_angle = f32::atan2(by - ay, bx - ax);

            let distance_to_move = distance_to_move(&at.translation, ar, &bt.translation, br);
            // move the second circle
            bt.translation.x += f32::cos(contact_angle) * distance_to_move;
            bt.translation.y += f32::sin(contact_angle) * distance_to_move;

            let v1 = av.length();
            let v2 = bv.length();
            let m1 = PI * ar.powi(2);
            let m2 = PI * br.powi(2);
            let a1 = av.angle_between(vec2(ax, ay));
            let a2 = bv.angle_between(vec2(bx, by));

            av.0 = vel_after_collision(v1, m1, a1, v2, m2, a2, contact_angle) * 0.992;
            bv.0 = vel_after_collision(v2, m2, a2, v1, m1, a1, contact_angle) * 0.992;
        }
    }
}

pub fn collision_system<A: Component, B: Component>(
    mut ev_hit: EventWriter<HitEvent>,
    mut ev_explode: EventWriter<ExplosionEvent>,
    mut ev_asteroid_spawn: EventWriter<AsteroidSpawnEvent>,
    mut ev_player_death: EventWriter<PlayerDeathEvent>,
    colliders: Query<(Entity, &Transform, &Bounding, &Velocity, With<A>)>,
    mut victims: Query<(
        Entity,
        &Transform,
        &Bounding,
        &Velocity,
        With<B>,
        Option<&Asteroid>,
        Option<&mut Ship>,
    )>,
    mut rng: Local<Random>,
) {
    for (_collider, at, ab, avel, _) in colliders.iter() {
        let Vec3 { x: x1, y: y1, z: _ } = at.translation;
        let r1 = ab.0;
        for (victim, bt, bb, bvel, _, asteroid, ship) in victims.iter_mut() {
            let Vec3 { x: x2, y: y2, z: _ } = bt.translation;
            let r2 = bb.0;
            if circles_touching(&at, ab, &bt, bb) {
                if let Some(mut ship) = ship {
                    if matches!(ship.state, ShipState::Alive) {
                        ev_explode.send(ExplosionEvent {
                            pos: bt.translation,
                            radius: r2,
                            particles: 150..200,
                            impact_vel: vec2(avel.x, avel.y),
                        });
                        ev_player_death.send(PlayerDeathEvent {});
                    }
                } else {
                    ev_hit.send(HitEvent { entity: victim });
                    if let Some(Asteroid) = asteroid {
                        match bb.0 as usize {
                            60..=80 => {
                                ev_asteroid_spawn.send(AsteroidSpawnEvent {
                                    amount: 2,
                                    pos: vec2(bt.translation.x, bt.translation.y),
                                    radius: rng.gen_range(ASTEROID_SIZES.1),
                                });
                            }
                            30..=50 => {
                                ev_asteroid_spawn.send(AsteroidSpawnEvent {
                                    amount: 3,
                                    pos: vec2(bt.translation.x, bt.translation.y),
                                    radius: rng.gen_range(ASTEROID_SIZES.2),
                                });
                            }
                            _ => {
                                // hack. need to add weight to impacters
                                ev_explode.send(ExplosionEvent {
                                    pos: bt.translation,
                                    radius: r2,
                                    particles: 50..100,
                                    impact_vel: vec2(
                                        bvel.x + (avel.x / 3.0),
                                        bvel.y + (avel.y / 3.0),
                                    ),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}
