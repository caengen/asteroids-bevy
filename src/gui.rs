use bevy::{
    math::{vec2, vec3},
    prelude::*,
    render::render_resource::Texture,
    window::WindowDescriptor,
};

use crate::{
    random::Random, FRAME_X_OFFSET, GAME_FRAME_HEIGHT, GAME_FRAME_WIDTH, LIGHT, SCREEN_HEIGHT,
    SCREEN_WIDTH,
};

// #[derive(Resource)]
// struct UIGameFrame(Handle<Texture>);
const STAT_FRAME_WIDTH: f32 = 244.0;
const STAT_FRAME_HEIGHT: f32 = 512.0;
const GAME_FRAME_SIZE: Vec2 = Vec2::from_array([GAME_FRAME_WIDTH, GAME_FRAME_HEIGHT]);
const STAT_FRAME_SIZE: Vec2 = Vec2::from_array([STAT_FRAME_WIDTH, STAT_FRAME_HEIGHT]);

pub fn draw_text(
    commands: &mut Commands,
    atlas_handle: &Handle<TextureAtlas>,
    text: &str,
    x: f32,
    y: f32,
    scale: f32,
    letter_spacing: f32,
) {
    for (i, c) in text.chars().enumerate() {
        match c {
            'A'..='P' => {
                commands.spawn_bundle(SpriteSheetBundle {
                    texture_atlas: atlas_handle.clone(),
                    sprite: TextureAtlasSprite {
                        color: LIGHT,
                        index: (c as u8 - 49) as usize,
                        custom_size: Some(vec2(8.0 * scale, 8.0 * scale)),
                        ..default()
                    },
                    transform: Transform {
                        translation: vec3(x + (i as f32 * letter_spacing) * scale, y, 2.0),
                        ..default()
                    },
                    ..default()
                });
            }
            'Q'..='Z' => {
                commands.spawn_bundle(SpriteSheetBundle {
                    texture_atlas: atlas_handle.clone(),
                    sprite: TextureAtlasSprite {
                        color: LIGHT,
                        index: (c as u8 - 43) as usize,
                        custom_size: Some(vec2(8.0 * scale, 8.0 * scale)),
                        ..default()
                    },
                    transform: Transform {
                        translation: vec3(x + (i as f32 * letter_spacing) * scale, y, 2.0),
                        ..default()
                    },
                    ..default()
                });
            }
            '0'..='9' => {
                commands.spawn_bundle(SpriteSheetBundle {
                    texture_atlas: atlas_handle.clone(),
                    sprite: TextureAtlasSprite {
                        color: LIGHT,
                        index: (c as u8 - 48) as usize,
                        custom_size: Some(vec2(8.0 * scale, 8.0 * scale)),
                        ..default()
                    },
                    transform: Transform {
                        translation: vec3(x + (i as f32 * letter_spacing) * scale, y, 2.0),
                        ..default()
                    },
                    ..default()
                });
            }
            _ => {}
        }
    }
}

pub fn setup_game_ui(
    mut commands: Commands,
    window: Res<WindowDescriptor>,
    asset_server: Res<AssetServer>,
    mut rng: Local<Random>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let radar_handle = asset_server.load("radar.png");
    let atlas_handle = asset_server.load("atlas.png");
    let atlas = TextureAtlas::from_grid(atlas_handle, vec2(8.0, 8.0), 16, 10);
    let texture_atlas_handle = texture_atlases.add(atlas);

    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("game_frame.png"),
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
        texture: asset_server.load("stat_frame.png"),
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
    commands.spawn_bundle(SpriteBundle {
        texture: radar_handle,
        sprite: Sprite {
            color: LIGHT,
            custom_size: Some(vec2(51.0 * 4.0, 45.0 * 4.0)),
            ..default()
        },
        transform: Transform {
            translation: vec3(
                SCREEN_WIDTH / 2.0 - (51.0 * 4.0) / 2.0 - 24.0,
                (-SCREEN_HEIGHT / 2.0) + (45.0 * 4.0) / 2.0 + 20.0,
                2.0,
            ),
            ..default()
        },
        ..default()
    });
    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("life_frame.png"),
        sprite: Sprite {
            color: LIGHT,
            custom_size: Some(vec2(20.0 * 4.0, 30.0 * 4.0)),
            ..default()
        },
        transform: Transform {
            translation: vec3(
                SCREEN_WIDTH / 2.0 - (21.0 * 4.0) / 2.0 - 24.0,
                2.0 * 4.0,
                2.0,
            ),
            ..default()
        },
        ..default()
    });

    draw_text(
        &mut commands,
        &texture_atlas_handle,
        "HULL",
        SCREEN_WIDTH / 2.0 - (53.0 * 4.0),
        2.0 * 4.0,
        4.0,
        7.0,
    );
    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("pc_frame.png"),
        sprite: Sprite {
            color: LIGHT,
            custom_size: Some(vec2(28.0 * 4.0, 10.0 * 4.0)),
            ..default()
        },
        transform: Transform {
            translation: vec3(SCREEN_WIDTH / 2.0 - (85.0 * 4.0) / 2.0, -8.0 * 4.0, 2.0),
            ..default()
        },
        ..default()
    });
    draw_text(
        &mut commands,
        &texture_atlas_handle,
        "100",
        SCREEN_WIDTH / 2.0 - (51.5 * 4.0),
        -8.0 * 4.0,
        4.0,
        9.0,
    );
    draw_text(
        &mut commands,
        &texture_atlas_handle,
        "SHLD",
        SCREEN_WIDTH / 2.0 - (53.0 * 4.0),
        22.0 * 4.0,
        4.0,
        7.0,
    );
    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("pc_frame.png"),
        sprite: Sprite {
            color: LIGHT,
            custom_size: Some(vec2(28.0 * 4.0, 10.0 * 4.0)),
            ..default()
        },
        transform: Transform {
            translation: vec3(SCREEN_WIDTH / 2.0 - (85.0 * 4.0) / 2.0, 12.0 * 4.0, 2.0),
            ..default()
        },
        ..default()
    });
    draw_text(
        &mut commands,
        &texture_atlas_handle,
        "100",
        SCREEN_WIDTH / 2.0 - (51.5 * 4.0),
        12.0 * 4.0,
        4.0,
        9.0,
    );
    draw_text(
        &mut commands,
        &texture_atlas_handle,
        "STAGE",
        SCREEN_WIDTH / 2.0 - STAT_FRAME_SIZE.x + 8.0 * 4.0,
        SCREEN_HEIGHT / 2.0 - 10.0 * 4.0,
        4.0,
        8.0,
    );
    draw_text(
        &mut commands,
        &texture_atlas_handle,
        "POINTS",
        SCREEN_WIDTH / 2.0 - STAT_FRAME_SIZE.x + 8.0 * 4.0,
        SCREEN_HEIGHT / 2.0 - 20.0 * 4.0,
        4.0,
        8.0,
    );
    draw_text(
        &mut commands,
        &texture_atlas_handle,
        "000000",
        SCREEN_WIDTH / 2.0 - STAT_FRAME_SIZE.x + 10.0 * 4.0,
        SCREEN_HEIGHT / 2.0 - 30.5 * 4.0,
        4.0,
        8.0,
    );
    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("points_frame.png"),
        sprite: Sprite {
            color: LIGHT,
            custom_size: Some(vec2(51.0 * 4.0, 11.0 * 4.0)),
            ..default()
        },
        transform: Transform {
            translation: vec3(
                SCREEN_WIDTH / 2.0 - (62.5 * 4.0) / 2.0,
                SCREEN_HEIGHT / 2.0 - 30.0 * 4.0,
                2.0,
            ),
            ..default()
        },
        ..default()
    });

    // Lives
    draw_text(
        &mut commands,
        &texture_atlas_handle,
        "3",
        SCREEN_WIDTH / 2.0 - (16.5 * 4.0),
        -7.5 * 4.0,
        4.0,
        9.0,
    );
}
