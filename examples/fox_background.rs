#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use bevy::prelude::*;
use bevy_novelgame_dialog::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            ui_templates::RPGStyleUIPlugin::default(),
            fox_background::FoxBackgroundPlugin,
            DebugTextAreaPlugin,
        ))
        .add_systems(Update, start_message)
        .run();
}

fn start_message(
    mut open_message_event: EventWriter<OpenRPGStyleDialog>,
    mut is_started: Local<bool>,
) {
    if !*is_started {
        let event = OpenRPGStyleDialog {
            script_path: "scripts/starter.md".to_string(),
        };
        open_message_event.send(event);
        *is_started = true;
    }
}
//----------

mod fox_background {
    use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*, utils::Duration};
    use bevy_novelgame_dialog::bds::*;
    use std::f32::consts::PI;

    pub struct FoxBackgroundPlugin;

    impl Plugin for FoxBackgroundPlugin {
        fn build(&self, app: &mut App) {
            app.insert_resource(AmbientLight {
                color: Color::WHITE,
                brightness: 1.0,
            })
            .add_systems(Startup, setup)
            .add_systems(Update, (setup_scene_once_loaded, signal_animation_control));
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

    fn signal_animation_control(
        mut animation_player: Query<&mut AnimationPlayer>,
        animations: Res<Animations>,
        mut signal_events: EventReader<BdsSignal>,
    ) {
        for BdsSignal { signal: sig } in signal_events.read() {
            if let Ok(mut player) = animation_player.get_single_mut() {
                if *sig == "Fox_run".to_string() {
                    player
                        .play_with_transition(
                            animations.0[0].clone_weak(),
                            Duration::from_millis(250),
                        )
                        .repeat();
                } else if *sig == "Fox_walk".to_string() {
                    player
                        .play_with_transition(
                            animations.0[1].clone_weak(),
                            Duration::from_millis(250),
                        )
                        .repeat();
                } else if *sig == "Fox_stop".to_string() {
                    player
                        .play_with_transition(
                            animations.0[2].clone_weak(),
                            Duration::from_millis(250),
                        )
                        .repeat();
                }
            }
        }
    }
}
