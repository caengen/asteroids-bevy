use crate::{
    FRAME_END_X, FRAME_END_Y, FRAME_START_X, FRAME_START_Y, GAME_BORDER_OFFSET, GAME_FRAME_HEIGHT,
    GAME_FRAME_WIDTH,
};

use super::Bounding;
use bevy::prelude::*;

#[derive(Debug, Component)]
pub struct BoundaryWrap;
#[derive(Debug, Component, Default)]
pub struct BoundaryRemoval;

pub fn boundary_removal_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &Bounding, With<BoundaryRemoval>)>,
) {
    for (entity, transform, bounding, _) in query.iter_mut() {
        let Vec3 { x, y, z: _ } = transform.translation;
        let r = bounding.0;
        if x > FRAME_END_X + r
            || x < FRAME_START_X - r
            || y > FRAME_END_Y + r
            || y < FRAME_START_Y - r
        {
            commands.entity(entity).despawn();
        }
    }
}

pub fn boundary_wrapping_system(mut query: Query<(&mut Transform, &Bounding, With<BoundaryWrap>)>) {
    for (mut transform, bound, _) in query.iter_mut() {
        let r = bound.0;
        let Vec3 { x, y, z: _ } = transform.translation;

        if x > FRAME_END_X + r {
            transform.translation.x = FRAME_START_X - r;
        } else if x < FRAME_START_X - r {
            transform.translation.x = FRAME_END_X + r;
        }

        if y > FRAME_END_Y + r {
            transform.translation.y = FRAME_START_Y - r;
        } else if y < FRAME_START_Y - r {
            transform.translation.y = FRAME_END_Y + r;
        }
    }
}
