use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::{serde::UntypedReflectDeserializer, TypePath, TypeUuid},
    utils::BoxedFuture,
};
use serde::{de::DeserializeSeed, Deserialize};

#[derive(Event)]
pub struct BMSEvent {
    pub value: Box<dyn Reflect>,
}

impl BMSEvent {
    pub fn get<T: Default + Reflect>(&self) -> T {
        let mut my_data = <T>::default();
        my_data.apply(&*self.value);
        my_data
    }

    pub fn get_opt<T: Default + Reflect>(&self) -> Option<T> {
        if self.value.represents::<T>() {
            Some(self.get::<T>())
        } else {
            None
        }
    }
}

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

pub fn read_ron<S: AsRef<str>>(
    type_registry: &AppTypeRegistry,
    ron: S,
) -> Result<Box<dyn Reflect>, ron::Error> {
    let ron_string = ron.as_ref().to_string();
    let reg = type_registry.read();
    let reflect_deserializer = UntypedReflectDeserializer::new(&reg);
    let mut deserializer = ron::de::Deserializer::from_str(&ron_string)?;
    reflect_deserializer.deserialize(&mut deserializer)
}

//-- 以下は仮設定
pub fn perse_script(base: String) -> Vec<Order> {
    base.chars()
        .map(|c| match c {
            '\t' => Order::PageFeed,
            '\n' => Order::CarriageReturn,
            '-' => Order::ThroghEvent { ron: r#"{
    "bevy_message_window::message_window::bms_event::FontSizeChange": (
        size: 27.0,
)}"#.to_string()},
            '+' => Order::ThroghEvent { ron: r#"{
    "bevy_message_window::message_window::bms_event::FontSizeChange": (
        size: 40.0,
),}}"#.to_string()},
            _ => Order::Type { character: c },
        })
        .rev()
        .collect()
}
