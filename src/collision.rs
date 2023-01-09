use std::{f32::consts::PI, time::Duration};

use crate::{
    asteroid::{Damage, Health},
    particles::BallParticleSpawnEvent,
    DamageTransferEvent, Flick,
};

use super::{
    random::{Random, RandomPlugin},
    Asteroid, AsteroidSpawnEvent, DestructionEvent, GrainParticleSpawnEvent, PlayerDeathEvent,
    Ship, ShipState, Velocity, ASTEROID_SIZES,
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

pub fn damage_transfer_system<Victim: Component>(
    mut ev_grain: EventWriter<GrainParticleSpawnEvent>,
    mut ev_ball_particles: EventWriter<BallParticleSpawnEvent>,
    mut ev_destruction: EventWriter<DestructionEvent>,
    mut ev_asteroid_spawn: EventWriter<AsteroidSpawnEvent>,
    mut victims: Query<(
        Entity,
        &Velocity,
        &Transform,
        &Bounding,
        &mut Health,
        Option<&Asteroid>,
        With<Victim>,
    )>,
    mut dealers: Query<(Entity, &Velocity, &Transform, &Bounding, &Damage)>,
    mut rng: Local<Random>,
    mut commands: Commands,
) {
    for (victim, vv, vt, vb, mut health, asteroid, _) in victims.iter_mut() {
        let Vec3 { x: x1, y: y1, z: _ } = vt.translation;
        for (dealer, dv, dt, db, damage) in dealers.iter_mut() {
            let Vec3 { x: x2, y: y2, z: _ } = dt.translation;
            if circles_touching(&vt, vb, &dt, db) {
                let new_health = health.0 - damage.0;
                if new_health < 0.0 {
                    ev_destruction.send(DestructionEvent { entity: victim });
                    if let Some(Asteroid) = asteroid {
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
                                    amount: 2,
                                    pos: vec2(vt.translation.x, vt.translation.y),
                                    radius: rng.gen_range(ASTEROID_SIZES.1),
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
                                    amount: 3,
                                    pos: vec2(vt.translation.x, vt.translation.y),
                                    radius: rng.gen_range(ASTEROID_SIZES.2),
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

                commands.entity(dealer).despawn();
            }
        }
    }
}

pub fn kill_collision_system<A: Component, B: Component>(
    mut ev_explode: EventWriter<GrainParticleSpawnEvent>,
    mut ev_player_death: EventWriter<PlayerDeathEvent>,
    colliders: Query<(Entity, &Transform, &Bounding, &Velocity, With<A>)>,
    mut victims: Query<(Entity, &Transform, &Bounding, With<B>, Option<&mut Ship>)>,
) {
    for (_collider, at, ab, avel, _) in colliders.iter() {
        let Vec3 { x: x1, y: y1, z: _ } = at.translation;
        let r1 = ab.0;
        for (victim, bt, bb, _, ship) in victims.iter_mut() {
            let Vec3 { x: x2, y: y2, z: _ } = bt.translation;
            let r2 = bb.0;
            if circles_touching(&at, ab, &bt, bb) {
                if let Some(mut ship) = ship {
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
