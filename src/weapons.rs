use std::time::Duration;

use crate::asteroid::Damage;

use super::{BoundaryRemoval, Bounding, Velocity, LIGHT, POLY_LINE_WIDTH};
use bevy::{
    math::{vec2, vec3},
    prelude::*,
};
use bevy_prototype_lyon::{
    entity::ShapeBundle,
    prelude::{DrawMode, FillMode, GeometryBuilder, StrokeMode},
    shapes,
};
use derive_more::From;

pub const CANNON_BULLET_RADIUS: f32 = 1.0;

#[derive(Debug, Component)]
pub struct Bullet(pub Timer);

#[derive(Debug, Component, Default, Deref, DerefMut, From)]
pub struct Cannon(pub f32);

pub fn cannon_control_system(
    mut commands: Commands,
    query: Query<(&Transform, &Bounding, &Cannon)>,
    keyboard: Res<Input<KeyCode>>,
) {
    for (transform, bounding, cannon) in query.iter() {
        if keyboard.just_pressed(KeyCode::Space) {
            let direction = transform.rotation * -Vec3::Y; //TODO: find out why this works
            let shape = shapes::Circle {
                radius: CANNON_BULLET_RADIUS,
                ..Default::default()
            };

            let _bullet = commands.spawn().insert_bundle(BulletBundle {
                bounding: Bounding::from(CANNON_BULLET_RADIUS),
                velocity: Velocity::from(vec2(cannon.0 * direction.x, cannon.0 * direction.y)),
                bullet: Bullet(Timer::new(Duration::from_millis(1250), false)),
                boundary_removal: BoundaryRemoval,
                damage: Damage(10.0),
                shape: (GeometryBuilder::build_as(
                    &shape,
                    DrawMode::Outlined {
                        outline_mode: StrokeMode::new(LIGHT, POLY_LINE_WIDTH),
                        fill_mode: FillMode::color(LIGHT),
                    },
                    Transform {
                        translation: transform.translation
                            + vec3(direction.x * bounding.0, direction.y * bounding.0, 0.0),
                        ..Default::default()
                    },
                )),
            });
            // .insert(Bounding::from(CANNON_BULLET_RADIUS))
            // .insert(BoundaryRemoval)
            // .insert(Velocity::from(vec2(
            //     cannon.0 * direction.x,
            //     cannon.0 * direction.y,
            // )))
            // .insert(Bullet(Timer::new(Duration::from_millis(1250), false)));
        }
    }
}

#[derive(Bundle)]

struct BulletBundle {
    bounding: Bounding,
    boundary_removal: BoundaryRemoval,
    velocity: Velocity,
    damage: Damage,
    bullet: Bullet,
    #[bundle]
    shape: ShapeBundle,
}

pub fn bullet_despawn_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Bullet)>,
) {
    for (entity, mut bullet) in query.iter_mut() {
        bullet.0.tick(time.delta());
        if bullet.0.finished() {
            commands.entity(entity).despawn();
        }
    }
}
