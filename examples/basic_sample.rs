use bevy::prelude::AnimationNodeType::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            message_controler::MessageControllerPlugin,
            models_controller::ModelsControllerPlugin,
        ))
        .run();
}

mod message_controler {
    use bevy::prelude::*;
    use bevy_novelgame_dialog::ui_templates::*;

    pub struct MessageControllerPlugin;
    impl Plugin for MessageControllerPlugin {
        fn build(&self, app: &mut App) {
            app.add_plugins(RoseStyleUIPlugin {
                max_button_index: 4,
                ..default()
            })
            .add_systems(Update, start_message);
        }
    }

    fn start_message(
        mut open_message_event: EventWriter<OpenRoseStyleDialog>,
        mut is_started: Local<bool>,
    ) {
        if !*is_started {
            let event = OpenRoseStyleDialog {
                script_path: "scripts/starter.md".to_string(),
            };
            open_message_event.write(event);
            *is_started = true;
        }
    }
}

mod models_controller {
    use super::*;
    use bevy::gltf::Gltf;
    use bevy_novelgame_dialog::prelude::BdsSignal;
    use core::time::Duration;
    use std::collections::HashMap;
    use std::f32::consts::TAU;

    pub struct ModelsControllerPlugin;
    impl Plugin for ModelsControllerPlugin {
        fn build(&self, app: &mut App) {
            app.add_plugins((rabit::RabitPlugin, kid::KidPlugin, room::RoomPlugin));
        }
    }

    mod room {
        use super::*;
        use bevy::pbr::CascadeShadowConfigBuilder;

        pub struct RoomPlugin;
        impl Plugin for RoomPlugin {
            fn build(&self, app: &mut App) {
                app.insert_resource(AmbientLight {
                    color: Color::WHITE,
                    brightness: 0.0,
                    ..default()
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
            commands.spawn((
                Camera3d::default(),
                Transform::from_xyz(0.0, 0.6, 2.0).looking_at(Vec3::new(0.0, 0.6, 0.0), Vec3::Y),
            ));
            commands.spawn((
                DirectionalLight {
                    shadows_enabled: true,
                    ..default()
                },
                Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -TAU / 8.)),
                CascadeShadowConfigBuilder {
                    first_cascade_far_bound: 200.0,
                    maximum_distance: 400.0,
                    ..default()
                }
                .build(),
            ));
        }

        fn load_scenes(
            mut commands: Commands,
            room: Res<Room>,
            assets_gltf: Res<Assets<Gltf>>,
            mut done: Local<bool>,
        ) {
            if !*done {
                if let Some(gltf) = assets_gltf.get(&room.0) {
                    commands.spawn(SceneRoot(gltf.named_scenes["Scene"].clone()));
                    *done = true;
                }
            }
        }
    }

    mod rabit {
        use super::*;

        const TRASITION_TIME: f32 = 0.5;

        pub struct RabitPlugin;
        impl Plugin for RabitPlugin {
            fn build(&self, app: &mut App) {
                app.add_systems(Startup, setup);
                app.add_systems(Update, load_scenes);
                app.add_systems(Update, start_stay);
                app.add_systems(Update, resume_stay);
                app.add_systems(Update, signal_animation_control);
            }
        }

        #[derive(Resource)]
        pub struct RabitGltf(Handle<Gltf>);

        #[derive(Resource)]
        struct RabitAnimations {
            name: HashMap<String, AnimationNodeIndex>,
            graph: Handle<AnimationGraph>,
        }

        #[derive(Component)]
        pub struct Rabit;

        fn setup(
            mut commands: Commands,
            asset_server: Res<AssetServer>,
            graphs: Res<Assets<AnimationGraph>>,
        ) {
            let gltf = asset_server.load("models/rabit.glb");
            commands.insert_resource(RabitGltf(gltf));
            let animations: HashMap<String, AnimationNodeIndex> = HashMap::new();
            let graph = graphs.reserve_handle();
            commands.insert_resource(RabitAnimations {
                name: animations,
                graph: graph.clone(),
            });
        }

        fn load_scenes(
            mut commands: Commands,
            rabit_gltf: Res<RabitGltf>,
            mut graphs: ResMut<Assets<AnimationGraph>>,
            assets_gltf: Res<Assets<Gltf>>,
            mut rabit_animations: ResMut<RabitAnimations>,
            mut done: Local<bool>,
        ) {
            if !*done {
                if let Some(gltf) = assets_gltf.get(&rabit_gltf.0) {
                    let mut graph = AnimationGraph::new();
                    let mut animations: HashMap<String, AnimationNodeIndex> = HashMap::new();
                    for (key, clip) in &gltf.named_animations {
                        let index = graph.add_clip(clip.clone(), 1.0, graph.root);
                        animations.insert(key.to_string(), index);
                    }
                    commands.spawn((
                        SceneRoot(gltf.named_scenes["Scene"].clone()),
                        Transform::from_xyz(0.5, 0.0, 0.0)
                            .with_rotation(Quat::from_rotation_y(TAU * -0.05)),
                        Rabit,
                    ));
                    let graph = graphs.add(graph);
                    *rabit_animations = RabitAnimations {
                        name: animations,
                        graph: graph.clone(),
                    };
                    *done = true;
                }
            }
        }

        fn start_stay(
            mut commands: Commands,
            mut players: Query<(Entity, &mut AnimationPlayer), Without<Rabit>>,
            scenes: Query<Entity, With<Rabit>>,
            parents: Query<&ChildOf>,
            animations: Res<RabitAnimations>,
        ) {
            for (p_entity, mut player) in &mut players {
                for s_entity in &scenes {
                    for parent in parents.iter_ancestors(p_entity) {
                        if parent == s_entity {
                            if let Some(node) = animations.name.get("stay.lookdown") {
                                let mut transitions = AnimationTransitions::new();
                                transitions
                                    .play(&mut player, *node, Duration::ZERO)
                                    .repeat();
                                commands
                                    .entity(p_entity)
                                    .insert(AnimationGraphHandle(animations.graph.clone()))
                                    .insert(transitions)
                                    .insert(Rabit);
                            }
                        }
                    }
                }
            }
        }

        fn resume_stay(
            mut animation_player: Query<
                (&mut AnimationPlayer, &mut AnimationTransitions),
                With<Rabit>,
            >,
            animations: Res<RabitAnimations>,
            graphs: Res<Assets<AnimationGraph>>,
            clips: Res<Assets<AnimationClip>>,
        ) {
            use bevy::animation::RepeatAnimation::*;
            if let Ok((mut player, mut transition)) = animation_player.single_mut() {
                if let Some(index) = transition.get_main_animation() {
                    if let Some(active) = player.animation(index) {
                        if let Some(graph) = graphs.get(&animations.graph) {
                            if let Some(Clip(clip_handle)) =
                                graph.get(index).map(|x| x.node_type.clone())
                            {
                                if let Some(clip) = clips.get(&clip_handle) {
                                    let remain = clip.duration() - active.seek_time();
                                    if active.repeat_mode() == Never && remain <= TRASITION_TIME {
                                        transition
                                            .play(
                                                &mut player,
                                                animations.name["stay.lookdown"],
                                                Duration::from_secs_f32(TRASITION_TIME),
                                            )
                                            .repeat();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        fn signal_animation_control(
            mut animation_player: Query<
                (&mut AnimationPlayer, &mut AnimationTransitions),
                With<Rabit>,
            >,
            animations: Res<RabitAnimations>,
            mut signal_events: EventReader<BdsSignal>,
        ) {
            let time = Duration::from_secs_f32(TRASITION_TIME);
            for BdsSignal { signal: sig } in signal_events.read() {
                if let Ok((mut player, mut transition)) = animation_player.single_mut() {
                    if let Some(current) = transition.get_main_animation() {
                        if *sig == "Rabit_greeting" {
                            if let Some(next) = animations.name.get("greeting") {
                                if *next != current {
                                    transition.play(&mut player, *next, time);
                                }
                            }
                        } else if *sig == "Rabit_clap" {
                            if let Some(next) = animations.name.get("clap") {
                                if *next != current {
                                    transition.play(&mut player, *next, time).repeat();
                                }
                            }
                        } else if *sig == "Rabit_stay" {
                            if let Some(next) = animations.name.get("stay.lookdown") {
                                if *next != current {
                                    transition.play(&mut player, *next, time).repeat();
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    mod kid {
        use super::*;

        const TRASITION_TIME: f32 = 0.5;

        pub struct KidPlugin;
        impl Plugin for KidPlugin {
            fn build(&self, app: &mut App) {
                app.add_systems(Startup, setup);
                app.add_systems(Update, load_scenes);
                app.add_systems(Update, start_stay);
                app.add_systems(Update, resume_stay);
                app.add_systems(Update, signal_animation_control);
            }
        }

        #[derive(Resource)]
        pub struct KidGltf(Handle<Gltf>);

        #[derive(Resource)]
        struct KidAnimations {
            name: HashMap<String, AnimationNodeIndex>,
            graph: Handle<AnimationGraph>,
        }

        #[derive(Component)]
        pub struct Kid;

        fn setup(
            mut commands: Commands,
            asset_server: Res<AssetServer>,
            graphs: Res<Assets<AnimationGraph>>,
        ) {
            let gltf = asset_server.load("models/kid.glb");
            commands.insert_resource(KidGltf(gltf));
            let animations: HashMap<String, AnimationNodeIndex> = HashMap::new();
            let graph = graphs.reserve_handle();
            commands.insert_resource(KidAnimations {
                name: animations,
                graph: graph.clone(),
            });
        }

        fn load_scenes(
            mut commands: Commands,
            kid_gltf: Res<KidGltf>,
            mut graphs: ResMut<Assets<AnimationGraph>>,
            assets_gltf: Res<Assets<Gltf>>,
            mut kid_animations: ResMut<KidAnimations>,
            mut done: Local<bool>,
        ) {
            if !*done {
                if let Some(gltf) = assets_gltf.get(&kid_gltf.0) {
                    let mut graph = AnimationGraph::new();
                    let mut animations: HashMap<String, AnimationNodeIndex> = HashMap::new();
                    for (key, clip) in &gltf.named_animations {
                        let index = graph.add_clip(clip.clone(), 1.0, graph.root);
                        animations.insert(key.to_string(), index);
                    }
                    commands.spawn((
                        SceneRoot(gltf.named_scenes["Scene"].clone()),
                        Transform::from_xyz(-0.5, 0.0, 0.0)
                            .with_rotation(Quat::from_rotation_y(TAU * 0.1)),
                        Kid,
                    ));
                    let graph = graphs.add(graph);
                    *kid_animations = KidAnimations {
                        name: animations,
                        graph: graph.clone(),
                    };
                    *done = true;
                }
            }
        }

        fn start_stay(
            mut commands: Commands,
            mut players: Query<(Entity, &mut AnimationPlayer), Without<Kid>>,
            scenes: Query<Entity, With<Kid>>,
            parents: Query<&ChildOf>,
            animations: Res<KidAnimations>,
        ) {
            for (p_entity, mut player) in &mut players {
                for s_entity in &scenes {
                    for parent in parents.iter_ancestors(p_entity) {
                        if parent == s_entity {
                            if let Some(node) = animations.name.get("stay.bored") {
                                let mut transitions = AnimationTransitions::new();
                                transitions
                                    .play(&mut player, *node, Duration::ZERO)
                                    .repeat();
                                commands
                                    .entity(p_entity)
                                    .insert(AnimationGraphHandle(animations.graph.clone()))
                                    .insert(transitions)
                                    .insert(Kid);
                            }
                        }
                    }
                }
            }
        }

        fn resume_stay(
            mut animation_player: Query<
                (&mut AnimationPlayer, &mut AnimationTransitions),
                With<Kid>,
            >,
            animations: Res<KidAnimations>,
            graphs: Res<Assets<AnimationGraph>>,
            clips: Res<Assets<AnimationClip>>,
        ) {
            use bevy::animation::RepeatAnimation::*;
            if let Ok((mut player, mut transition)) = animation_player.single_mut() {
                if let Some(index) = transition.get_main_animation() {
                    if let Some(active) = player.animation(index) {
                        if let Some(graph) = graphs.get(&animations.graph) {
                            if let Some(Clip(clip_handle)) =
                                graph.get(index).map(|x| x.node_type.clone())
                            {
                                if let Some(clip) = clips.get(&clip_handle) {
                                    let remain = clip.duration() - active.seek_time();
                                    if active.repeat_mode() == Never && remain <= TRASITION_TIME {
                                        transition
                                            .play(
                                                &mut player,
                                                animations.name["stay.bored"],
                                                Duration::from_secs_f32(TRASITION_TIME),
                                            )
                                            .repeat();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        fn signal_animation_control(
            mut animation_player: Query<
                (&mut AnimationPlayer, &mut AnimationTransitions),
                With<Kid>,
            >,
            animations: Res<KidAnimations>,
            mut signal_events: EventReader<BdsSignal>,
        ) {
            let time = Duration::from_secs_f32(TRASITION_TIME);
            for BdsSignal { signal: sig } in signal_events.read() {
                if let Ok((mut player, mut transition)) = animation_player.single_mut() {
                    if let Some(current) = transition.get_main_animation() {
                        if *sig == "Kid_bow" {
                            if let Some(next) = animations.name.get("bow") {
                                if *next != current {
                                    transition.play(&mut player, *next, time);
                                }
                            }
                        } else if *sig == "Kid_clap" {
                            if let Some(next) = animations.name.get("clap") {
                                if *next != current {
                                    transition.play(&mut player, *next, time).repeat();
                                }
                            }
                        } else if *sig == "Kid_stay" {
                            if let Some(next) = animations.name.get("stay.bored") {
                                if *next != current {
                                    transition.play(&mut player, *next, time).repeat();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
