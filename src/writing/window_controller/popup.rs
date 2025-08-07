use super::*;
use crate::read_script::split_path_and_section;
use bevy::render::view::{RenderLayers, Visibility::*};

pub(in crate::writing) fn open_window(
    mut commands: Commands,
    bg_query: Query<(Entity, &DialogBoxBackground)>,
    db_query: Query<Entity, (With<Current>, With<DialogBox>)>,
    mut tf_query: Query<&mut Transform>,
    mut ow_event: EventReader<OpenDialog>,
    asset_server: Res<AssetServer>,
    setup_config: Res<SetupConfig>,
) {
    for window_config in &mut ow_event.read() {
        db_query.iter().for_each(|e| {
            commands.entity(e).remove::<Current>();
        });
        let (script_path, script_section) =
            split_path_and_section(window_config.script_path.clone());
        let loaded_script = if window_config.raw_orders.is_some() {
            LoadedScript {
                bds_handle_opt: None,
                bdt_handle_list: Vec::new(),
                target_section: script_section,
                order_list: window_config.raw_orders.clone(),
            }
        } else {
            LoadedScript {
                bds_handle_opt: Some(asset_server.load(script_path)),
                bdt_handle_list: window_config
                    .template_path
                    .iter()
                    .map(|x| asset_server.load(x.clone()))
                    .collect(),
                target_section: script_section,
                order_list: None,
            }
        };
        let mwb = DialogBoxBundle {
            writing: DialogBox {
                name: window_config.writing_name.clone(),
            },
            state: DialogBoxPhase::Preparing,
            waitting: window_config.wait_breaker.clone(),
            script: loaded_script,
            popup_type: window_config.popup,
        };
        let mw_spirte = (
            Sprite::default(),
            Transform::from_translation(window_config.position.extend(0.0)),
        );
        let bg_opt = bg_query
            .iter()
            .find(|x| x.1.writing_name == window_config.writing_name);
        let mw = match bg_opt {
            Some((entity, _)) => entity,
            None => commands.spawn((mw_spirte, Instant)).id(),
        };
        if let Ok(mut tf) = tf_query.get_mut(mw) {
            tf.scale = Vec3::ONE;
        }
        let additional_mw = (Hidden, window_config.template_open_choice.clone());
        let layer = RenderLayers::layer(setup_config.render_layer.into());
        commands
            .entity(mw)
            .insert((mwb, layer.clone(), Current, additional_mw));
        let mut ta_id_list = Vec::new();
        let mut current_exists_in_text_areas = false;
        for t_cfg in &window_config.text_area_configs {
            let tab = TextAreaBundle {
                text_area: TextArea {
                    name: t_cfg.area_name.clone(),
                },
                feeding: t_cfg.feeding,
                config: initialize_text_config(
                    asset_server.clone(),
                    t_cfg,
                    Res::clone(&setup_config),
                ),
            };
            let ta_sprite = (
                Sprite {
                    anchor: Anchor::TopLeft,
                    color: Color::WHITE.with_alpha(0.0),
                    // color: Color::BLACK.with_a(0.5),
                    custom_size: Some(t_cfg.area_size),
                    ..default()
                },
                Transform::from_translation(t_cfg.area_origin.extend(0.0)),
                // transform: Transform::from_translation(t_cfg.area_origin.extend(10.0)),
            );
            let tai = commands.spawn((tab, ta_sprite, layer.clone())).id();
            commands.entity(mw).add_child(tai);
            if t_cfg.area_name == window_config.main_text_area_name {
                commands.entity(tai).insert(Current);
                current_exists_in_text_areas = true;
            }
            ta_id_list.push(tai);
        }
        if !current_exists_in_text_areas {
            if let Some(id) = ta_id_list.first() {
                commands.entity(*id).insert(Current);
            }
        }
    }
}

fn initialize_text_config(
    asset_server: AssetServer,
    t_cfg: &TextAreaConfig,
    setup_config: Res<SetupConfig>,
) -> TypeTextConfig {
    let text_font_vec = t_cfg
        .text_config
        .font_settings
        .iter()
        .map(|f| TextFont {
            font: asset_server.load(f.path.clone()),
            font_size: f.size_coefficient,
            ..default()
        })
        .collect::<Vec<_>>();
    TypeTextConfig {
        text_fonts: text_font_vec,
        kerning_by_regulars: t_cfg.text_config.kerning_by_regulars.clone(),
        size_by_regulars: t_cfg.text_config.size_by_regulars.clone(),
        text_color: t_cfg.text_config.font_color,
        writing: t_cfg.writing,
        base_size: t_cfg.text_config.text_base_size,
        typing_timing: t_cfg.typing_timing,
        layer: RenderLayers::layer(setup_config.render_layer.into()),
        horizon_alignment: t_cfg.horizon_alignment,
        vertical_alignment: t_cfg.vertical_alignment,
        monospace: t_cfg.monospace,
        pos_z: t_cfg.text_pos_z,
    }
}

#[derive(Component)]
pub(in crate::writing) struct ScalingUp {
    pub add_per_sec: f32,
}

#[allow(clippy::type_complexity)]
pub(in crate::writing) fn window_popper(
    mut commands: Commands,
    mut db_query: Query<
        (
            Entity,
            &mut DialogBoxPhase,
            &PopupType,
            &mut Visibility,
            &mut Transform,
        ),
        With<DialogBox>,
    >,
) {
    for (ent, mut ws, pt, mut vis, mut tf) in &mut db_query {
        if *ws == DialogBoxPhase::Preparing {
            match pt {
                PopupType::Scale { sec: s } => {
                    tf.scale = Vec3::new(0., 0., 0.);
                    commands.entity(ent).insert(ScalingUp {
                        add_per_sec: 1.0 / s,
                    });
                }
            }
            *vis = Visible;
            *ws = DialogBoxPhase::PoppingUp;
        }
    }
}

pub(in crate::writing) fn scaling_up(
    mut commands: Commands,
    mut db_query: Query<(Entity, &mut Transform, &ScalingUp, &mut DialogBoxPhase)>,
    time: Res<Time>,
) {
    for (ent, mut tf, ScalingUp { add_per_sec: aps }, mut ws) in &mut db_query {
        if *ws == DialogBoxPhase::PoppingUp {
            if tf.scale.x >= 1.0 {
                tf.scale = Vec3::new(1., 1., 1.);
                *ws = DialogBoxPhase::Typing;
                commands.entity(ent).remove::<ScalingUp>();
            } else {
                tf.scale.x += time.delta_secs() * aps;
                tf.scale.y += time.delta_secs() * aps;
            }
        }
    }
}
