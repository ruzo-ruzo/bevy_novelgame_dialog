#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy::text::JustifyText;
use bevy_dialog_box::*;

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
        .add_systems(Update, start_message)
        .add_systems(Update, animate_sprite)
        .run();
}

fn start_message(
    mut ow_event: EventWriter<OpenWindowEvent>,
    waiting_sprite: Query<Entity, With<WaitingSprite>>,
    background: Query<Entity, With<DialogBoxBackground>>,
    mut is_started: Local<bool>,
) {
    if !*is_started {
        ow_event.send(OpenWindowEvent {
            font_paths: [
                // "NotoColorEmoji-Regular",
                "yurumoji.ttf",
                "yinghuayunduoxiaohuzi.ttf",
                "NotoSansJP-Black.ttf",
            ]
            .iter()
            .map(|s| String::from("fonts/".to_owned() + s))
            .collect(),
            // background_path:
                // "2d_picture/ui/messageframe/material/messageframe_non_line/message_001.png"
                    // .to_string(),
            position: Vec2::new(0., -200.),
            feeding: FeedingStyle::Scroll { size: 0, sec: 0.5 },
			dialog_box_entity: Some(background.single()),
			font_color: Color::DARK_GRAY,
            // script_path: "scripts/test.bds".to_string(),
            script_path: "scripts/reload_test.bds#テストヘッダー2".to_string(),
            template_path: "scripts/test.bdt".to_string(),
            main_box_origin: Vec2::new(-540.0, 70.0),
            main_box_size: Vec2::new(1060.0, 140.0),
            // main_alignment: JustifyText::Center,
            // writing:WritingStyle::Wipe{ sec: 0.7 },
            // writing:WritingStyle::Put,
            // typing_timing: TypingTiming::ByLine { sec: 1.5 },
            // typing_timing: TypingTiming::ByPage,
            wait_breaker: WaitBrakerStyle::Input {
                icon_entity: Some(waiting_sprite.single()),
                is_icon_moving_to_last: true,
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

fn setup_messageframe(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
){
    let dialog_box_image_handle =
		asset_server.load("textures\\ui\\dialog_box_02.png");

    let dialog_box_slice = ImageScaleMode::Sliced(TextureSlicer {
        border: BorderRect::rectangle(55., 71.),
        ..default()
    });
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
				color: Color::rgba(1., 1., 1., 0.8),
                custom_size: Some(Vec2::new(1200., 300.)),
                ..default()
            },
            transform: Transform::from_xyz(0., -200., 0.),
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
    let texture_handle =
        asset_server.load("2d_picture/ui/kenney_input-prompts-pixel-16/Tilemap/tilemap.png");
    let texture_atlas = TextureAtlasLayout::from_grid(
        Vec2::new(16.0, 16.0),
        34,
        24,
        Some(Vec2::new(1.0, 1.0)),
        None,
    );
    let animation_indices = AnimationIndices {
        first: 705,
        last: 739,
        step: 34,
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
            transform: Transform::from_scale(Vec3::splat(1.5)),
            texture: texture_handle,
            ..default()
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.5, TimerMode::Repeating)),
        WaitingSprite,
    ));
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
