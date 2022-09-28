use bevy::{
    math::{const_vec2, vec2, vec3},
    prelude::*,
    window::PresentMode,
};
use bevy_prototype_lyon::{
    entity::ShapeBundle,
    prelude::{
        tess::{geom::Rotation, math::Angle},
        *,
    },
    shapes::Polygon,
};
use derive_more::From;
use std::{default::Default, f32::consts::PI, ops::Range, time::Duration};

const SCREEN_HEIGHT: f32 = 640.0;
const SCREEN_WIDTH: f32 = 960.0;
pub const SCREEN: Vec2 = Vec2::from_array([SCREEN_WIDTH, SCREEN_HEIGHT]);
// pub const TIME_STEP: f32 = 1.0 / 60.0;
pub const GAME_WIDTH: f32 = 240.0;
pub const SCALE: f32 = SCREEN_WIDTH / GAME_WIDTH;
// pub const PIXELS_PER_METER: f32 = 30.0 / SCALE;

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
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_startup_system(setup_system)
        .add_system(steering_control_system)
        .add_system(movement_system)
        .add_system(drive_control_system)
        .add_system(drive_system)
        .add_system(damping_system)
        .add_system(boundary_wrapping_system)
        .run();
}

fn setup_system(mut commands: Commands) {
    let shape = shapes::Polygon {
        points: scaled_ship_points(),
        closed: false,
    };

    commands.spawn_bundle(Camera2dBundle::default());

    let mut player = commands.spawn();
    // commands.spawn().insert(Ship::spawn(Duration::from_secs(0)));
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
        .insert(Bounds::from(vec2(1.0, 1.0)))
        .insert(Velocity::default())
        .insert(AngularVelocity::default())
        .insert(Damping::from(PLAYER_DAMPING))
        .insert(SteeringControl::from(Angle::degrees(180.0)))
        .insert(Drive::new(1.5));
}

fn boundary_wrapping_system(mut query: Query<(&mut Transform, &Bounds)>) {
    for (mut transform, bounds) in query.iter_mut() {
        if (transform.translation.x + bounds.x / 2.0) > (SCREEN_WIDTH / 2.0) {
            transform.translation.x = -SCREEN_WIDTH / 2.0 - bounds.x / 2.0;
        } else if (transform.translation.x - bounds.x / 2.0) < (-SCREEN_WIDTH / 2.0) {
            transform.translation.x = SCREEN_WIDTH / 2.0 + bounds.x / 2.0;
        }

        if (transform.translation.y + bounds.y / 2.0) > (SCREEN_HEIGHT / 2.0) {
            transform.translation.y = -SCREEN_HEIGHT / 2.0 - bounds.y / 2.0;
        } else if (transform.translation.y - bounds.y) < (-SCREEN_HEIGHT / 2.0) {
            transform.translation.y = SCREEN_HEIGHT / 2.0 + bounds.y / 2.0;
        }
    }
}

fn drive_control_system(mut query: Query<(&mut Drive)>, keyboard: Res<Input<KeyCode>>) {
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
//2-4, 3-5
#[derive(Debug, Component)]
struct Ship {
    state: ShipState,
}

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
struct Bounds(Vec2);

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
