use bevy::{
    math::{const_vec2, vec2, vec3},
    prelude::*,
    time::FixedTimestep,
    transform,
    window::PresentMode,
};
use bevy_prototype_lyon::{
    entity::ShapeBundle,
    prelude::{
        tess::{geom::Rotation, math::Angle},
        *,
    },
    shapes::{Circle, Polygon},
};
use derive_more::From;
use rand::Rng;
use random::{Random, RandomPlugin};
use std::{default::Default, f32::consts::PI, ops::Range, time::Duration};

mod random;

const SCREEN_HEIGHT: f32 = 640.0;
const SCREEN_WIDTH: f32 = 960.0;
pub const SCREEN: Vec2 = Vec2::from_array([SCREEN_WIDTH, SCREEN_HEIGHT]);
// pub const TIME_STEP: f32 = 1.0 / 60.0;
pub const GAME_WIDTH: f32 = 240.0;
pub const SCALE: f32 = SCREEN_WIDTH / GAME_WIDTH;
// pub const PIXELS_PER_METER: f32 = 30.0 / SCALE;
pub const CANNON_BULLET_RADIUS: f32 = 0.25;

pub const PLAYER_DAMPING: f32 = 0.998;
pub const POLY_LINE_WIDTH: f32 = 0.075;

pub const DARK: (f32, f32, f32) = (49.0, 47.0, 40.0);
pub const LIGHT: (f32, f32, f32) = (218.0, 216.0, 209.0);

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "asteroids-bevy".to_string(),
            present_mode: PresentMode::Fifo,
            width: SCREEN.x,
            height: SCREEN.y,
            ..default()
        })
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(AsteroidSizes {
            big: 20.0..25.0,
            medium: 10.0..15.0,
            small: 5.0..7.5,
        })
        .add_event::<AsteroidSpawnEvent>()
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_plugin(RandomPlugin)
        .add_startup_system(setup_system)
        .add_system(steering_control_system)
        .add_system(movement_system)
        .add_system(drive_control_system)
        .add_system(drive_system)
        .add_system(damping_system)
        .add_system(boundary_wrapping_system)
        .add_system(asteroid_spawn_system.with_run_criteria(FixedTimestep::step(0.5)))
        .add_system(asteroid_generation_system)
        .add_system(cannon_control_system)
        .add_system(boundary_removal_system)
        // .add_system(flick_system)
        // .add_system(player_state_system)
        .run();
}

// fn asteroid_setup_system(mut commands: Commands) {
//     let amount = 10;
//     let angle_increment = 360.0 / amount as f32;

//     for i in 0..amount {}
// }

fn setup_system(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
    let shape = shapes::Polygon {
        points: scaled_ship_points(),
        closed: false,
    };
    let mut player = commands.spawn();
    player
        .insert_bundle(
            (GeometryBuilder::build_as(
                &shape,
                DrawMode::Outlined {
                    outline_mode: StrokeMode::new(Color::WHITE, POLY_LINE_WIDTH * SCALE),
                    fill_mode: FillMode::color(Color::WHITE),
                },
                Transform {
                    scale: Vec3::splat(SCALE),
                    rotation: Quat::from_rotation_z(180.0_f32.to_radians()),
                    ..Default::default()
                },
            )),
        )
        .insert(Ship {
            state: ShipState::Spawning(Timer::new(Duration::from_secs(2), false)),
        })
        .insert(Bounding::from(0.5))
        .insert(BoundaryWrap)
        .insert(Velocity::default())
        .insert(AngularVelocity::default())
        .insert(Damping::from(PLAYER_DAMPING))
        .insert(SteeringControl::from(Angle::degrees(180.0)))
        .insert(Drive::new(1.5))
        .insert(Visibility::default())
        .insert(Cannon::from(400.0));
    // .insert(Flick {
    //     duration: Timer::new(Duration::from_millis(2250), false),
    //     switch_timer: Timer::new(Duration::from_millis(150), true),
    // });
}

// fn player_state_system(
//     mut commands: Commands,
//     time: Res<Time>,
//     mut query: Query<(Entity, &mut Ship, &mut Transform)>,
// ) {
//     for (entity, mut ship, mut transform) in query.iter_mut() {
//         match ship.state {
//             ShipState::Spawning(ref mut timer) => {
//                 if timer.finished() {
//                     // ship.state = ShipState::Alive;
//                     commands
//                         .entity(entity)
//                         .insert(SteeringControl::from(Angle::degrees(180.0)))
//                         .insert(Drive::new(1.5))
//                         .insert(Cannon::from(400.0))
//                         .log_components();
//                 }
//                 timer.tick(time.delta());
//             }
//             ShipState::Dead(timer) => {}
//             ShipState::Alive => {}
//         }
//     }
// }

pub struct AsteroidSpawnEvent {
    pub pos: Vec2,
    pub radius: f32,
    pub amount: i32,
}

fn asteroid_spawn_system(
    window: Res<WindowDescriptor>,
    asteroid_sizes: Res<AsteroidSizes>,
    mut rng: Local<Random>,
    mut ev_asteroid_spawn: EventWriter<AsteroidSpawnEvent>,
) {
    if !rng.gen_bool(1.0 / 3.0) {
        return;
    }

    let h = window.height / 2.0 / SCALE;
    let w = window.width / 2.0 / SCALE;

    let side = rng.gen_range(0..=3);
    let pos = match side {
        0 => vec2(-w, rng.gen_range(-h..h)),
        1 => vec2(w, rng.gen_range(-h..h)),
        2 => vec2(rng.gen_range(-w..w), -h),
        _ => vec2(rng.gen_range(-w..w), h),
    };

    let size = rng.gen_range(0..=10);
    let radius = match size {
        0..=4 => rng.gen_range(asteroid_sizes.big.clone()),
        5..=8 => rng.gen_range(asteroid_sizes.medium.clone()),
        9..=10 => rng.gen_range(asteroid_sizes.small.clone()),
        _ => rng.gen_range(asteroid_sizes.big.clone()),
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
        for _i in 0..*amount {
            let a = rng.gen_range(7..12);

            let mut points = Vec::new();
            let angle_inc = 360.0 / a as f32;
            let mut bounding = 0.0;
            for i in 1..=a {
                let r = rng.gen_range((radius * 0.5)..*radius);
                if r > bounding {
                    bounding = r;
                }
                let rot = (angle_inc * i as f32).to_radians();
                points.push(vec2(pos.x + r * rot.sin(), pos.y - r * rot.cos()));
            }

            let shape = shapes::Polygon {
                points,
                closed: true,
            };

            let start = vec3(pos.x, pos.y, 1.0);
            let dest = vec3(1.0, 1.0, 1.0);
            let angle = start.angle_between(dest);
            let direction = Quat::from_rotation_z(angle) * -Vec3::Y; //TODO: find out why this works
            let force = rng.gen_range(10.0..30.00);
            let vel = vec2(force * direction.x, force * direction.y);

            let _asteroid = commands
                .spawn()
                .insert_bundle(
                    (GeometryBuilder::build_as(
                        &shape,
                        DrawMode::Outlined {
                            outline_mode: StrokeMode::new(
                                Color::WHITE,
                                POLY_LINE_WIDTH * 1.25 * SCALE,
                            ),
                            fill_mode: FillMode::color(Color::NONE),
                        },
                        Transform {
                            scale: Vec3::splat(SCALE),
                            translation: start,
                            ..Default::default()
                        },
                    )),
                )
                .insert(Bounding::from(bounding))
                .insert(BoundaryRemoval(false))
                .insert(Velocity::from(vel))
                .insert(AngularVelocity::from(0.05));
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
    mut query: Query<(Entity, &Transform, &Bounding, &mut BoundaryRemoval)>,
) {
    let w = window.width / 2.0;
    let h = window.height / 2.0;
    for (entity, transform, bounding, mut removal) in query.iter_mut() {
        let Vec3 { x, y, z: _ } = transform.translation;
        let r = bounding.0;
        if !removal.0 {
            if x + r > -w && x + r < w && y + r < h && y + r > -h {
                removal.0 = true;
            }
        } else if x - w > r || x + r < -w || y - h > r || y + r < -h {
            commands.entity(entity).despawn();
        }
    }
}

fn boundary_wrapping_system(
    window: Res<WindowDescriptor>,
    mut query: Query<(&mut Transform, &Bounding, With<BoundaryWrap>)>,
) {
    for (mut transform, bound, _) in query.iter_mut() {
        let w = window.width;
        let h = window.height;
        let r = bound.0;
        let Vec3 { x, y, z: _ } = transform.translation;

        if (x - r) > (w / 2.0) {
            transform.translation.x = -w / 2.0 - r;
        } else if (x - r) < (-w / 2.0) {
            transform.translation.x = w / 2.0 + r;
        }

        if (y + r) > (h / 2.0) {
            transform.translation.y = -h / 2.0 - r;
        } else if (y - r) < (-h / 2.0) {
            transform.translation.y = h / 2.0 + r;
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
                            outline_mode: StrokeMode::new(Color::WHITE, POLY_LINE_WIDTH * SCALE),
                            fill_mode: FillMode::color(Color::WHITE),
                        },
                        Transform {
                            scale: Vec3::splat(SCALE),
                            translation: transform.translation
                                + vec3(direction.x * bounding.0, direction.y * bounding.0, 0.0),
                            ..Default::default()
                        },
                    )),
                )
                .insert(Bounding::from(CANNON_BULLET_RADIUS))
                .insert(BoundaryRemoval(true))
                .insert(Velocity::from(vec2(
                    cannon.0 * direction.x,
                    cannon.0 * direction.y,
                )));
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

        //what the fuck is this quat shit
        // changed from Vec3::X to -Vec::Y and now this shit works wtf?
        let direction = transform.rotation * -Vec3::Y;
        velocity.x += direction.x * drive.force;
        velocity.y += direction.y * drive.force;
    }
}

fn damping_system(mut query: Query<(&mut Velocity, &Damping)>) {
    for (mut velocity, damping) in query.iter_mut() {
        velocity.0 *= damping.0;
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

fn movement_system(
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

pub fn scaled_ship_points() -> Vec<Vec2> {
    let rot = 0.0_f32.to_radians();
    let sh = 1.0 * SCALE; // ship height
    let sw = 1.0 * SCALE; // ship width

    let v1 = vec2(rot.sin() * sh / 2., -rot.cos() * sh / 2.);
    let v2 = vec2(
        -rot.cos() * sw / 2. - rot.sin() * sh / 2.,
        -rot.sin() * sw / 2. + rot.cos() * sh / 2.,
    );
    let v3 = vec2(
        rot.cos() * sw / 2. - rot.sin() * sh / 2.,
        rot.sin() * sw / 2. + rot.cos() * sh / 2.,
    );
    let v4 = vec2(
        -rot.cos() * sw / 1.5 - rot.sin() * sh / 1.5,
        -rot.sin() * sw / 1.5 + rot.cos() * sh / 1.5,
    );
    let v5 = vec2(
        rot.cos() * sw / 1.5 - rot.sin() * sh / 1.5,
        rot.sin() * sw / 1.5 + rot.cos() * sh / 1.5,
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
//2-4, 3-5
#[derive(Debug, Component, Default, From)]
struct Flick {
    switch_timer: Timer,
    duration: Timer,
}

#[derive(Debug, Component, Default)]
struct Ship {
    state: ShipState,
}

#[derive(Debug, Component, Default, Deref, DerefMut, From)]
struct Cannon(f32);

#[derive(Debug, Component)]
struct Drive {
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

#[derive(Debug, Component, Default, Deref, DerefMut, From)]
struct Bullet(Timer);

#[derive(Debug, Component)]
struct AsteroidSizes {
    big: Range<f32>,
    medium: Range<f32>,
    small: Range<f32>,
}
#[derive(Debug, Component, Default, Deref, DerefMut, From)]
struct Bounding(f32);
#[derive(Debug, Component)]
struct BoundaryWrap;
#[derive(Debug, Component, Deref, DerefMut)]
struct BoundaryRemoval(bool);

#[derive(Debug, Component, Default, Deref, DerefMut, From)]
struct Velocity(Vec2);

#[derive(Debug, Component, Default, Deref, DerefMut, From)]
struct AngularVelocity(f32);

#[derive(Debug, Component)]
struct SpeedLimit(f32);

#[derive(Debug, Component, Default, Deref, DerefMut, From)]
struct Damping(f32);

#[derive(Debug, Component, Default, Deref, DerefMut, From)]
struct SteeringControl(Angle);

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

#[derive(Debug)]
enum ShipState {
    Alive,
    Dead(Timer),
    Spawning(Timer),
}

impl Default for ShipState {
    fn default() -> Self {
        ShipState::Alive
    }
}
