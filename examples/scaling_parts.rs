#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_novelgame_dialog::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            dialog_box::DialogBoxPlugin::default(),
        ))
        .add_systems(Startup, button_box_setup)
        .run();
}

//--

fn button_box_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let button_image_handle =
        asset_server.load("textures\\ui\\choice_buttons\\button_default.png");
    let choicing_frame_image_handle =
        asset_server.load("textures\\ui\\choice_buttons\\choicing_frame.png");
    let dialog_box_image_handle =
        asset_server.load("textures\\ui\\dialog_box_01.png");
    let button_slice = ImageScaleMode::Sliced(TextureSlicer {
        border: BorderRect::square(30.),
        ..default()
    });
    let choicing_frame_slice = ImageScaleMode::Sliced(TextureSlicer {
        border: BorderRect::rectangle(56., 102.),
        ..default()
    });
    let dialog_box_slice = ImageScaleMode::Sliced(TextureSlicer {
        border: BorderRect::rectangle(44., 52.),
        ..default()
    });
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(400., 100.)),
                ..default()
            },
            transform: Transform::from_xyz(0., 130., 0.8),
            texture: button_image_handle.clone(),
            ..default()
        },
        button_slice.clone(),
    ));
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(400., 100.)),
                ..default()
            },
            transform: Transform::from_xyz(0., -10., 0.8),
            texture: button_image_handle.clone(),
            ..default()
        },
        button_slice.clone(),
    ));
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(400., 100.)),
                ..default()
            },
            transform: Transform::from_xyz(0., -140., 0.8),
            texture: button_image_handle.clone(),
            ..default()
        },
        button_slice,
    ));
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(480., 200.)),
                ..default()
            },
            transform: Transform::from_xyz(-2., 135., 0.5),
            texture: choicing_frame_image_handle,
            ..default()
        },
        choicing_frame_slice,
    ));
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgba(1., 1., 1., 0.5),
                custom_size: Some(Vec2::new(600., 500.)),
                ..default()
            },
            transform: Transform::from_xyz(0., 0., 0.3),
            texture: dialog_box_image_handle,
            ..default()
        },
        dialog_box_slice,
    ));
}
