use crate::movement::ThrustersMode;

use super::{
    movement::{Drive, DriveMode, SideThrusters},
    random::Random,
    Color, Damping, Darken, DelayedVisibility, Shrink, TimedRemoval, Velocity, DARK, LIGHT,
    PLAYER_DAMPING, PLAYER_SIZE, POLY_LINE_WIDTH,
};
use bevy::{
    math::{vec2, vec3},
    prelude::{
        BuildChildren, Commands, Component, Entity, EventReader, Local, Query, Res, Time,
        Transform, Vec2, Vec3, Visibility, Without,
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

const EXHAUST_SIZE: f32 = 10.0;
const GAS_EXHAUST_SIZE: f32 = 15.0;
const GAS_LINE_WIDTH: f32 = 0.1;

// in seconds
const GAS_EXHAUST_TIMEOUT: f32 = 0.15;
const GAS_EXHAUST_LIVE_TIME: f32 = 0.2;
const GAS_EXHAUST_SIZE_RANGE: Range<f32> = (GAS_EXHAUST_SIZE / 1.5)..(GAS_EXHAUST_SIZE * 1.25);

const EXHAUST_TIMEOUT: f32 = 0.15;
const EXHAUST_SHRINK_TIMEOUT: f32 = 0.05;
const EXHAUST_LIVE_TIME: f32 = 2.0;

const EXHAUST_VEL_MUT: f32 = -0.3;
const EXHAUST_POS_X_RANGE: Range<f32> = -(PLAYER_SIZE / 2.0)..(PLAYER_SIZE / 2.0);
const EXHAUST_POS_Y_RANGE: Range<f32> = (PLAYER_SIZE / 1.5)..(PLAYER_SIZE);
const EXHAUST_SIZE_RANGE: Range<f32> = (EXHAUST_SIZE / 2.0)..EXHAUST_SIZE;

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

/**
 * Shrink the component by subtracting the scale vector each time the timer finishes
 */
pub fn shrink_system(mut shrinking: Query<(&mut Transform, &mut Shrink)>, time: Res<Time>) {
    for (mut transform, mut shrink) in shrinking.iter_mut() {
        shrink.0.tick(time.delta());

        if shrink.0.just_finished() && transform.scale.x > 0.0 && transform.scale.y > 0.0 {
            transform.scale *= 1.0 - (0.9 * time.delta_seconds());
        }
    }
}

/**
 * Spawns an array of growing balls that the darken with time.
 * Useful particle effect for explosions... maybe.
 */
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

/**
 * Particle effect used for explosions or impacts. Very versatile.
 */
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

#[derive(Component)]
pub struct ExhaustTimer(pub Timer);

#[derive(Component)]
pub struct GasExhaustTimer(pub Timer);

pub fn gas_exhaust_system(
    mut gas_valves: Query<(Entity, &SideThrusters, Option<&mut GasExhaustTimer>)>,
    mut commands: Commands,
    mut rng: Local<Random>,
    time: Res<Time>,
) {
    for (entity, side_thrusters, timer) in gas_valves.iter_mut() {
        match side_thrusters.mode {
            ThrustersMode::Off => {
                if let Some(_) = timer {
                    commands.entity(entity).remove::<GasExhaustTimer>();
                }

                return;
            }
            _ => {
                if let Some(mut timer) = timer {
                    timer.0.tick(time.delta());
                    if (timer.0.just_finished()) {
                        let size = rng.gen_range(GAS_EXHAUST_SIZE_RANGE);
                        let start_x = PLAYER_SIZE / 2.0;
                        let start_y = rng.gen_range(-0.5..0.5);
                        let line = match side_thrusters.mode {
                            ThrustersMode::Left => shapes::Line(
                                vec2(-start_x - size, start_y),
                                vec2(-start_x, start_y),
                            ),
                            _ => {
                                shapes::Line(vec2(start_x, start_y), vec2(start_x + size, start_y))
                            }
                        };

                        let geo = GeometryBuilder::default().add(&line).build(
                            DrawMode::Outlined {
                                outline_mode: StrokeMode::new(LIGHT, POLY_LINE_WIDTH),
                                fill_mode: FillMode::color(LIGHT),
                            },
                            Transform {
                                translation: vec3(0.0, 0.0, 1.0),
                                ..Default::default()
                            },
                        );

                        let gas = commands
                            .spawn()
                            .insert_bundle(geo)
                            .insert(TimedRemoval(Timer::from_seconds(
                                GAS_EXHAUST_LIVE_TIME,
                                false,
                            )))
                            .id();
                        commands.entity(entity).insert_children(0, &[gas]);
                    }
                } else {
                    commands
                        .entity(entity)
                        .insert(GasExhaustTimer(Timer::from_seconds(
                            GAS_EXHAUST_TIMEOUT,
                            true,
                        )));
                }
            }
        }
    }
}

/**
 * Particle effect that creates a cross particle that shrinks and fades
 */
pub fn propulsion_exhaust_system(
    mut drive_engines: Query<(
        Entity,
        &Velocity,
        &Transform,
        &Drive,
        Option<&mut ExhaustTimer>,
    )>,
    mut commands: Commands,
    mut rng: Local<Random>,
    time: Res<Time>,
) {
    for (entity, velocity, transform, drive, timer) in drive_engines.iter_mut() {
        if drive.mode == DriveMode::Off {
            if let Some(_) = timer {
                commands.entity(entity).remove::<ExhaustTimer>();
            }

            return;
        }

        if let Some(mut timer) = timer {
            timer.0.tick(time.delta());
            if (timer.0.just_finished()) {
                let rot = transform.rotation;
                let pos = vec3(
                    rng.gen_range(EXHAUST_POS_X_RANGE),
                    rng.gen_range(EXHAUST_POS_Y_RANGE),
                    1.0,
                );

                let size = rng.gen_range(EXHAUST_SIZE_RANGE);
                let horisontal = shapes::Line(vec2(-size / 2.0, 0.0), vec2(size / 2.0, 0.0));
                let vertical = shapes::Line(vec2(0.0, -size / 2.0), vec2(0.0, size / 2.0));

                let exhaust = GeometryBuilder::default()
                    .add(&horisontal)
                    .add(&vertical)
                    .build(
                        DrawMode::Outlined {
                            outline_mode: StrokeMode::new(LIGHT, POLY_LINE_WIDTH),
                            fill_mode: FillMode::color(LIGHT),
                        },
                        Transform {
                            translation: transform.translation + (rot * pos),
                            ..Default::default()
                        },
                    );

                commands
                    .spawn()
                    .insert_bundle(exhaust)
                    .insert(Velocity(velocity.0 * EXHAUST_VEL_MUT))
                    .insert(TimedRemoval(Timer::from_seconds(EXHAUST_LIVE_TIME, false)))
                    .insert(Damping::from(PLAYER_DAMPING))
                    .insert(Shrink(Timer::from_seconds(EXHAUST_SHRINK_TIMEOUT, true)));
            }
        } else {
            commands
                .entity(entity)
                .insert(ExhaustTimer(Timer::from_seconds(EXHAUST_TIMEOUT, true)));
        }
    }
}
