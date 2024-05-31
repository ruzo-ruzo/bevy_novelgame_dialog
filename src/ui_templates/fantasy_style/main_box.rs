use super::*;
use bevy::render::view::RenderLayers;

pub(super) struct MainBoxPlugIn;

impl Plugin for MainBoxPlugIn {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "../assets/fantasy_style/textures/ui/dialog_box_01.png");
        embedded_asset!(app, "../assets/fantasy_style/textures/ui/dialog_box_02.png");
        embedded_asset!(app, "../assets/fantasy_style/textures/ui/name_plate.png");
        embedded_asset!(app, "../assets/fantasy_style/textures/ui/cursor.png");
        app.add_systems(Startup, setup_messageframe)
            .add_systems(Startup, waiting_sprite_setup)
            .add_systems(Update, setup_name_plate)
            .add_systems(Update, animate_sprite);
    }
}

#[derive(Component)]
struct WaitingSprite;

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
    let dialog_box_image_handle =
        asset_server.load(ASSETS_PATH.to_owned() + "fantasy_style/textures/ui/dialog_box_02.png");
    let dialog_box_slice = ImageScaleMode::Sliced(TextureSlicer {
        border: BorderRect::rectangle(55.0, 71.0),
        ..default()
    });
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(1200.0, 300.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -200.0, 0.0),
            texture: dialog_box_image_handle,
            ..default()
        },
        dialog_box_slice,
        DialogBoxBackground {
            dialog_box_name: "Main Box".to_string(),
        },
    ));
}

fn setup_name_plate(
    mut commands: Commands,
    dbb_query: Query<(Entity, &DialogBoxBackground)>,
    config: Res<TemplateSetupConfig>,
    asset_server: Res<AssetServer>,
    mut is_setup: Local<bool>,
) {
    if !*is_setup {
        let name_plate_image_handle =
            asset_server.load(ASSETS_PATH.to_owned() + "fantasy_style/textures/ui/name_plate.png");
        for (
            dbb_entity,
            DialogBoxBackground {
                dialog_box_name: name,
            },
        ) in &dbb_query
        {
            if name == "Main Box" {
                commands.entity(dbb_entity).with_children(|child_builder| {
                    child_builder.spawn((
                        SpriteBundle {
                            transform: Transform::from_xyz(-350.0, 130.0, 0.1),
                            texture: name_plate_image_handle.clone(),
                            visibility: Visibility::Inherited,
                            ..default()
                        },
                        RenderLayers::layer(config.render_layer),
                    ));
                });
                *is_setup = true;
            }
        }
    }
}

fn waiting_sprite_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture_image_path = "fantasy_style/textures/ui/cursor.png";
    let texture_handle = asset_server.load(ASSETS_PATH.to_owned() + texture_image_path);
    let texture_atlas = TextureAtlasLayout::from_grid(Vec2::new(44.0, 56.0), 1, 2, None, None);
    let animation_indices = AnimationIndices {
        first: 0,
        last: 1,
        step: 1,
    };
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    let wi_sprite = Sprite::default();
    commands.spawn((
        SpriteSheetBundle {
            atlas: TextureAtlas {
                layout: texture_atlas_handle,
                index: animation_indices.first,
            },
            sprite: wi_sprite,
            transform: Transform::from_scale(Vec3::splat(0.5)),
            texture: texture_handle,
            ..default()
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.5, TimerMode::Repeating)),
        WaitingIcon {
            target_box_name: "Main Box".to_string(),
        },
        WaitingSprite,
    ));
}
