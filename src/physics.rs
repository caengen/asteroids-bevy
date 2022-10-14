use bevy::ecs::component::Component;
use bevy::prelude::*;
use derive_more::From;

#[derive(Debug, Component, Default, Deref, DerefMut, From)]
pub struct Velocity(pub Vec2);

#[derive(Debug, Component, Default, Deref, DerefMut, From)]
pub struct AngularVelocity(pub f32);

pub fn movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, Option<&AngularVelocity>, Option<&Velocity>)>,
) {
    for (mut transform, angular_velocity, velocity) in query.iter_mut() {
        if let Some(AngularVelocity(vel)) = angular_velocity {
            transform.rotate(Quat::from_rotation_z(vel * time.delta_seconds()))
        }
        if let Some(Velocity(vel)) = velocity {
            transform.translation.x += vel.x * time.delta_seconds();
            transform.translation.y += vel.y * time.delta_seconds();
        }
    }
}
