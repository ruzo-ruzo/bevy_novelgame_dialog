use crate::prelude::*;
use bevy::asset::embedded_asset;
use bevy::prelude::*;

const ASSETS_PATH: &str = "embedded://bevy_novelgame_dialog/ui_templates/assets/";

#[derive(Resource, Default)]
pub struct TemplateSetupConfig {
    pub render_layer: u8,
    pub render_order: isize,
}

pub struct RPGStyleUIPlugin {
    pub layer_num: u8,
    pub render_order: isize,
}

impl Default for RPGStyleUIPlugin {
    fn default() -> Self {
        RPGStyleUIPlugin {
            layer_num: 2,
            render_order: 1,
        }
    }
}

impl Plugin for RPGStyleUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DialogBoxPlugin {
                layer_num: self.layer_num,
                render_order: self.render_order,
            },
            EmbeddedAssetPlugin,
        ))
        .insert_resource(TemplateSetupConfig {
            render_layer: self.layer_num,
            render_order: self.render_order,
        })
        .add_event::<OpenRPGStyleDialog>()
        .add_systems(Startup, waiting_sprite_setup)
        .add_systems(Startup, setup_messageframe)
        .add_systems(Startup, setup_choice_images)
        .add_systems(Update, setup_name_plate)
        .add_systems(Update, open_message)
        .add_systems(Update, animate_sprite)
        .add_systems(Update, move_cursor)
        .add_systems(Update, reset_images)
        .add_systems(Update, button_clicked);
    }
}

struct EmbeddedAssetPlugin;

impl Plugin for EmbeddedAssetPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "assets/common/scripts/basic.csv");
        embedded_asset!(app, "assets/fantasy_style/scripts/basic.csv");
        embedded_asset!(app, "assets/fantasy_style/textures/ui/dialog_box_01.png");
        embedded_asset!(app, "assets/fantasy_style/textures/ui/dialog_box_02.png");
        embedded_asset!(app, "assets/fantasy_style/textures/ui/name_plate.png");
        embedded_asset!(app, "assets/fantasy_style/textures/ui/cursor.png");
        embedded_asset!(
            app,
            "assets/fantasy_style/textures/ui/choice_buttons/button_default.png"
        );
        embedded_asset!(
            app,
            "assets/fantasy_style/textures/ui/choice_buttons/button_pushed.png"
        );
        embedded_asset!(
            app,
            "assets/fantasy_style/textures/ui/choice_buttons/choicing_frame.png"
        );
        embedded_asset!(
            app,
            "assets/fantasy_style/fonts/UnifrakturMaguntia/UnifrakturMaguntia-Regular.ttf"
        );
        embedded_asset!(
            app,
            "assets/fantasy_style/fonts/赤薔薇/akabara-cinderella.ttf"
        );
        embedded_asset!(app, "assets/fantasy_style/fonts/网风雅宋/网风雅宋.ttf");
        embedded_asset!(
            app,
            "assets/fantasy_style/fonts/noto/NotoEmoji-VariableFont_wght.ttf"
        );
    }
}

#[derive(Event)]
pub struct OpenRPGStyleDialog {
    pub script_path: String,
}

fn open_message(
    mut open_message_event: EventReader<OpenRPGStyleDialog>,
    mut ow_event: EventWriter<OpenDialog>,
) {
    for OpenRPGStyleDialog { script_path: path } in open_message_event.read() {
        let font_path_vec = [
            "UnifrakturMaguntia/UnifrakturMaguntia-Regular.ttf",
            "赤薔薇/akabara-cinderella.ttf",
            "网风雅宋/网风雅宋.ttf",
            "noto/NotoEmoji-VariableFont_wght.ttf",
        ]
        .iter()
        .map(|s| ASSETS_PATH.to_owned() + "fantasy_style/fonts/" + s);
        let font_vec = font_path_vec
            .zip([(1.0, 0.0), (1.0, 0.0), (1.3, 0.0), (1.0, 0.0)].iter())
            .map(|(p, (s, k))| FontConfig {
                path: p.clone(),
                kerning: *k,
                size: *s,
            })
            .collect::<Vec<_>>();
        let frame_tac = TextAreaConfig {
            area_name: "Main Area".to_string(),
            font_sets: font_vec.clone(),
            feeding: FeedingStyle::Scroll { size: 0, sec: 0.5 },
            font_color: Color::rgb(0.7, 0.5, 0.3),
            text_base_size: 32.0,
            area_origin: Vec2::new(-520.0, 70.0),
            area_size: Vec2::new(1010.0, 140.0),
            // monospace: true,
            ..default()
        };
        let name_plate_tac = TextAreaConfig {
            area_name: "Name Area".to_string(),
            area_origin: Vec2::new(-480.0, 170.0),
            area_size: Vec2::new(400.0, 80.0),
            font_sets: font_vec.clone(),
            font_color: Color::ANTIQUE_WHITE,
            text_base_size: 32.0,
            feeding: FeedingStyle::Rid,
            writing: WritingStyle::Put,
            typing_timing: TypingTiming::ByPage,
            vertical_alignment: AlignVertical::Center,
            ..default()
        };
        let tac_base = TextAreaConfig {
            font_sets: font_vec.clone(),
            area_size: Vec2::new(360.0, 100.0),
            font_color: Color::NAVY,
            writing: WritingStyle::Put,
            typing_timing: TypingTiming::ByPage,
            horizon_alignment: AlignHorizon::Center,
            vertical_alignment: AlignVertical::Center,
            ..default()
        };
        let tac_list = (0..4)
            .map(|i| TextAreaConfig {
                area_origin: Vec2::new(-180.0, -20.0 - 140.0 * (i as f32)),
                area_name: format!("Button Area {i:02}"),
                ..tac_base.clone()
            })
            .collect::<Vec<_>>();
        ow_event.send(OpenDialog {
            dialog_box_name: "Main Box".to_string(),
            script_path: path.clone(),
            template_path: vec![
                (ASSETS_PATH.to_owned() + "fantasy_style/scripts/basic.csv").to_string(),
                (ASSETS_PATH.to_owned() + "common/scripts/basic.csv").to_string(),
            ],
            text_area_configs: vec![frame_tac, name_plate_tac],
            position: Vec2::new(0., -200.),
            wait_breaker: WaitBrakerStyle::Input {
                is_icon_moving_to_last: true,
                is_all_range_area: true,
            },
            template_open_choice: ChoiceBoxConfig {
                choice_box_name: "Choice Box".to_string(),
                button_text_areas: tac_list,
                background_scaling_per_button: Vec2::new(0.0, 140.0),
                background_scaling_anchor: Anchor::TopCenter,
                ..default()
            },
            ..default()
        });
    }
}

//----------
use bevy::sprite::Anchor;

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

//----------
use bevy::render::view::RenderLayers;

#[derive(Component)]
struct ChoiceCursor;

#[derive(Component)]
struct PushedButton;

fn setup_choice_images(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<TemplateSetupConfig>,
) {
    let button_image_path = "fantasy_style/textures/ui/choice_buttons/button_default.png";
    let pushed_image_path = "fantasy_style/textures/ui/choice_buttons/button_pushed.png";
    let frame_image_path = "fantasy_style/textures/ui/choice_buttons/choicing_frame.png";
    let box_image_path = "fantasy_style/textures/ui/dialog_box_01.png";
    let button_image_handle = asset_server.load(ASSETS_PATH.to_owned() + button_image_path);
    let pushed_image_handle = asset_server.load(ASSETS_PATH.to_owned() + pushed_image_path);
    let choicing_frame_image_handle = asset_server.load(ASSETS_PATH.to_owned() + frame_image_path);
    let dialog_box_image_handle = asset_server.load(ASSETS_PATH.to_owned() + box_image_path);
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
                custom_size: Some(Vec2::new(400., 100.)),
                ..default()
            },
            texture: button_image_handle.clone(),
            transform: Transform::from_xyz(0.0, -70.0 - 140.0 * (i as f32), 0.6),
            ..default()
        };
        let cb = ChoiceButton {
            target_box_name: "Choice Box".to_string(),
            sort_number: i,
        };
        commands.spawn((button_sprite_bundle, button_slice.clone(), cb));
    }
    let frame_sprite_bundle = SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(600., 100.)),
            ..default()
        },
        texture: dialog_box_image_handle,
        transform: Transform::from_xyz(0., 0., 1.1),
        ..default()
    };
    let pushed_sprite_bundle = SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(400., 100.)),
            ..default()
        },
        texture: pushed_image_handle,
        transform: Transform::from_xyz(0.0, -70.0, 0.7),
        visibility: Visibility::Hidden,
        ..default()
    };
    let cursor_sprite_bundle = SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(480., 200.)),
            ..default()
        },
        texture: choicing_frame_image_handle,
        transform: Transform::from_xyz(-2., 200., 0.3),
        visibility: Visibility::Hidden,
        ..default()
    };
    commands
        .spawn((
            frame_sprite_bundle,
            dialog_box_slice,
            DialogBoxBackground {
                dialog_box_name: "Choice Box".to_string(),
            },
        ))
        .with_children(|c| {
            c.spawn((
                cursor_sprite_bundle,
                choicing_frame_slice,
                ChoiceCursor,
                RenderLayers::layer(config.render_layer),
            ));
            c.spawn((
                pushed_sprite_bundle,
                PushedButton,
                button_slice,
                RenderLayers::layer(config.render_layer),
            ));
        });
}

fn move_cursor(
    mut cursor_query: Query<(Entity, &mut Visibility), With<ChoiceCursor>>,
    button_query: Query<(Entity, &ChoiceButton)>,
    mut tf_query: Query<&mut Transform>,
    mut events: EventReader<ButtonIsSelected>,
) {
    for se in events.read() {
        let cb_opt = button_query
            .iter()
            .find(|x| x.1.sort_number == se.select_number);
        if let Some((button_entity, _)) = cb_opt {
            if let Ok((choice_entity, mut vis)) = cursor_query.get_single_mut() {
                let cb_y_opt = tf_query.get(button_entity).map(|x| x.translation.y);
                if let Ok(mut cc_tf) = tf_query.get_mut(choice_entity) {
                    cc_tf.translation.y = cb_y_opt.unwrap_or_default() + 5.0;
                }
                *vis = Visibility::Inherited;
            }
        }
    }
}

fn reset_images(
    mut cursor_query: Query<&mut Visibility, With<ChoiceCursor>>,
    mut pushed_query: Query<&mut Visibility, (With<PushedButton>, Without<ChoiceCursor>)>,
    mut events: EventReader<FinisClosingBox>,
) {
    for fcb in events.read() {
        if fcb.dialog_box_name == *"Choice Box" {
            if let Ok(mut vis) = cursor_query.get_single_mut() {
                *vis = Visibility::Hidden;
            }
            if let Ok(mut vis) = pushed_query.get_single_mut() {
                *vis = Visibility::Hidden;
            }
        }
    }
}

fn button_clicked(
    mut pushed_query: Query<(&mut Transform, &mut Visibility), With<PushedButton>>,
    button_query: Query<(&Transform, &ChoiceButton), Without<PushedButton>>,
    mut events: EventReader<ButtonIsPushed>,
) {
    for gse in events.read() {
        if gse.dialog_box_name == *"Choice Box" {
            for (button_tf, cb) in &button_query {
                let ta_name = format!("Button Area {:02}", cb.sort_number);
                if gse.text_area_name == ta_name {
                    if let Ok((mut pushed_tf, mut vis)) = pushed_query.get_single_mut() {
                        *pushed_tf = *button_tf;
                        pushed_tf.translation.z += 0.1;
                        *vis = Visibility::Inherited;
                    }
                }
            }
        }
    }
}