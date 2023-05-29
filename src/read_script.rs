use crate::message_window::*;
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use serde::Deserialize;

#[derive(Debug, Deserialize, TypeUuid)]
#[uuid = "edb6ad8f-ca38-189e-9dce-ae1fb5031888"]
pub struct BMWScript {
    pub script: String,
}

#[derive(Default)]
pub struct BMWScriptLoader;

impl AssetLoader for BMWScriptLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let raw_text = String::from_utf8(bytes.to_vec())?;
            let bms = BMWScript { script: raw_text };
            load_context.set_default_asset(LoadedAsset::new(bms));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["bms"]
    }
}

pub fn script_on_load(mut loaded_text: ResMut<LoadedText>, script_assets: Res<Assets<BMWScript>>) {
    let script_opt = script_assets.get(&loaded_text.base_bms);
    if loaded_text.loading && script_opt.is_some() {
        loaded_text.char_list = script_opt.unwrap().script.chars().rev().collect::<String>();
        loaded_text.loading = false;
    }
}
