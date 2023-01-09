use super::{random::Random, Damping, TimedRemoval, Velocity};
use bevy::{
    math::{vec2, vec3},
    prelude::{Commands, EventReader, Local, Transform, Vec2, Vec3},
    time::Timer,
};
use bevy_prototype_lyon::{
    prelude::{DrawMode, FillMode, GeometryBuilder, StrokeMode},
    shapes,
};
use rand::Rng;
use std::{ops::Range, time::Duration};

use crate::{LIGHT, POLY_LINE_WIDTH};

pub const GRAIN_RADIUS: f32 = 0.3;
pub const PARTICLE_DAMPING: f32 = 0.992;

pub struct GrainSpawnEvent {
    pub pos: Vec3,
    pub radius: f32,
    pub particles: Range<i32>,
    pub impact_vel: Vec2,
}

pub fn grain_spawn_system(
    mut commands: Commands,
    mut rng: Local<Random>,
    mut ev_grain: EventReader<GrainSpawnEvent>,
) {
    for GrainSpawnEvent {
        pos,
        radius,
        particles,
        impact_vel,
    } in ev_grain.iter()
    {
        let shape = shapes::Circle {
            radius: GRAIN_RADIUS,
            ..Default::default()
        };

        let particles = rng.gen_range(particles.start..particles.end);

        for i in 1..=particles {
            let angle = ((i * (360 / particles)) as f32).to_radians();
            let r = rng.gen_range((*radius * 0.1)..(*radius * 0.9));
            let particle_pos = vec3(r * f32::sin(angle), r * f32::cos(angle), 1.0);
            let force = rng.gen_range(20.0..90.0);
            let vel = vec2(
                impact_vel.x + f32::sin(angle) * force,
                impact_vel.y + f32::cos(angle) * force,
            );
            commands
                .spawn()
                .insert_bundle(
                    (GeometryBuilder::build_as(
                        &shape,
                        DrawMode::Outlined {
                            outline_mode: StrokeMode::new(LIGHT, POLY_LINE_WIDTH),
                            fill_mode: FillMode::color(LIGHT),
                        },
                        Transform {
                            translation: vec3(pos.x + particle_pos.x, pos.y + particle_pos.y, 1.0),
                            ..Default::default()
                        },
                    )),
                )
                .insert(TimedRemoval(Timer::new(
                    Duration::from_millis(rng.gen_range(300..1500)),
                    false,
                )))
                .insert(Velocity::from(vel))
                .insert(Damping::from(PARTICLE_DAMPING));
        }
    }
}
