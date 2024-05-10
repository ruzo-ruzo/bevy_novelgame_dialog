#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use bevy::prelude::*;
use bevy_novelgame_dialog::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            ui_templates::RPGStyleUIPlugin::default(),
            models_controller::ModelsControllerPlugin,
            // DebugTextAreaPlugin,
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

mod models_controller {
    use bevy::{gltf::Gltf, pbr::CascadeShadowConfigBuilder, prelude::*, utils::Duration};
    use bevy_novelgame_dialog::public::bds::*;
    use std::collections::HashMap;
    use std::f32::consts::TAU;

    pub struct ModelsControllerPlugin;
    impl Plugin for ModelsControllerPlugin {
        fn build(&self, app: &mut App) {
            app.add_plugins((girl::GirlPlugin, rabit::RabitPlugin, room::RoomPlugin));
        }
    }

    #[derive(Component)]
    struct Animations {
        list: HashMap<String, Handle<AnimationClip>>,
    }

    mod room {
        use super::*;

        pub struct RoomPlugin;
        impl Plugin for RoomPlugin {
            fn build(&self, app: &mut App) {
                app.insert_resource(AmbientLight {
                    color: Color::WHITE,
                    brightness: 0.0,
                })
                .add_systems(Startup, setup)
                .add_systems(Update, load_scenes);
            }
        }

        #[derive(Resource)]
        struct Room(Handle<Gltf>);

        fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
            let gltf = asset_server.load("models/room.glb");
            commands.insert_resource(Room(gltf));
            commands.spawn(Camera3dBundle {
                transform: Transform::from_xyz(0.0, 0.6, 2.0)
                    .looking_at(Vec3::new(0.0, 0.6, 0.0), Vec3::Y),
                ..default()
            });
            commands.spawn(DirectionalLightBundle {
                transform: Transform::from_rotation(Quat::from_euler(
                    EulerRot::ZYX,
                    0.0,
                    1.0,
                    -TAU / 8.,
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
        }

        fn load_scenes(
            mut commands: Commands,
            room: Res<Room>,
            assets_gltf: Res<Assets<Gltf>>,
            mut done: Local<bool>,
        ) {
            if !*done {
                if let Some(gltf) = assets_gltf.get(&room.0) {
                    commands.spawn(SceneBundle {
                        scene: gltf.named_scenes["Scene"].clone(),
                        ..default()
                    });
                    *done = true;
                }
            }
        }
    }

    mod rabit {
        use super::*;

        pub struct RabitPlugin;
        impl Plugin for RabitPlugin {
            fn build(&self, app: &mut App) {
                app.add_systems(Startup, setup).add_systems(
                    Update,
                    (
                        load_scenes,
                        start_stay,
                        signal_animation_control,
                        resume_stay,
                    ),
                );
            }
        }

        #[derive(Resource)]
        pub struct RabitGltf(Handle<Gltf>);

        #[derive(Component)]
        pub struct Rabit;

        fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
            let gltf = asset_server.load("models/rabit.glb");
            commands.insert_resource(RabitGltf(gltf));
        }

        fn load_scenes(
            mut commands: Commands,
            rabit_gltf: Res<RabitGltf>,
            assets_gltf: Res<Assets<Gltf>>,
            mut done: Local<bool>,
        ) {
            if !*done {
                if let Some(gltf) = assets_gltf.get(&rabit_gltf.0) {
                    commands.spawn((
                        SceneBundle {
                            scene: gltf.named_scenes["Scene"].clone(),
                            transform: Transform::from_xyz(0.5, 0.0, 0.0)
                                .with_rotation(Quat::from_rotation_y(TAU * -0.05)),
                            ..default()
                        },
                        Rabit,
                    ));
                    commands.spawn((
                        Animations {
                            list: HashMap::from([
                                ("bow".to_string(), gltf.named_animations["bow"].clone()),
                                ("clap".to_string(), gltf.named_animations["clap"].clone()),
                                (
                                    "greeting".to_string(),
                                    gltf.named_animations["greeting"].clone(),
                                ),
                                (
                                    "stay.bored".to_string(),
                                    gltf.named_animations["stay.bored"].clone(),
                                ),
                                (
                                    "stay.lookdown".to_string(),
                                    gltf.named_animations["stay.lookdown"].clone(),
                                ),
                            ]),
                        },
                        Rabit,
                    ));
                    *done = true;
                }
            }
        }

        fn start_stay(
            mut commands: Commands,
            animations: Query<&Animations, With<Rabit>>,
            mut players: Query<(Entity, &mut AnimationPlayer), Without<Rabit>>,
            scenes: Query<Entity, With<Rabit>>,
            parents: Query<&Parent>,
        ) {
            for anim in &animations {
                for (p_entity, mut player) in &mut players {
                    for s_entity in &scenes {
                        for parent in parents.iter_ancestors(p_entity) {
                            if parent == s_entity {
                                player
                                    .play(anim.list["stay.lookdown"].clone_weak())
                                    .repeat();
                                commands.entity(p_entity).insert(Rabit);
                            }
                        }
                    }
                }
            }
        }

        fn resume_stay(
            mut animation_player: Query<&mut AnimationPlayer, With<Rabit>>,
            animations: Query<&Animations, With<Rabit>>,
            assets_clip: Res<Assets<AnimationClip>>,
        ) {
            use bevy::animation::RepeatAnimation::Never;
            if let Ok(mut player) = animation_player.get_single_mut() {
                if let Some(clip) = assets_clip.get(player.animation_clip()) {
                    if let Ok(anim) = animations.get_single() {
                        let trasition_time = 0.25;
                        let remain = clip.duration() - player.seek_time();
                        if player.repeat_mode() == Never && remain <= trasition_time {
                            player
                                .play_with_transition(
                                    anim.list["stay.lookdown"].clone_weak(),
                                    Duration::from_secs_f32(trasition_time),
                                )
                                .repeat();
                        }
                    }
                }
            }
        }

        fn signal_animation_control(
            mut animation_player: Query<&mut AnimationPlayer, With<Rabit>>,
            animations: Query<&Animations, With<Rabit>>,
            mut signal_events: EventReader<BdsSignal>,
        ) {
            for BdsSignal { signal: sig } in signal_events.read() {
                if let Ok(mut player) = animation_player.get_single_mut() {
                    if let Ok(anim) = animations.get_single() {
                        if *sig == "Rabit greeting".to_string() {
                            player.play_with_transition(
                                anim.list["greeting"].clone_weak(),
                                Duration::from_millis(250),
                            );
                        } else if *sig == "Rabit clap".to_string() {
                            player
                                .play_with_transition(
                                    anim.list["clap"].clone_weak(),
                                    Duration::from_millis(250),
                                )
                                .repeat();
                        } else if *sig == "Rabit stay".to_string() {
                            player
                                .play_with_transition(
                                    anim.list["stay.lookdown"].clone_weak(),
                                    Duration::from_millis(250),
                                )
                                .repeat();
                        }
                    }
                }
            }
        }
    }

    mod girl {
        use super::*;

        pub struct GirlPlugin;
        impl Plugin for GirlPlugin {
            fn build(&self, app: &mut App) {
                app.add_systems(Startup, setup).add_systems(
                    Update,
                    (
                        load_scenes,
                        start_stay,
                        signal_animation_control,
                        resume_stay,
                    ),
                );
            }
        }

        #[derive(Resource)]
        pub struct GirlGltf(Handle<Gltf>);

        #[derive(Component)]
        pub struct Girl;

        fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
            let gltf = asset_server.load("models/girl.glb");
            commands.insert_resource(GirlGltf(gltf));
        }

        fn load_scenes(
            mut commands: Commands,
            girl_gltf: Res<GirlGltf>,
            assets_gltf: Res<Assets<Gltf>>,
            mut done: Local<bool>,
        ) {
            if !*done {
                if let Some(gltf) = assets_gltf.get(&girl_gltf.0) {
                    commands.spawn((
                        SceneBundle {
                            scene: gltf.named_scenes["Scene"].clone(),
                            transform: Transform::from_xyz(-0.5, 0.0, 0.0)
                                .with_rotation(Quat::from_rotation_y(TAU * 0.1)),
                            ..default()
                        },
                        Girl,
                    ));
                    commands.spawn((
                        Animations {
                            list: HashMap::from([
                                ("bow".to_string(), gltf.named_animations["bow"].clone()),
                                ("clap".to_string(), gltf.named_animations["clap"].clone()),
                                (
                                    "greeting".to_string(),
                                    gltf.named_animations["greeting"].clone(),
                                ),
                                (
                                    "stay.bored".to_string(),
                                    gltf.named_animations["stay.bored"].clone(),
                                ),
                                (
                                    "stay.lookdown".to_string(),
                                    gltf.named_animations["stay.lookdown"].clone(),
                                ),
                            ]),
                        },
                        Girl,
                    ));
                    *done = true;
                }
            }
        }

        fn start_stay(
            mut commands: Commands,
            animations: Query<&Animations, With<Girl>>,
            mut players: Query<(Entity, &mut AnimationPlayer), Without<Girl>>,
            scenes: Query<Entity, With<Girl>>,
            parents: Query<&Parent>,
        ) {
            for anim in &animations {
                for (p_entity, mut player) in &mut players {
                    for s_entity in &scenes {
                        for parent in parents.iter_ancestors(p_entity) {
                            if parent == s_entity {
                                player.play(anim.list["stay.bored"].clone_weak()).repeat();
                                commands.entity(p_entity).insert(Girl);
                            }
                        }
                    }
                }
            }
        }

        fn resume_stay(
            mut animation_player: Query<&mut AnimationPlayer, With<Girl>>,
            animations: Query<&Animations, With<Girl>>,
            assets_clip: Res<Assets<AnimationClip>>,
        ) {
            use bevy::animation::RepeatAnimation::Never;
            if let Ok(mut player) = animation_player.get_single_mut() {
                if let Some(clip) = assets_clip.get(player.animation_clip()) {
                    if let Ok(anim) = animations.get_single() {
                        let trasition_time = 0.25;
                        let remain = clip.duration() - player.seek_time();
                        if player.repeat_mode() == Never && remain <= trasition_time {
                            player
                                .play_with_transition(
                                    anim.list["stay.bored"].clone_weak(),
                                    Duration::from_secs_f32(trasition_time),
                                )
                                .repeat();
                        }
                    }
                }
            }
        }

        fn signal_animation_control(
            mut animation_player: Query<&mut AnimationPlayer, With<Girl>>,
            animations: Query<&Animations, With<Girl>>,
            mut signal_events: EventReader<BdsSignal>,
        ) {
            for BdsSignal { signal: sig } in signal_events.read() {
                if let Ok(mut player) = animation_player.get_single_mut() {
                    if let Ok(anim) = animations.get_single() {
                        if *sig == "Girl bow".to_string() {
                            player.play_with_transition(
                                anim.list["bow"].clone_weak(),
                                Duration::from_millis(250),
                            );
                        } else if *sig == "Girl clap".to_string() {
                            player
                                .play_with_transition(
                                    anim.list["clap"].clone_weak(),
                                    Duration::from_millis(250),
                                )
                                .repeat();
                        } else if *sig == "Girl stay".to_string() {
                            player
                                .play_with_transition(
                                    anim.list["stay.bored"].clone_weak(),
                                    Duration::from_millis(250),
                                )
                                .repeat();
                        }
                    }
                }
            }
        }
    }
}
