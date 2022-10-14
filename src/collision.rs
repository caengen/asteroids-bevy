use super::{
    random::{Random, RandomPlugin},
    Asteroid, AsteroidSizes, AsteroidSpawnEvent, ExplosionEvent, HitEvent, PlayerDeathEvent, Ship,
    ShipState, Velocity, ASTEROID_SIZES,
};
use bevy::ecs::component::Component;
use bevy::math::vec2;
use bevy::prelude::*;
use derive_more::From;
use rand::Rng;

#[derive(Debug, Component, Default, Deref, DerefMut, From)]
pub struct Bounding(pub f32);

// TODO
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
