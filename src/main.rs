use bevy::{math::vec2, prelude::*, window::PresentMode};

use bevy_prototype_lyon::{
    entity::ShapeBundle,
    prelude::{
        tess::{geom::Rotation, math::Angle},
        *,
    },
    shapes::Polygon,
};
use std::{default::Default, f32::consts::PI, ops::Range, time::Duration};

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "asteroids-bevy".to_string(),
            present_mode: PresentMode::Fifo,
            ..default()
        })
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_startup_system(setup_system)
        .run();
}

fn setup_system(mut commands: Commands) {
    let shape = shapes::Polygon {
        points: ship_points(),
        closed: false,
    };

    // commands.spawn().insert(Ship::spawn(Duration::from_secs(0)));
    commands.spawn_bundle(Camera2dBundle::default());
    commands.spawn_bundle(GeometryBuilder::build_as(
        &shape,
        DrawMode::Outlined {
            outline_mode: StrokeMode::new(Color::WHITE, 5.0),
            fill_mode: FillMode::color(Color::BLACK),
        },
        Transform::default(),
    ));
}

pub fn ship_points() -> Vec<Vec2> {
    let rot = 0.0_f32.to_radians();
    let sh = 200.0; // ship height
    let sw = 200.0; // ship width

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
