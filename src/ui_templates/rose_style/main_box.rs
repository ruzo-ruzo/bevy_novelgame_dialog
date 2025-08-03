use super::*;
use bevy::render::view::RenderLayers;

pub(super) struct MainBoxPlugIn;

impl Plugin for MainBoxPlugIn {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "assets/textures/ui/plate_base.png");
        embedded_asset!(app, "assets/textures/ui/rose_plate.png");
        embedded_asset!(app, "assets/textures/ui/waiting.png");
        embedded_asset!(app, "assets/textures/ui/wait_feeding.png");
        app.add_systems(Startup, setup_messageframe)
            .add_systems(Startup, waiting_sprite_setup)
            .add_systems(Startup, wait_feeding_sprite_setup)
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

fn setup_messageframe(
    mut commands: Commands,
    config: Res<TemplateSetupConfig>,
    asset_server: Res<AssetServer>,
) {
    let writing_image_handle =
        asset_server.load(ASSETS_PATH.to_owned() + "textures/ui/plate_base.png");
    let writing_slice = SpriteImageMode::Sliced(TextureSlicer {
        border: BorderRect::axes(360.0, 360.0),
        center_scale_mode: SliceScaleMode::Tile {
            stretch_value: 0.33,
        },
        sides_scale_mode: SliceScaleMode::Tile {
            stretch_value: 0.33,
        },
        ..default()
    });
    commands.spawn((
        Sprite {
            image: writing_image_handle,
            custom_size: Some(config.box_size),
            image_mode: writing_slice,
            ..default()
        },
        Transform::from_translation(config.box_pos.extend(0.0)),
        DialogBoxBackground {
            writing_name: "Main Box".to_string(),
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
            asset_server.load(ASSETS_PATH.to_owned() + "textures/ui/rose_plate.png");
        let writing_slice = SpriteImageMode::Sliced(TextureSlicer {
            border: BorderRect::axes(64.0, 35.0),
            ..default()
        });
        for (dbb_entity, DialogBoxBackground { writing_name: name }) in &dbb_query {
            if name == "Main Box" {
                let name_x = -(config.box_size.x / 2.0) + (config.box_pos.x + 280.0);
                let name_y = config.box_size.y / 2.0 + (config.box_pos.y + 142.0);
                commands.entity(dbb_entity).with_children(|child_builder| {
                    child_builder.spawn((
                        Sprite {
                            image: name_plate_image_handle.clone(),
                            custom_size: Some(config.name_plate_size),
                            image_mode: writing_slice.clone(),
                            ..default()
                        },
                        Transform::from_xyz(name_x, name_y, 0.1),
                        Visibility::Inherited,
                        RenderLayers::layer(config.render_layer.into()),
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
    let texture_image_path = "textures/ui/waiting.png";
    let texture_handle = asset_server.load(ASSETS_PATH.to_owned() + texture_image_path);
    let texture_atlas = TextureAtlasLayout::from_grid(UVec2::new(128, 128), 1, 2, None, None);
    let animation_indices = AnimationIndices {
        first: 0,
        last: 1,
        step: 1,
    };
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    let wi_sprite = Sprite {
        image: texture_handle,
        texture_atlas: Some(TextureAtlas {
            layout: texture_atlas_handle,
            index: animation_indices.first,
        }),
        ..default()
    };
    commands.spawn((
        wi_sprite,
        Transform::from_scale(Vec3::splat(0.5)),
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.5, TimerMode::Repeating)),
        WaitingIcon {
            target_box_name: "Main Box".to_string(),
            wait_for: vec![WaitTarget::SimpleWaiting],
        },
        WaitingSprite,
    ));
}

fn wait_feeding_sprite_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture_image_path = "textures/ui/wait_feeding.png";
    let texture_handle = asset_server.load(ASSETS_PATH.to_owned() + texture_image_path);
    let texture_atlas = TextureAtlasLayout::from_grid(UVec2::new(128, 128), 1, 2, None, None);
    let animation_indices = AnimationIndices {
        first: 0,
        last: 1,
        step: 1,
    };
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    let wi_sprite = Sprite {
        image: texture_handle,
        texture_atlas: Some(TextureAtlas {
            layout: texture_atlas_handle,
            index: animation_indices.first,
        }),
        ..default()
    };
    commands.spawn((
        wi_sprite,
        Transform::from_scale(Vec3::splat(0.5)),
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.5, TimerMode::Repeating)),
        WaitingIcon {
            target_box_name: "Main Box".to_string(),
            wait_for: vec![WaitTarget::Feeding],
        },
        WaitingSprite,
    ));
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut Sprite)>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if let Some(ref mut atlas) = &mut sprite.texture_atlas {
            if timer.just_finished() {
                atlas.index = if atlas.index >= indices.last {
                    indices.first
                } else {
                    atlas.index + indices.step
                };
            }
        }
    }
}
