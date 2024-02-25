mod parse_bds;
mod regex;

use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    reflect::{
        serde::{ReflectSerializer, UntypedReflectDeserializer},
        TypePath,
    },
    utils::{thiserror, BoxedFuture},
};
use parse_bds::*;
use serde::{de::DeserializeSeed, Deserialize};
use thiserror::Error;

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

    pub fn get_opt<T: Default + Reflect + TypePath>(&self) -> Option<T> {
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

#[derive(Asset, Debug, Deserialize, TypePath)]
pub struct BMWScript {
    pub script: String,
}

#[derive(Default)]
pub struct BMWScriptLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum BMWScriptLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    /// A [String](std::string) Error
    #[error("Could not read utf8: {0}")]
    ReadingStringError(#[from] std::string::FromUtf8Error),
}

impl AssetLoader for BMWScriptLoader {
    type Asset = BMWScript;
    type Settings = ();
    type Error = BMWScriptLoaderError;
    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let raw_text = String::from_utf8(bytes)?;
            let bds = BMWScript { script: raw_text };
            Ok(bds)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["md", "bds"]
    }
}

#[derive(Asset, Debug, Deserialize, TypePath)]
pub struct BMWTemplate {
    pub template: String,
}

#[derive(Default)]
pub struct BMWTemplateLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum BMWTemplateLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    /// A [String](std::string) Error
    #[error("Could not read utf8: {0}")]
    ReadingStringError(#[from] std::string::FromUtf8Error),
}

impl AssetLoader for BMWTemplateLoader {
    type Asset = BMWTemplate;
    type Settings = ();
    type Error = BMWTemplateLoaderError;
    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let raw_text = String::from_utf8(bytes)?;
            let bdt = BMWTemplate { template: raw_text };
            Ok(bdt)
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
                let parsed =
                    parse_script(&bds.script, &bdt.template, &loaded_script.target_section);
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

pub fn split_path_and_section<S: AsRef<str>>(uri: S) -> (String, String) {
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
