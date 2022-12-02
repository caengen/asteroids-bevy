use super::Bounding;
use bevy::prelude::*;

#[derive(Debug, Component)]
pub struct BoundaryWrap;
#[derive(Debug, Component, Default)]
pub struct BoundaryRemoval;

pub fn boundary_removal_system(
    mut commands: Commands,
    window: Res<WindowDescriptor>,
    mut query: Query<(Entity, &Transform, &Bounding, With<BoundaryRemoval>)>,
) {
    let w = window.width / 2.0;
    let h = window.height / 2.0;
    for (entity, transform, bounding, _) in query.iter_mut() {
        let Vec3 { x, y, z: _ } = transform.translation;
        let r = bounding.0;
        if x < -w - r || x > w + r || y > h + r || y < -h - r {
            commands.entity(entity).despawn();
        }
    }
}

pub fn boundary_wrapping_system(
    window: Res<WindowDescriptor>,
    mut query: Query<(&mut Transform, &Bounding, With<BoundaryWrap>)>,
) {
    for (mut transform, bound, _) in query.iter_mut() {
        let w = window.width / 2.0;
        let h = window.height / 2.0;
        let r = bound.0;
        let Vec3 { x, y, z: _ } = transform.translation;

        if x > w + r {
            transform.translation.x = -w - r;
        } else if x < -w - r {
            transform.translation.x = w + r;
        }

        if y > h + r {
            transform.translation.y = -h - r;
        } else if y < -h - r {
            transform.translation.y = h + r;
        }
    }
}
