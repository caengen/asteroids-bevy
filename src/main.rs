use bevy::ecs::component::Component;
use bevy::{
    math::{const_vec2, vec2, vec3},
    prelude::*,
    time::FixedTimestep,
    transform,
    window::PresentMode,
};
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_inspector_egui::{Inspectable, InspectorPlugin};
use bevy_prototype_lyon::{
    prelude::{
        tess::{geom::Rotation, math::Angle},
        *,
    },
    shapes::{Circle, Polygon},
};
use collision::*;
use derive_more::From;
use physics::*;
use rand::Rng;
use random::{Random, RandomPlugin};
use std::ops::RangeInclusive;
use std::{default::Default, f32::consts::PI, ops::Range, time::Duration};

mod collision;
mod physics;
mod random;

const SCREEN_HEIGHT: f32 = 640.0;
const SCREEN_WIDTH: f32 = 960.0;
pub const SCREEN: Vec2 = Vec2::from_array([SCREEN_WIDTH, SCREEN_HEIGHT]);
// pub const TIME_STEP: f32 = 1.0 / 60.0;
pub const GAME_WIDTH: f32 = 240.0;
// pub const PIXELS_PER_METER: f32 = 30.0 / SCALE;
pub const CANNON_BULLET_RADIUS: f32 = 1.0;
pub const PARTICLE_RADIUS: f32 = 0.3;

pub const PLAYER_SIZE: f32 = 20.0;
pub const PLAYER_DAMPING: f32 = 0.992;
pub const PARTICLE_DAMPING: f32 = 0.992;
pub const POLY_LINE_WIDTH: f32 = 1.0;
pub const ASTEROID_LINE_WIDTH: f32 = 3.0;

pub const DARK: (f32, f32, f32) = (49.0, 47.0, 40.0);
pub const LIGHT: (f32, f32, f32) = (218.0, 216.0, 209.0);

pub const ASTEROID_SIZES: (
    RangeInclusive<f32>,
    RangeInclusive<f32>,
    RangeInclusive<f32>,
) = (60.0..=80.0, 30.0..=50.00, 10.0..=20.0);

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemLabel)]
enum System {
    Collision,
    Input,
    Movement,
    Boundary,
    Spawning,
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "asteroids-bevy".to_string(),
            present_mode: PresentMode::Fifo,
            width: SCREEN_WIDTH,
            height: SCREEN_HEIGHT,
            ..default()
        })
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Msaa { samples: 4 })
        .add_event::<AsteroidSpawnEvent>()
        .add_event::<HitEvent>()
        .add_event::<PlayerDeathEvent>()
        .add_event::<ExplosionEvent>()
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_plugin(RandomPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(setup_system)
        .add_system_set(
            SystemSet::new()
                .label(System::Input)
                .with_system(steering_control_system)
                .with_system(drive_control_system)
                .with_system(cannon_control_system),
        )
        .add_system_set(
            SystemSet::new()
                .label(System::Movement)
                .with_system(movement_system)
                .with_system(drive_system)
                .with_system(damping_system)
                .after(System::Input),
        )
        .add_system_set(
            SystemSet::new()
                .label(System::Boundary)
                .with_system(boundary_removal_system)
                .with_system(bullet_despawn_system)
                .after(System::Movement),
        )
        .add_system(boundary_wrapping_system)
        .add_system_set(
            SystemSet::new()
                .label(System::Collision)
                .with_system(collision_system::<Bullet, Asteroid>)
                .with_system(collision_system::<Asteroid, Bullet>)
                .with_system(collision_system::<Asteroid, Ship>)
                .after(System::Boundary),
        )
        .add_system(self_collision_system::<Asteroid>.before(System::Movement))
        .add_system(hit_system.after(System::Collision))
        .add_system(explosion_system.after(System::Collision))
        .add_system(asteroid_spawn_system.with_run_criteria(FixedTimestep::step(0.5)))
        .add_system(asteroid_generation_system)
        .add_system(timed_removal_system.after(System::Movement))
        .add_system(player_state_system)
        .add_system(flick_system)
        .run();
}

fn player_state_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Ship, &mut Transform, &mut Visibility)>,
    mut ev_death: EventReader<PlayerDeathEvent>,
) {
    let delta = time.delta();
    for (entity, mut ship, mut transform, mut visibility) in query.iter_mut() {
        match ship.state {
            ShipState::Spawning => {
                ship.timer.tick(delta);
                if ship.timer.just_finished() {
                    commands
                        .entity(entity)
                        .remove::<Flick>()
                        .insert(Bounding::from(PLAYER_SIZE / 2.0))
                        .insert(SteeringControl::from(Angle::degrees(180.0)))
                        .insert(Drive::new(3.0))
                        .insert(Cannon::from(400.0))
                        .insert(Velocity::default())
                        .insert(AngularVelocity::default());
                    ship.state = ShipState::Alive;
                    visibility.is_visible = true;
                }
            }
            ShipState::Dead => {
                ship.timer.tick(delta);
                if ship.timer.just_finished() {
                    transform.rotation = Quat::from_rotation_z(180.0_f32.to_radians());
                    transform.translation.x = 0.0;
                    transform.translation.y = 0.0;
                    commands.entity(entity).insert(Flick {
                        duration: Timer::new(Duration::from_secs(2), false),
                        switch_timer: Timer::new(Duration::from_millis(200), true),
                    });
                    *ship = Ship {
                        state: ShipState::Spawning,
                        timer: Timer::from_seconds(1.0, false),
                    };
                    visibility.is_visible = true;
                }
            }
            ShipState::Alive => {
                for _PlayerDeathEvent in ev_death.iter() {
                    commands
                        .entity(entity)
                        .remove::<Bounding>()
                        .remove::<SteeringControl>()
                        .remove::<Drive>()
                        .remove::<Cannon>()
                        .remove::<Velocity>()
                        .remove::<AngularVelocity>();
                    *ship = Ship {
                        state: ShipState::Dead,
                        timer: Timer::from_seconds(2.0, false),
                    };
                    visibility.is_visible = false;
                }
            }
        }
    }
}

fn setup_system(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
    let shape = shapes::Polygon {
        points: ship_points(),
        closed: false,
    };
    let mut player = commands
        .spawn()
        .insert_bundle(
            (GeometryBuilder::build_as(
                &shape,
                DrawMode::Outlined {
                    outline_mode: StrokeMode::new(Color::WHITE, POLY_LINE_WIDTH),
                    fill_mode: FillMode::color(Color::WHITE),
                },
                Transform {
                    rotation: Quat::from_rotation_z(180.0_f32.to_radians()),
                    ..Default::default()
                },
            )),
        )
        .insert(Ship {
            state: ShipState::Spawning,
            timer: Timer::new(Duration::from_millis(1), false),
        })
        .insert(Flick {
            duration: Timer::new(Duration::from_secs(2), false),
            switch_timer: Timer::new(Duration::from_millis(200), true),
        })
        .insert(BoundaryWrap)
        .insert(Velocity::default())
        .insert(AngularVelocity::default())
        .insert(Damping::from(PLAYER_DAMPING));
}

fn hit_system(mut commands: Commands, mut ev_hit: EventReader<HitEvent>) {
    for HitEvent { entity } in ev_hit.iter() {
        commands.entity(*entity).despawn_recursive();
    }
}
fn explosion_system(
    mut commands: Commands,
    mut rng: Local<Random>,
    mut ev_explode: EventReader<ExplosionEvent>,
) {
    for ExplosionEvent {
        pos,
        radius,
        particles,
        impact_vel,
    } in ev_explode.iter()
    {
        let shape = shapes::Circle {
            radius: PARTICLE_RADIUS,
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
                            outline_mode: StrokeMode::new(Color::WHITE, POLY_LINE_WIDTH),
                            fill_mode: FillMode::color(Color::WHITE),
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

fn timed_removal_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut TimedRemoval)>,
) {
    for (entity, mut removal) in query.iter_mut() {
        removal.0.tick(time.delta());

        if removal.0.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub struct HitEvent {
    entity: Entity,
}

pub struct ExplosionEvent {
    pub pos: Vec3,
    pub radius: f32,
    pub particles: Range<i32>,
    pub impact_vel: Vec2,
}
pub struct AsteroidSpawnEvent {
    pub pos: Vec2,
    pub radius: f32,
    pub amount: i32,
}

pub struct PlayerDeathEvent {}

fn asteroid_spawn_system(
    window: Res<WindowDescriptor>,
    mut rng: Local<Random>,
    mut ev_asteroid_spawn: EventWriter<AsteroidSpawnEvent>,
) {
    if !rng.gen_bool(1.0 / 4.0) {
        return;
    }

    let h = window.height / 2.0;
    let w = window.width / 2.0;

    let size = rng.gen_range(0..=10);
    let radius = match size {
        0..=3 => rng.gen_range(ASTEROID_SIZES.0),
        4..=6 => rng.gen_range(ASTEROID_SIZES.1),
        7..=9 => rng.gen_range(ASTEROID_SIZES.2),
        _ => rng.gen_range(ASTEROID_SIZES.0),
    };

    let side = rng.gen_range(0..=3);
    let pos = match side {
        0 => vec2(-w, rng.gen_range(-h..h)),
        1 => vec2(w, rng.gen_range(-h..h)),
        2 => vec2(rng.gen_range(-w..w), -h),
        _ => vec2(rng.gen_range(-w..w), h),
    };

    let amount = 1;
    ev_asteroid_spawn.send(AsteroidSpawnEvent {
        amount,
        pos,
        radius,
    });
}

fn asteroid_generation_system(
    mut commands: Commands,
    mut rng: Local<Random>,
    mut ev_asteroid_spawn: EventReader<AsteroidSpawnEvent>,
) {
    for AsteroidSpawnEvent {
        amount,
        pos,
        radius,
    } in ev_asteroid_spawn.iter()
    {
        for i in 0..*amount {
            let pos = if *amount > 0 {
                let angle = ((360 / *amount * i) as f32).to_radians();
                vec2(
                    pos.x + *radius * 1.25 * angle.sin(),
                    pos.y + *radius * 1.25 * angle.cos(),
                )
            } else {
                *pos
            };

            let edges = rng.gen_range(7..12);

            let mut points = Vec::new();
            let angle_inc = 360.0 / edges as f32;

            for i in 1..=edges {
                let r = match *radius as usize {
                    60..=80 => rng.gen_range(60..=(*radius as i32)),
                    30..=50 => rng.gen_range(30..=(*radius as i32)),
                    _ => rng.gen_range(10..=(*radius as i32)),
                } as f32;

                let angle = (angle_inc * i as f32).to_radians();
                points.push(vec2(r * angle.sin(), r * angle.cos()));
            }
            let p_len = points.len();
            let average = points
                .clone()
                .iter()
                .map(|p| ((p.x).powi(2) + (p.y).powi(2)).sqrt())
                .sum::<f32>()
                / p_len as f32;
            let bounding = average;
            let shape = shapes::Polygon {
                points,
                closed: true,
            };

            let center = vec3(pos.x, pos.y, 1.0);
            let vel = match *radius as usize {
                60..=80 => {
                    let dest = vec3(1.0, 1.0, 1.0);
                    let angle = center.angle_between(dest);
                    let direction = Quat::from_rotation_z(angle) * -Vec3::Y; //TODO: find out why this works
                    let force = rng.gen_range(10.0..50.00);
                    vec2(force * direction.x, force * direction.y)
                }
                40..=50 => {
                    let direction =
                        Quat::from_rotation_z((rng.gen_range(0..360) as f32).to_radians())
                            * -Vec3::Y; //TODO: find out why this works
                    let force = rng.gen_range(20.0..60.00);
                    vec2(force * direction.x, force * direction.y)
                }
                _ => {
                    let direction =
                        Quat::from_rotation_z((rng.gen_range(0..360) as f32).to_radians())
                            * -Vec3::Y; //TODO: find out why this works
                    let force = rng.gen_range(30.0..70.00);
                    vec2(force * direction.x, force * direction.y)
                }
            };

            let d_circle = shapes::Circle {
                radius: average,
                ..Default::default()
            };
            let debug_bound = commands
                .spawn()
                .insert_bundle(
                    (GeometryBuilder::build_as(
                        &d_circle,
                        DrawMode::Outlined {
                            outline_mode: StrokeMode::new(Color::RED, POLY_LINE_WIDTH * 1.5),
                            fill_mode: FillMode::color(Color::NONE),
                        },
                        Transform::default(),
                    )),
                )
                .id();
            let mut asteroid = commands
                .spawn()
                .insert_bundle(
                    (GeometryBuilder::build_as(
                        &shape,
                        DrawMode::Outlined {
                            outline_mode: StrokeMode::new(Color::WHITE, POLY_LINE_WIDTH * 1.5),
                            fill_mode: FillMode::color(Color::NONE),
                        },
                        Transform::default().with_translation(center),
                    )),
                )
                .insert(Bounding::from(bounding))
                .insert(BoundaryRemoval)
                .insert(Velocity::from(vel))
                .insert(AngularVelocity::from(rng.gen_range(0.5..2.0)))
                .insert(Asteroid)
                .insert_children(0, &[debug_bound]);
        }
    }
}

pub fn polygon(origo: Vec2, r: f32, amount: i32) -> Vec<Vec2> {
    let mut points = Vec::new();
    let angle_inc = 360.0 / amount as f32;

    for i in 1..=amount {
        let rot = (angle_inc * i as f32).to_radians();
        points.push(vec2(origo.x + r * rot.sin(), origo.y - r * rot.cos()));
    }

    points
}

fn boundary_removal_system(
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

fn boundary_wrapping_system(
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

fn cannon_control_system(
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

            let _bullet = commands
                .spawn()
                .insert_bundle(
                    (GeometryBuilder::build_as(
                        &shape,
                        DrawMode::Outlined {
                            outline_mode: StrokeMode::new(Color::WHITE, POLY_LINE_WIDTH),
                            fill_mode: FillMode::color(Color::WHITE),
                        },
                        Transform {
                            translation: transform.translation
                                + vec3(direction.x * bounding.0, direction.y * bounding.0, 0.0),
                            ..Default::default()
                        },
                    )),
                )
                .insert(Bounding::from(CANNON_BULLET_RADIUS))
                .insert(BoundaryRemoval)
                .insert(Velocity::from(vec2(
                    cannon.0 * direction.x,
                    cannon.0 * direction.y,
                )))
                .insert(Bullet(Timer::new(Duration::from_millis(1250), false)));
        }
    }
}

fn bullet_despawn_system(
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

fn drive_control_system(mut query: Query<&mut Drive>, keyboard: Res<Input<KeyCode>>) {
    for mut drive in query.iter_mut() {
        drive.on = keyboard.pressed(KeyCode::Up);
    }
}

fn drive_system(mut query: Query<(&mut Velocity, &Transform, &Drive)>) {
    for (mut velocity, transform, drive) in query.iter_mut() {
        if !drive.on {
            return;
        }

        // what the fuck is this quat shit
        // changed from Vec3::X to -Vec::Y and now this shit works wtf?
        let direction = transform.rotation * -Vec3::Y;
        velocity.x += direction.x * drive.force;
        velocity.y += direction.y * drive.force;
    }
}

fn steering_control_system(
    mut query: Query<(&mut AngularVelocity, &SteeringControl)>,
    keyboard: Res<Input<KeyCode>>,
) {
    for (mut angular_velocity, steering_control) in query.iter_mut() {
        if keyboard.pressed(KeyCode::Left) {
            *angular_velocity = AngularVelocity::from(steering_control.0.get());
        } else if keyboard.pressed(KeyCode::Right) {
            *angular_velocity = AngularVelocity::from(-steering_control.0.get());
        } else {
            *angular_velocity = AngularVelocity::from(0.0);
        }
    }
}

pub fn ship_points() -> Vec<Vec2> {
    let rot = 0.0_f32.to_radians();
    let h = PLAYER_SIZE; // ship height
    let w = PLAYER_SIZE; // ship width

    let v1 = vec2(rot.sin() * h / 2., -rot.cos() * h / 2.);
    let v2 = vec2(
        -rot.cos() * w / 2. - rot.sin() * h / 2.,
        -rot.sin() * w / 2. + rot.cos() * h / 2.,
    );
    let v3 = vec2(
        rot.cos() * w / 2. - rot.sin() * h / 2.,
        rot.sin() * w / 2. + rot.cos() * h / 2.,
    );
    let v4 = vec2(
        -rot.cos() * w / 1.5 - rot.sin() * h / 1.5,
        -rot.sin() * w / 1.5 + rot.cos() * h / 1.5,
    );
    let v5 = vec2(
        rot.cos() * w / 1.5 - rot.sin() * h / 1.5,
        rot.sin() * w / 1.5 + rot.cos() * h / 1.5,
    );

    vec![v1, v2, v4, v2, v3, v5, v3, v1]
}

fn flick_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Visibility, &mut Flick)>,
    time: Res<Time>,
) {
    for (entity, mut visibility, mut flick) in query.iter_mut() {
        flick.duration.tick(time.delta());
        flick.switch_timer.tick(time.delta());

        if flick.duration.finished() {
            visibility.is_visible = true;
            commands.entity(entity).remove::<Flick>();
        } else if flick.switch_timer.just_finished() {
            visibility.is_visible = !visibility.is_visible;
        }
    }
}

#[derive(Debug, Component)]
pub struct Asteroid;
#[derive(Debug, Component, Default, From)]
pub struct Flick {
    pub switch_timer: Timer,
    pub duration: Timer,
}

#[derive(Debug, Component, Default)]
pub struct Ship {
    pub state: ShipState,
    pub timer: Timer,
}

#[derive(Debug, Component, Default, Deref, DerefMut, From)]
pub struct Cannon(pub f32);

#[derive(Debug, Component)]
pub struct Drive {
    pub on: bool,
    pub force: f32,
}
impl Drive {
    pub fn new(force: f32) -> Self {
        Drive {
            on: false,
            force: force,
        }
    }
}

#[derive(Debug, Component)]
pub struct TimedRemoval(pub Timer);

#[derive(Debug, Component)]
pub struct Bullet(pub Timer);

#[derive(Debug, Component)]
pub struct BoundaryWrap;
#[derive(Debug, Component)]
pub struct BoundaryRemoval;

#[derive(Debug, Component, Default, Deref, DerefMut, From)]
pub struct SteeringControl(Angle);

// impl Ship {
//     fn alive() -> Self {
//         Ship {
//             state: ShipState::Alive,
//         }
//     }

//     fn dead(duration: Duration) -> Self {
//         Ship {
//             state: ShipState::Dead(duration.),
//         }
//     }

//     fn spawn(duration: Duration) -> Self {
//         Ship {
//             state: ShipState::Spawning(duration),
//         }
//     }
// }

#[derive(Debug, Clone)]
pub enum ShipState {
    Alive,
    Dead,
    Spawning,
}

impl Default for ShipState {
    fn default() -> Self {
        ShipState::Alive
    }
}
