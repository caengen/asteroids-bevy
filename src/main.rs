use asteroid::*;
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
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::{
    prelude::{tess::math::Angle, *},
    shapes::{Circle, Polygon},
};
use boundary::*;
use collision::*;
use derive_more::From;
use movement::*;
use rand::Rng;
use random::{Random, RandomPlugin};
use std::{default::Default, ops::Range, time::Duration};
use std::{env, process};
use weapons::*;

mod asteroid;
mod boundary;
mod collision;
mod movement;
mod random;
mod weapons;

const SCREEN_HEIGHT: f32 = 640.0;
const SCREEN_WIDTH: f32 = 960.0;
pub const SCREEN: Vec2 = Vec2::from_array([SCREEN_WIDTH, SCREEN_HEIGHT]);
// pub const TIME_STEP: f32 = 1.0 / 60.0;
pub const GAME_WIDTH: f32 = 240.0;
// pub const PIXELS_PER_METER: f32 = 30.0 / SCALE;

pub const PARTICLE_RADIUS: f32 = 0.3;

pub const PLAYER_SIZE: f32 = 20.0;
pub const PLAYER_DAMPING: f32 = 0.992;
pub const PARTICLE_DAMPING: f32 = 0.992;
pub const POLY_LINE_WIDTH: f32 = 1.0;

pub const DARK: Color = Color::rgb(0.191, 0.184, 0.156);
pub const LIGHT: Color = Color::rgb(0.852, 0.844, 0.816);

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemLabel)]
enum System {
    Collision,
    Input,
    Movement,
    Boundary,
    Spawning,
}

#[derive(Default)]
struct ProgramConfig {
    debug: bool,
}

pub struct Debug(pub bool);

impl ProgramConfig {
    fn build(args: &[String]) -> Result<ProgramConfig, &'static str> {
        let mut cfg = ProgramConfig::default();
        if args.len() == 0 {
            return Ok(cfg);
        }

        for arg in args {
            match arg.as_ref() {
                "-d" | "--debug" => {
                    cfg.debug = true;
                }
                _ => return Err("unknown argument"),
            }
        }

        Ok(cfg)
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let cfg = ProgramConfig::build(&args).unwrap_or_else(|err| {
        println!("A problem occured when parsing args: {err}");
        process::exit(1);
    });

    let mut app = App::new();
    app.insert_resource(WindowDescriptor {
        title: "asteroids-bevy".to_string(),
        present_mode: PresentMode::Fifo,
        width: SCREEN_WIDTH,
        height: SCREEN_HEIGHT,
        ..default()
    })
    .insert_resource(ClearColor(DARK))
    .insert_resource(Msaa { samples: 4 })
    .insert_resource(Debug(cfg.debug))
    .add_event::<AsteroidSpawnEvent>()
    .add_event::<DestructionEvent>()
    .add_event::<PlayerDeathEvent>()
    .add_event::<ExplosionEvent>()
    .add_event::<DamageTransferEvent>()
    .add_plugins(DefaultPlugins)
    .add_plugin(ShapePlugin)
    .add_plugin(RandomPlugin)
    .add_startup_system(setup_system)
    .add_startup_system(setup_stars)
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
            .with_system(damage_transfer_system::<Asteroid>)
            .with_system(kill_collision_system::<Asteroid, Ship>)
            .with_system(self_collision_system::<Asteroid>)
            .after(System::Boundary),
    )
    .add_system(destruction_system.after(System::Collision))
    .add_system(explosion_system.after(System::Collision))
    .add_system(asteroid_spawn_system.with_run_criteria(FixedTimestep::step(0.5)))
    .add_system(asteroid_generation_system)
    .add_system(timed_removal_system.after(System::Movement))
    .add_system(player_state_system)
    .add_system(flick_system);

    if cfg.debug {
        app.add_plugin(WorldInspectorPlugin::new());
    }

    app.run();
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

#[derive(Bundle)]
struct StarBundle {
    #[bundle]
    shape: ShapeBundle,
    // flick: Flick, blink system?
}

fn setup_stars(mut commands: Commands, window: Res<WindowDescriptor>, mut rng: Local<Random>) {
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

    for i in 0..200 {
        let pos = vec2(rng.gen_range(-w..w), rng.gen_range(-h..h));

        let shape = shapes::Circle {
            radius: rng.gen_range(0.01..CANNON_BULLET_RADIUS),
            ..Default::default()
        };

        let flick = rng.gen_range(3000..10000);
        let _star = commands.spawn().insert_bundle(StarBundle {
            // flick: Flick {
            //     duration: Timer::new(Duration::from_millis(flick + 100), true),
            //     switch_timer: Timer::new(Duration::from_millis(flick), true),
            // },
            shape: (GeometryBuilder::build_as(
                &shape,
                DrawMode::Outlined {
                    outline_mode: StrokeMode::new(LIGHT, POLY_LINE_WIDTH),
                    fill_mode: FillMode::color(LIGHT),
                },
                Transform {
                    translation: vec3(pos.x, pos.y, 0.0),
                    ..Default::default()
                },
            )),
        });
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
                    outline_mode: StrokeMode::new(LIGHT, POLY_LINE_WIDTH),
                    fill_mode: FillMode::color(LIGHT),
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
        .insert(SpeedLimit::from(200.0))
        .insert(AngularVelocity::default())
        .insert(Damping::from(PLAYER_DAMPING));
}

fn destruction_system(mut commands: Commands, mut ev_hit: EventReader<DestructionEvent>) {
    for DestructionEvent { entity } in ev_hit.iter() {
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

pub struct DestructionEvent {
    entity: Entity,
}

pub struct DamageTransferEvent {
    victim: Entity,
    damage: f32,
}

pub struct PlayerDeathEvent {}
pub struct ExplosionEvent {
    pub pos: Vec3,
    pub radius: f32,
    pub particles: Range<i32>,
    pub impact_vel: Vec2,
}

pub fn polygon(center: Vec2, r: f32, amount: i32) -> Vec<Vec2> {
    let mut points = Vec::new();
    let angle_inc = 360.0 / amount as f32;

    for i in 1..=amount {
        let rot = (angle_inc * i as f32).to_radians();
        points.push(vec2(center.x + r * rot.sin(), center.y - r * rot.cos()));
    }

    points
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

#[derive(Debug, Component)]
pub struct TimedRemoval(pub Timer);

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
