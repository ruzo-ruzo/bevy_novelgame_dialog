mod parse_bds;
mod regex;

use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::{
        serde::{ReflectSerializer, UntypedReflectDeserializer},
        TypePath, TypeUuid,
    },
    utils::BoxedFuture,
};
use parse_bds::*;
use serde::{de::DeserializeSeed, Deserialize};

#[derive(Event)]
pub struct BdsEvent {
    pub value: Box<dyn Reflect>,
}

impl BdsEvent {
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
    pub bds_handle: Handle<BMWScript>,
    pub bdt_handle: Handle<BMWTemplate>,
    pub target_section: String,
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
            let bds = BMWScript { script: raw_text };
            load_context.set_default_asset(LoadedAsset::new(bds));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["md", "bds"]
    }
}

#[derive(Debug, Deserialize, TypeUuid, TypePath)]
#[uuid = "82967a68-951e-3f8d-c4ce-8143f7180d33"]
pub struct BMWTemplate {
    pub template: String,
}

#[derive(Default)]
pub struct BMWTemplateLoader;

impl AssetLoader for BMWTemplateLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let raw_text = String::from_utf8(bytes.to_vec())?;
            let bdt = BMWTemplate { template: raw_text };
            load_context.set_default_asset(LoadedAsset::new(bdt));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["csv", "bdt"]
    }
}

pub fn script_on_load(
    mut loaded_script_query: Query<&mut LoadedScript>,
    script_assets: Res<Assets<BMWScript>>,
    template_assets: Res<Assets<BMWTemplate>>,
) {
    for mut loaded_script in &mut loaded_script_query {
        if loaded_script.order_list.is_none() {
            let script_opt = script_assets.get(&loaded_script.bds_handle);
            let template_opt = template_assets.get(&loaded_script.bdt_handle);
            if let (Some(bds), Some(bdt)) = (script_opt, template_opt) {
                // info!("script is {}, \r\n section is {}", bds.script, loaded_script.target_section);
                let parsed = parse_script(&bds.script, &bdt.template, &loaded_script.target_section);
                loaded_script.order_list = Some(parsed);
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

pub fn split_path_and_section<S: AsRef<str>>(uri: S) -> (String, String){
    parse_uri(uri.as_ref())
}

pub fn write_ron<R: Reflect>(
    type_registry: &AppTypeRegistry,
    value: R,
) -> Result<String, ron::Error> {
    let type_registry = type_registry.read();
    let serializer = ReflectSerializer::new(&value, &type_registry);
    ron::ser::to_string_pretty(&serializer, ron::ser::PrettyConfig::default())
}

pub fn parse_script<S1: AsRef<str>, S2: AsRef<str>, S3: AsRef<str>>(
    base: S1,
    template: S2,
    section: S3,
) -> Vec<Order> {
    let orders = read_script(base, template);
    // info!("{orders:?}");
    orders[section.as_ref()].clone().into_iter().rev().collect()
}
