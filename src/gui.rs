use bevy::{
    math::{vec2, vec3},
    prelude::*,
    render::render_resource::Texture,
    window::WindowDescriptor,
};

use crate::{random::Random, FRAME_HEIGHT, FRAME_WIDTH, FRAME_X_OFFSET, LIGHT, SCREEN_WIDTH};

// #[derive(Resource)]
// struct UIGameFrame(Handle<Texture>);

const GAME_FRAME_SIZE: Vec2 = Vec2::from_array([FRAME_WIDTH, FRAME_HEIGHT]);

pub fn setup_game_ui(
    mut commands: Commands,
    window: Res<WindowDescriptor>,
    asset_server: Res<AssetServer>,
    mut rng: Local<Random>,
) {
    let handle = asset_server.load("game_frame.png");
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
}

pub fn render_game_ui() {}
