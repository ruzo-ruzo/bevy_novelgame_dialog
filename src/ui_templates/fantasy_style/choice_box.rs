use super::*;
use bevy::render::view::RenderLayers;

pub(super) struct ChoiceBoxPlugIn;

impl Plugin for ChoiceBoxPlugIn {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "assets/textures/ui/choice_buttons/button_default.png");
        embedded_asset!(app, "assets/textures/ui/choice_buttons/button_pushed.png");
        embedded_asset!(app, "assets/textures/ui/choice_buttons/choicing_frame.png");
        app.add_systems(Startup, setup_choice_images)
            .add_systems(Update, move_cursor)
            .add_systems(Update, reset_images)
            .add_systems(Update, button_clicked);
    }
}

#[derive(Component)]
struct ChoiceCursor;

#[derive(Component)]
struct PushedButton;

fn setup_choice_images(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<TemplateSetupConfig>,
) {
    let button_image_path = "textures/ui/choice_buttons/button_default.png";
    let pushed_image_path = "textures/ui/choice_buttons/button_pushed.png";
    let frame_image_path = "textures/ui/choice_buttons/choicing_frame.png";
    let box_image_path = "textures/ui/dialog_box_01.png";
    let button_image_handle = asset_server.load(ASSETS_PATH.to_owned() + button_image_path);
    let pushed_image_handle = asset_server.load(ASSETS_PATH.to_owned() + pushed_image_path);
    let choicing_frame_image_handle = asset_server.load(ASSETS_PATH.to_owned() + frame_image_path);
    let dialog_box_image_handle = asset_server.load(ASSETS_PATH.to_owned() + box_image_path);
    let button_slice = ImageScaleMode::Sliced(TextureSlicer {
        border: BorderRect::square(30.0),
        ..default()
    });
    let choicing_frame_slice = ImageScaleMode::Sliced(TextureSlicer {
        border: BorderRect::rectangle(56.0, 102.0),
        ..default()
    });
    let dialog_box_slice = ImageScaleMode::Sliced(TextureSlicer {
        border: BorderRect::rectangle(44.0, 52.0),
        ..default()
    });
    for i in 0..config.max_button_index {
        let button_height = -70.0 - ((config.button_size.y + 40.0)*(i as f32));
        let button_sprite_bundle = SpriteBundle {
            sprite: Sprite {
                custom_size: Some(config.button_size),
                ..default()
            },
            texture: button_image_handle.clone(),
            transform: Transform::from_xyz(0.0, button_height , 0.6),
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
            custom_size: Some(Vec2::new(config.button_size.x + 200.0, config.button_size.y)),
            ..default()
        },
        texture: dialog_box_image_handle,
        transform: Transform::from_xyz(0.0, 0.0, 1.1),
        ..default()
    };
    let pushed_sprite_bundle = SpriteBundle {
        sprite: Sprite {
            custom_size: Some(config.button_size),
            ..default()
        },
        texture: pushed_image_handle,
        transform: Transform::from_xyz(0.0, -70.0, 0.7),
        visibility: Visibility::Hidden,
        ..default()
    };
    let cursor_size = Vec2::new(config.button_size.x + 80.0, config.button_size.y + 100.0);
    let cursor_sprite_bundle = SpriteBundle {
        sprite: Sprite {
            custom_size: Some(cursor_size),
            ..default()
        },
        texture: choicing_frame_image_handle,
        transform: Transform::from_xyz(-2.0, cursor_size.y, 0.3),
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
