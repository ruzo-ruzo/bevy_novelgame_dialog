#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use bevy::prelude::*;
use bevy_novelgame_dialog::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            ui_templates::RPGStyleUIPlugin::default(),
            models_controller::ModelsControllerPlugin,
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
    pub use bevy::{gltf::Gltf, pbr::CascadeShadowConfigBuilder, prelude::*, utils::Duration};
    use std::collections::HashMap;
    use std::f32::consts::TAU;

    pub struct ModelsControllerPlugin;
    impl Plugin for ModelsControllerPlugin {
        fn build(&self, app: &mut App) {
            app.add_plugins((girl::GirlPlugin, rabit::RabitPlugin, room::RoomPlugin))
                .add_systems(Update, start_animation);
        }
    }

    #[derive(Component)]
    struct Animations {
        gltf: Handle<Gltf>,
        list: HashMap<String, Handle<AnimationClip>>,
    }

    #[derive(Component)]
    struct Owned(Handle<Gltf>);

    #[derive(Resource)]
    pub struct Girl(Handle<Gltf>);

    #[derive(Resource)]
    pub struct Rabit(Handle<Gltf>);

    fn start_animation(
        mut commands: Commands,
        animations: Query<&Animations>,
        mut players: Query<(Entity, &mut AnimationPlayer), Without<Owned>>,
        girl: Res<Girl>,
        rabit: Res<Rabit>,
    ) {
        let res_vec = vec![
            (rabit.0.clone(), "stay.lookdown"),
            (girl.0.clone(), "stay.bored"),
        ];
        if players.iter().len() == res_vec.len() && res_vec.len() == animations.iter().len() {
            for ((entity, mut player), owner) in players.iter_mut().zip(res_vec.iter()) {
                for anim in &animations {
                    if anim.gltf == owner.0 {
                        player.play(anim.list[owner.1].clone_weak()).repeat();
                        commands.entity(entity).insert(Owned(anim.gltf.clone()));
                        info!("{entity:?}");
                    }
                }
            }
        }
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
                        // signal_animation_control
                    ),
                );
            }
        }

        fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
            let gltf = asset_server.load("models/rabit.glb");
            commands.insert_resource(Rabit(gltf));
        }

        fn load_scenes(
            mut commands: Commands,
            rabit: Res<Rabit>,
            assets_gltf: Res<Assets<Gltf>>,
            mut done: Local<bool>,
        ) {
            if !*done {
                if let Some(gltf) = assets_gltf.get(&rabit.0) {
                    commands.spawn(SceneBundle {
                        scene: gltf.named_scenes["Scene"].clone(),
                        transform: Transform::from_xyz(0.5, 0.0, 0.0)
                            .with_rotation(Quat::from_rotation_y(TAU * -0.05)),
                        ..default()
                    });
                    commands.spawn(Animations {
                        gltf: rabit.0.clone(),
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
                    });
                    *done = true;
                }
            }
        }

        // fn start_animation(
        // mut commands: Commands,
        // animations: Query<&Animations>,
        // mut players: Query<(Entity, &mut AnimationPlayer), Without<Owned>>,
        // owners: Query<&Owned>,
        // rabit: Res<Rabit>,
        // ) {
        // if owners.iter().find(|x| x.0 == rabit.0).is_none() {
        // for anim in &animations {
        // if anim.gltf == rabit.0 {
        // for (entity, mut player) in &mut players {
        // player
        // .play(anim.list["stay.lookdown"].clone_weak())
        // .repeat();
        // commands.entity(entity).insert(Owned(anim.gltf.clone()));
        // return;
        // }
        // }
        // }
        // }
        // }
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
                        // signal_animation_control
                    ),
                );
            }
        }

        fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
            let gltf = asset_server.load("models/girl.glb");
            commands.insert_resource(Girl(gltf));
        }

        fn load_scenes(
            mut commands: Commands,
            girl: Res<Girl>,
            assets_gltf: Res<Assets<Gltf>>,
            mut done: Local<bool>,
        ) {
            if !*done {
                if let Some(gltf) = assets_gltf.get(&girl.0) {
                    commands.spawn(SceneBundle {
                        scene: gltf.named_scenes["Scene"].clone(),
                        transform: Transform::from_xyz(-0.5, 0.0, 0.0)
                            .with_rotation(Quat::from_rotation_y(TAU * 0.1)),
                        ..default()
                    });
                    commands.spawn(Animations {
                        gltf: girl.0.clone(),
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
                    });
                    *done = true;
                }
            }
        }

        // fn start_animation(
        // mut commands: Commands,
        // animations: Query<&Animations>,
        // mut players: Query<(Entity, &mut AnimationPlayer), Without<Owned>>,
        // owners: Query<&Owned>,
        // girl: Res<Girl>,
        // ) {
        // if owners.iter().find(|x| x.0 == girl.0).is_none() {
        // for anim in &animations {
        // if anim.gltf == girl.0 {
        // for (entity, mut player) in &mut players {
        // player.play(anim.list["stay.bored"].clone_weak()).repeat();
        // commands.entity(entity).insert(Owned(anim.gltf.clone()));
        // return;
        // }
        // }
        // }
        // }
        // }
    }
}
