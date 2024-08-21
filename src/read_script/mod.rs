mod parse_bds;
mod regex;

use crate::prelude::Order;
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    reflect::{
        serde::{ReflectSerializer, ReflectDeserializer},
        TypePath,
    },
};
use parse_bds::*;
use serde::{de::DeserializeSeed, Deserialize};
use thiserror::Error;

#[derive(Component, Debug)]
pub(crate) struct LoadedScript {
    pub bds_handle_opt: Option<Handle<BMWScript>>,
    pub bdt_handle_list: Vec<Handle<BMWTemplate>>,
    pub target_section: String,
    pub order_list: Option<Vec<Order>>,
}

#[derive(Asset, Debug, Deserialize, TypePath)]
pub(crate) struct BMWScript {
    pub script: String,
}

#[derive(Default)]
pub(crate) struct BMWScriptLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub(crate) enum BMWScriptLoaderError {
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
    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let raw_text = String::from_utf8(bytes)?;
            let bds = BMWScript { script: raw_text };
            Ok(bds)
    }

    fn extensions(&self) -> &[&str] {
        &["md", "bds"]
    }
}

#[derive(Asset, Debug, Deserialize, TypePath)]
pub(crate) struct BMWTemplate {
    pub template: String,
}

#[derive(Default)]
pub(crate) struct BMWTemplateLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub(crate) enum BMWTemplateLoaderError {
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
    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let raw_text = String::from_utf8(bytes)?;
            let bdt = BMWTemplate { template: raw_text };
            Ok(bdt)
    }

    fn extensions(&self) -> &[&str] {
        &["csv", "bdt"]
    }
}

pub(crate) fn script_on_load(
    mut loaded_script_query: Query<&mut LoadedScript>,
    script_assets: Res<Assets<BMWScript>>,
    template_assets: Res<Assets<BMWTemplate>>,
) {
    for mut loaded_script in &mut loaded_script_query {
        if loaded_script.order_list.is_none() {
            let script_opt = loaded_script
                .bds_handle_opt
                .clone()
                .and_then(|x| script_assets.get(&x));
            let template_list = loaded_script
                .bdt_handle_list
                .iter()
                .filter_map(|x| template_assets.get(x))
                .map(|x| x.template.clone())
                .collect::<Vec<_>>();
            if let Some(bds) = script_opt {
                let parsed =
                    parse_script(&bds.script, &template_list, &loaded_script.target_section);
                loaded_script.order_list = Some(parsed);
            }
        }
    }
}

pub(crate) fn read_ron<S: AsRef<str>>(
    type_registry: &AppTypeRegistry,
    ron: S,
) -> Result<Box<dyn Reflect>, ron::Error> {
    let ron_string = ron.as_ref().to_string();
    let reg = type_registry.read();
    let reflect_deserializer = ReflectDeserializer::new(&reg);
    let mut deserializer = ron::de::Deserializer::from_str(&ron_string)?;
    reflect_deserializer.deserialize(&mut deserializer)
}

pub(crate) fn split_path_and_section<S: AsRef<str>>(uri: S) -> (String, String) {
    parse_uri(uri.as_ref())
}

pub(crate) fn write_ron<R: Reflect>(
    type_registry: &AppTypeRegistry,
    value: R,
) -> Result<String, ron::Error> {
    let type_registry = type_registry.read();
    let serializer = ReflectSerializer::new(&value, &type_registry);
    ron::ser::to_string_pretty(&serializer, ron::ser::PrettyConfig::default())
}

pub(crate) fn parse_script<S1: AsRef<str>, S2: AsRef<str>, S3: AsRef<str>>(
    base: S1,
    templates: &[S2],
    section: S3,
) -> Vec<Order> {
    let orders = read_script(base, templates);
    orders[section.as_ref()].clone().into_iter().rev().collect()
}
