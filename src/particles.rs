use crate::{DelayedVisibility, DARK};

use super::{
    random::Random, Color, Damping, Darken, TimedRemoval, Velocity, LIGHT, POLY_LINE_WIDTH,
};
use bevy::{
    math::{vec2, vec3},
    prelude::{
        Commands, EventReader, Local, Query, Res, Time, Transform, Vec2, Vec3, Visibility, Without,
    },
    time::Timer,
};
use bevy_prototype_lyon::{
    prelude::{DrawMode, FillMode, GeometryBuilder, StrokeMode},
    shapes,
};
use rand::Rng;
use std::{ops::Range, time::Duration};

const GRAIN_RADIUS: f32 = 0.3;
const PARTICLE_DAMPING: f32 = 0.992;

pub struct GrainParticleSpawnEvent {
    pub pos: Vec3,
    pub spawn_radius: f32,
    pub particles: Range<i32>,
    pub impact_vel: Vec2,
}

pub struct BallParticleSpawnEvent {
    pub pos: Vec3,
    pub spawn_radius: f32,
    pub ball_radius: f32,
    pub particles: Range<i32>,
    pub delay: i32,
    pub dir_vel: Vec2,
}

pub fn darken_system(
    mut query: Query<(&mut DrawMode, &mut Darken, Without<DelayedVisibility>)>,
    time: Res<Time>,
) {
    for (mut draw_mode, mut darken, _) in query.iter_mut() {
        darken.0.tick(time.delta());

        if !darken.0.finished() {
            if let DrawMode::Outlined {
                ref mut fill_mode,
                ref mut outline_mode,
            } = *draw_mode
            {
                let pc = darken.0.percent_left();
                let new_cor = Color::rgb(
                    (LIGHT.r() * pc).clamp(DARK.r(), LIGHT.r()),
                    (LIGHT.g() * pc).clamp(DARK.g(), LIGHT.g()),
                    (LIGHT.b() * pc).clamp(DARK.b(), LIGHT.b()),
                );
                fill_mode.color = new_cor;
                outline_mode.color = new_cor;
            }
        }
    }
}

pub fn ball_spawn_system(
    mut commands: Commands,
    mut rng: Local<Random>,
    mut ev_ball: EventReader<BallParticleSpawnEvent>,
) {
    for BallParticleSpawnEvent {
        pos,
        spawn_radius,
        ball_radius,
        particles,
        delay,
        dir_vel,
    } in ev_ball.iter()
    {
        let shape = shapes::Circle {
            radius: *ball_radius,
            ..Default::default()
        };

        let particles = rng.gen_range(particles.start..particles.end);
        for i in 1..=particles {
            let angle = ((rng.gen_range(1..360)) as f32).to_radians();
            // let r = rng.gen_range((*spawn_radius * 0.1)..(*spawn_radius * 0.9));
            let r = rng.gen_range(0.0..*spawn_radius);
            let particle_pos = vec3(r * f32::sin(angle), r * f32::cos(angle), 1.0);

            let append_pos = if rng.gen_ratio(2, 3) {
                *dir_vel * i as f32
            } else {
                Vec2::ZERO
            };

            let translation = vec3(
                pos.x + append_pos.x + particle_pos.x,
                pos.y + append_pos.y + particle_pos.y,
                1.0,
            );
            let duration = rng.gen_range(300..500);
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
                            translation,
                            ..Default::default()
                        },
                    )),
                )
                .insert(TimedRemoval(Timer::new(
                    Duration::from_millis(duration),
                    false,
                )))
                .insert(DelayedVisibility(Timer::new(
                    Duration::from_millis((delay * i) as u64),
                    false,
                )))
                .insert(Visibility { is_visible: false })
                .insert(Darken(Timer::new(Duration::from_millis(duration), false)));
        }
    }
}

pub fn grain_spawn_system(
    mut commands: Commands,
    mut rng: Local<Random>,
    mut ev_grain: EventReader<GrainParticleSpawnEvent>,
) {
    for GrainParticleSpawnEvent {
        pos,
        spawn_radius,
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
            let r = rng.gen_range((*spawn_radius * 0.1)..(*spawn_radius * 0.9));
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
