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

#[derive(Debug, Component, Default, Deref, DerefMut, From)]
pub struct Bounding(pub f32);

// Temporarily Radius will act as Mass for momentum calculation
pub fn self_collision_system<A: Component>(
    mut colliders: Query<(Entity, &mut Transform, &Bounding, &mut Velocity, With<A>)>,
    mut rng: Local<Random>,
) {
    let mut combinations = colliders.iter_combinations_mut();
    while let Some([mut a, mut b]) = combinations.fetch_next() {
        let (_, mut ct, cb, mut cv, _) = a;
        let Vec3 { x: x1, y: y1, z: _ } = ct.translation;
        let r1 = cb.0;
        let (_, ot, ob, mut ov, _) = b;
        let Vec3 { x: x2, y: y2, z: _ } = ot.translation;
        let r2 = ob.0;
        let d = ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt();
        if d < r1 + r2 {
            // set distance between to exactly r1 + r2
            let mut dist = (ct.translation - ot.translation);
            dist.x *= (r1 + r2) / d;
            dist.y *= (r1 + r2) / d;
            ct.translation = dist + ot.translation;

            // calculate projection of colliders velocity vector along distance vector between centers
            let v = vec2((x1 - x2).powi(2).sqrt(), (y1 - y2).powi(2).sqrt());

            // w parallel to v
            let wp1 = ((cv.x * v.x + cv.y * v.y) / (v.x.powi(2) + v.y.powi(2))) * v;
            // w orthogonal / perpendicular to v
            let wo1 = cv.0 - wp1;
            // momentum
            let wp2 = ((ov.x * v.x + ov.y * v.y) / (v.x.powi(2) + v.y.powi(2))) * v;
            // w orthogonal / perpendicular to v
            let wo2 = ov.0 - wp1;

            let p1 = wp1 * (PI * r1.powi(2));
            let p2 = wp2 * (PI * r2.powi(2));

            cv.0 = wp1 * -0.992 + wo1;

            // w parallel to v
            ov.0 = wp2 * -0.992 + wo2;
        }
    }
}
// Temporarily Radius will act as Mass for momentum calculation
pub fn physics_collision_system<A: Component, B: Component>(
    mut colliders: Query<(Entity, &mut Transform, &Bounding, &mut Velocity, With<A>)>,
    obstructors: Query<(Entity, &Transform, &Bounding, &Velocity, With<B>)>,
    mut rng: Local<Random>,
) {
    for (_, mut ct, cb, mut cv, _) in colliders.iter_mut() {
        let Vec3 { x: x1, y: y1, z: _ } = ct.translation;
        let r1 = cb.0;
        for (_, ot, ob, ov, _) in obstructors.iter() {
            let Vec3 { x: x2, y: y2, z: _ } = ot.translation;
            let r2 = ob.0;
            let d = ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt();
            if d < r1 + r2 {
                // calculate projection of colliders velocity vector along distance vector between centers
                let v = vec2((x1 - x2).powi(2).sqrt(), (y1 - y2).powi(2).sqrt());
                // w parallel to v
                let wp = ((cv.x * v.x + cv.y * v.y) / (v.x.powi(2) + v.y.powi(2))) * v;
                // w orthogonal / perpendicular to v
                let wo = cv.0 - wp;
                cv.0 = wp * -1.0 + wo;
            }
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
            let d = ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt();
            if d < r1 + r2 {
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
