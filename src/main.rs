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

pub const PLAYER_DAMPING: f32 = 0.75;

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
                    outline_mode: StrokeMode::new(Color::WHITE, 0.075 * SCALE),
                    fill_mode: FillMode::color(Color::BLACK),
                },
                Transform {
                    scale: Vec3::splat(SCALE),
                    ..Default::default()
                },
            )),
        )
        .insert(Velocity::default())
        .insert(AngularVelocity::default())
        .insert(Damping::from(PLAYER_DAMPING))
        .insert(SteeringControl::from(Angle::degrees(180.0)));
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
    for (mut transform, angular_velocity, _velocity) in query.iter_mut() {
        if let Some(AngularVelocity(vel)) = angular_velocity {
            transform.rotate(Quat::from_rotation_z(vel * time.delta_seconds()))
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
struct SideThrusters {}

#[derive(Debug, Component, Default)]
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
