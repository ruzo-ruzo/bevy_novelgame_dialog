use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::{TypePath, TypeUuid},
    utils::BoxedFuture,
};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq)]
pub enum Order {
    Type { character: char },
    CarriageReturn,
    PageFeed,
    ThroghEvent { ron: String },
}

#[derive(Component, Debug)]
pub struct LoadedScript {
    pub bms_handle: Handle<BMWScript>,
    pub order_list: Option<Vec<Order>>,
}

#[derive(Debug, Deserialize, TypeUuid, TypePath)]
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

pub fn perse_script(base: String) -> Vec<Order> {
    base.chars()
        .map(|c| match c {
            '\t' => Order::PageFeed,
            '\n' => Order::CarriageReturn,
            _ => Order::Type { character: c },
        })
        .rev()
        .collect()
}

pub fn script_on_load(
    mut loaded_script_query: Query<&mut LoadedScript>,
    script_assets: Res<Assets<BMWScript>>,
) {
    for mut loaded_script in &mut loaded_script_query {
        if loaded_script.order_list.is_none() {
            let script_opt = script_assets.get(&loaded_script.bms_handle);
            if let Some(bms) = script_opt {
                loaded_script.order_list = Some(perse_script(bms.script.clone()));
            }
        }
    }
}
