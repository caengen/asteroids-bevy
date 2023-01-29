use bevy::{
    math::{vec2, vec3},
    prelude::*,
    render::render_resource::Texture,
    window::WindowDescriptor,
};

use crate::{
    random::Random, FRAME_X_OFFSET, GAME_FRAME_HEIGHT, GAME_FRAME_WIDTH, LIGHT, SCREEN_WIDTH,
};

// #[derive(Resource)]
// struct UIGameFrame(Handle<Texture>);
const STAT_FRAME_WIDTH: f32 = 244.0;
const STAT_FRAME_HEIGHT: f32 = 512.0;
const GAME_FRAME_SIZE: Vec2 = Vec2::from_array([GAME_FRAME_WIDTH, GAME_FRAME_HEIGHT]);
const STAT_FRAME_SIZE: Vec2 = Vec2::from_array([STAT_FRAME_WIDTH, STAT_FRAME_HEIGHT]);

pub fn setup_game_ui(
    mut commands: Commands,
    window: Res<WindowDescriptor>,
    asset_server: Res<AssetServer>,
    mut rng: Local<Random>,
) {
    let handle = asset_server.load("game_frame.png");
    let handle2 = asset_server.load("stat_frame.png");
    // commands.insert_resource(UIGameFrame(handle));
    commands.spawn_bundle(SpriteBundle {
        texture: handle,
        sprite: Sprite {
            color: LIGHT,
            custom_size: Some(vec2(GAME_FRAME_SIZE.x, GAME_FRAME_SIZE.y)),
            ..default()
        },
        transform: Transform {
            translation: vec3(-FRAME_X_OFFSET, 0.0, 2.0),
            ..default()
        },
        ..default()
    });
    commands.spawn_bundle(SpriteBundle {
        texture: handle2,
        sprite: Sprite {
            color: LIGHT,
            custom_size: Some(vec2(STAT_FRAME_SIZE.x, STAT_FRAME_SIZE.y)),
            ..default()
        },
        transform: Transform {
            translation: vec3(
                (SCREEN_WIDTH / 2.0) - STAT_FRAME_SIZE.x / 2.0 - 4.0,
                0.0,
                2.0,
            ),
            ..default()
        },
        ..default()
    });
}
