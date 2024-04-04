#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy::text::JustifyText;
use bevy_novelgame_dialog::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            dialog_box::DialogBoxPlugin::default(),
            fox_background::FoxBackgroundPlugin,
            DebugTextAreaPlugin,
        ))
        .add_systems(Startup, waiting_sprite_setup)
        .add_systems(Startup, setup_messageframe)
        .add_systems(Startup, setup_choice_images)
        .add_systems(Update, start_message)
        .add_systems(Update, animate_sprite)
        .run();
}

fn start_message(
    mut ow_event: EventWriter<OpenDialogEvent>,
    background: Query<Entity, With<DialogBoxBackground>>,
    choice_frame: Query<Entity, With<ChoiceFrame>>,
    choice_buttons: Query<(Entity, &ChoiceButton)>,
    mut is_started: Local<bool>,
) {
    if !*is_started {
        let mut buttons_vec = choice_buttons.iter().collect::<Vec<_>>();
        let font_vec = [
            "UnifrakturMaguntia/UnifrakturMaguntia-Regular.ttf",
            "赤薔薇/akabara-cinderella.ttf",
            "网风雅宋/网风雅宋.ttf",
            "noto/NotoEmoji-VariableFont_wght.ttf",
        ]
        .iter()
        .map(|s| String::from("fonts/".to_owned() + s))
        .collect::<Vec<_>>();
        let frame_tac = TextAreaConfig {
            font_paths: font_vec.clone(),
            feeding: FeedingStyle::Scroll { size: 0, sec: 0.5 },
            font_color: Color::DARK_GRAY,
            area_origin: Vec2::new(-540.0, 70.0),
            area_size: Vec2::new(1060.0, 140.0),
            // main_alignment: JustifyText::Center,
            // writing:WritingStyle::Wipe{ sec: 0.7 },
            // writing:WritingStyle::Put,
            // typing_timing: TypingTiming::ByLine { sec: 1.5 },
            // typing_timing: TypingTiming::ByPage,
            ..default()
        };
        let tac_base = TextAreaConfig {
            font_paths: font_vec.clone(),
            area_origin: Vec2::new(-220.0, 200.0),
            area_size: Vec2::new(400.0, 100.0),
            font_color: Color::NAVY,
            writing: WritingStyle::Put,
            typing_timing: TypingTiming::ByPage,
            main_alignment: JustifyText::Center,
            ..default()
        };
        let tac_list = (0..4)
            .map(|i| TextAreaConfig {
                area_origin: Vec2::new(-220.0, -30.0 - 140.0 * (i as f32)),
                area_name: format!("Button Area {i:02}"),
                ..tac_base.clone()
            })
            .collect::<Vec<_>>();
        buttons_vec.sort_by_key(|x| x.1 .0);
        ow_event.send(OpenDialogEvent {
            dialog_box_name:  "Main Box".to_string(),
            script_path: "scripts/reload_test.md#テストヘッダー2".to_string(),
            template_path: vec!["scripts/test.csv".to_string(), "scripts/basic.bdt".to_string()],
            text_area_configs: vec![frame_tac],
            dialog_box_entity: Some(background.single()),
            position: Vec2::new(0., -200.),
            wait_breaker: WaitBrakerStyle::Input {
                is_icon_moving_to_last: true,
            },
            template_open_choice: ChoiceBoxConfig {
                background_entity: choice_frame.get_single().ok(),
                button_entities: buttons_vec.iter().map(|x| x.0).collect::<Vec<_>>(),
                button_text_areas: tac_list,
                background_scaling_per_button: Vec2::new(0.0, 140.0),
                background_scaling_anchor: Anchor::TopCenter,
                ..default()
            },
            ..default()
        });
        *is_started = true;
    }
}

//----------
use bevy::sprite::Anchor;

#[derive(Component)]
struct WaitingSprite;

#[derive(Component)]
struct DialogBoxBackground;

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
    step: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut TextureAtlas)>,
) {
    for (indices, mut timer, mut atlas) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            atlas.index = if atlas.index >= indices.last {
                indices.first
            } else {
                atlas.index + indices.step
            };
        }
    }
}

fn setup_messageframe(mut commands: Commands, asset_server: Res<AssetServer>) {
    let dialog_box_image_handle = asset_server.load("textures/ui/dialog_box_02.png");
    let dialog_box_slice = ImageScaleMode::Sliced(TextureSlicer {
        border: BorderRect::rectangle(55.0, 71.0),
        ..default()
    });
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                // color: Color::rgba(1.0, 1.0, 1.0, 0.8),
                custom_size: Some(Vec2::new(1200.0, 300.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -200.0, 0.0),
            texture: dialog_box_image_handle,
            ..default()
        },
        dialog_box_slice,
        DialogBoxBackground,
    ));
}

fn waiting_sprite_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture_handle = asset_server.load("textures/ui/cursor.png");
    let texture_atlas = TextureAtlasLayout::from_grid(Vec2::new(44.0, 56.0), 1, 2, None, None);
    let animation_indices = AnimationIndices {
        first: 0,
        last: 1,
        step: 1,
    };
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    let mut sprite = Sprite::default();
    sprite.anchor = Anchor::BottomLeft;
    commands.spawn((
        SpriteSheetBundle {
            atlas: TextureAtlas {
                layout: texture_atlas_handle,
                index: animation_indices.first,
            },
            sprite: sprite,
            transform: Transform::from_scale(Vec3::splat(0.5)),
            texture: texture_handle,
            ..default()
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.5, TimerMode::Repeating)),
        WaitingIcon {
            target_window_name: "Main Box".to_string(),
        },
        WaitingSprite,
    ));
}

//----------

#[derive(Component)]
struct ChoiceFrame;

#[derive(Component)]
struct ChoiceButton(usize);

#[derive(Component)]
struct ChoiceCursor;

fn setup_choice_images(mut commands: Commands, asset_server: Res<AssetServer>) {
    let button_image_handle = asset_server.load("textures/ui/choice_buttons/button_default.png");
    let choicing_frame_image_handle =
        asset_server.load("textures/ui/choice_buttons/choicing_frame.png");
    let dialog_box_image_handle = asset_server.load("textures/ui/dialog_box_01.png");
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
    for i in 0..4 {
        let button_sprite_bundle = SpriteBundle {
            sprite: Sprite {
                // color: Color::rgba(1., 1., 1., 0.3),
                custom_size: Some(Vec2::new(400., 100.)),
                ..default()
            },
            texture: button_image_handle.clone(),
            transform: Transform::from_xyz(0.0, -70.0 - 140.0 * (i as f32), 0.6),
            ..default()
        };
        commands.spawn((button_sprite_bundle, button_slice.clone(), ChoiceButton(i)));
    }
    let cursor_sprite_bundle = SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(480., 200.)),
            ..default()
        },
        texture: choicing_frame_image_handle,
        transform: Transform::from_xyz(-2., 200., 0.3),
        ..default()
    };
    commands.spawn((cursor_sprite_bundle, choicing_frame_slice, ChoiceCursor));
    let frame_sprite_bundle = SpriteBundle {
        sprite: Sprite {
            // color: Color::rgba(1., 1., 1., 0.3),
            custom_size: Some(Vec2::new(600., 100.)),
            ..default()
        },
        texture: dialog_box_image_handle,
        transform: Transform::from_xyz(0., 100., 1.1),
        ..default()
    };
    commands.spawn((frame_sprite_bundle, dialog_box_slice, ChoiceFrame));
}

//----------

mod fox_background {
    use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*, utils::Duration};
    use std::f32::consts::PI;

    pub struct FoxBackgroundPlugin;

    impl Plugin for FoxBackgroundPlugin {
        fn build(&self, app: &mut App) {
            app.insert_resource(AmbientLight {
                color: Color::WHITE,
                brightness: 1.0,
            })
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (setup_scene_once_loaded, keyboard_animation_control),
            );
        }
    }

    #[derive(Resource)]
    struct Animations(Vec<Handle<AnimationClip>>);

    fn setup(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        // Insert a resource with the current scene information
        commands.insert_resource(Animations(vec![
            asset_server.load("models/animated/Fox.glb#Animation2"),
            asset_server.load("models/animated/Fox.glb#Animation1"),
            asset_server.load("models/animated/Fox.glb#Animation0"),
        ]));

        // Camera
        commands.spawn(Camera3dBundle {
            transform: Transform::from_xyz(100.0, 100.0, 150.0)
                .looking_at(Vec3::new(0.0, 20.0, 0.0), Vec3::Y),
            ..default()
        });

        // Plane
        commands.spawn(PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(500000.0, 500000.0)),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3)),
            ..default()
        });

        // Light
        commands.spawn(DirectionalLightBundle {
            transform: Transform::from_rotation(Quat::from_euler(
                EulerRot::ZYX,
                0.0,
                1.0,
                -PI / 4.,
            )),
            directional_light: DirectionalLight {
                shadows_enabled: true,
                ..default()
            },
            cascade_shadow_config: CascadeShadowConfigBuilder {
                first_cascade_far_bound: 200.0,
                maximum_distance: 400.0,
                ..default()
            }
            .into(),
            ..default()
        });

        // Fox
        commands.spawn(SceneBundle {
            scene: asset_server.load("models/animated/Fox.glb#Scene0"),
            ..default()
        });
    }

    // Once the scene is loaded, start the animation
    fn setup_scene_once_loaded(
        animations: Res<Animations>,
        mut player: Query<&mut AnimationPlayer>,
        mut done: Local<bool>,
    ) {
        if !*done {
            if let Ok(mut player) = player.get_single_mut() {
                player.play(animations.0[0].clone_weak()).repeat();
                *done = true;
            }
        }
    }

    fn keyboard_animation_control(
        keyboard_input: Res<ButtonInput<KeyCode>>,
        mut animation_player: Query<&mut AnimationPlayer>,
        animations: Res<Animations>,
        mut current_animation: Local<usize>,
    ) {
        if let Ok(mut player) = animation_player.get_single_mut() {
            if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
                let anim_len = animations.0.len();
                *current_animation = (*current_animation + anim_len - 1) % anim_len;
                player
                    .play_with_transition(
                        animations.0[*current_animation].clone_weak(),
                        Duration::from_millis(250),
                    )
                    .repeat();
            }

            if keyboard_input.just_pressed(KeyCode::ArrowRight) {
                *current_animation = (*current_animation + 1) % animations.0.len();
                player
                    .play_with_transition(
                        animations.0[*current_animation].clone_weak(),
                        Duration::from_millis(250),
                    )
                    .repeat();
            }
        }
    }
}
