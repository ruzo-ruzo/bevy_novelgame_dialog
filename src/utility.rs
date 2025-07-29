use rustybuzz::{shape, Face, UnicodeBuffer, GlyphBuffer};
use bevy::prelude::*;
use rand::{distributions::uniform::SampleRange, Rng};
use regex::Regex;
use std::collections::HashMap;

#[allow(dead_code)]
pub(crate) fn get_random<T, R: AsRef<[T]>>(list: &R) -> Option<&T> {
    let list_ref: &[T] = list.as_ref();
    list_ref.get(rand::thread_rng().gen_range(0..list_ref.len()))
}

pub(crate) fn find_by_regex<T: Clone, S: AsRef<str>>(s: S, base: &HashMap<String, T>) -> Option<T> {
    let reg = base.keys().find_map(|k| {
        let r = Regex::new(k).ok()?;
        if r.is_match(s.as_ref()) {
            Some(k)
        } else {
            None
        }
    });
    reg.and_then(|r| base.get(r)).cloned()
}

pub(crate) fn choice_font<R: AsRef<[Handle<Font>]>>(
    list: &R,
    target: char,
    fonts: &Assets<Font>,
) -> Option<Handle<Font>> {
    let finded = list
        .as_ref()
        .iter()
        .find(|h| {
            fonts
                .get(*h)
                .map(|f| glyph_exists_in_font(f.clone(), target))
                .unwrap_or(false)
        })
        .cloned();
    finded.or(list.as_ref().iter().last().cloned())
}

#[allow(dead_code)]
pub(crate) fn choice_font_with_index<R: AsRef<[Handle<Font>]>>(
    list: &R,
    target: char,
    fonts: &Assets<Font>,
) -> Option<(usize, Handle<Font>)> {
    let finded = list.as_ref().iter().enumerate().find(|(_, h)| {
        fonts
            .get(*h)
            .map(|f| glyph_exists_in_font(f.clone(), target))
            .unwrap_or(false)
    });
    finded.map(|(i, f)| (i, f.clone())).or(list
        .as_ref()
        .iter()
        .map(|x| (list.as_ref().iter().len() - 1, x.clone()))
        .last())
}

fn glyph_exists_in_font(font: Font, target: char) -> bool {
    let Some(buffer) = get_glyph_buffer(&font, target) else { return false };
    buffer.glyph_infos().iter().next().is_some_and(|x|x.glyph_id != 0)
}

pub(crate) fn get_glyph_buffer(font: &Font, target: char) -> Option<GlyphBuffer> {
    let face = Face::from_slice(&font.data, 0)?;
    let mut code = UnicodeBuffer::new();
    code.push_str(&target.to_string());
    Some(shape(&face, &[], code))
}

#[allow(dead_code)]
pub(crate) fn random_char() -> Option<char> {
    fn range_to_char<R: SampleRange<u32>>(range: R) -> Option<char> {
        std::char::from_u32(rand::thread_rng().gen_range(range))
    }

    let emoji = range_to_char(0x1F300..0x1F5FF)?;
    let kanji = range_to_char(0x4E00..0x9FFF)?;
    let hiragana = range_to_char(0x3040..0x309F)?;
    let alphabet_large = range_to_char(0x41..0x5A)?;
    let alphabet_small = range_to_char(0x61..0x7A)?;
    let mixed = &[emoji, kanji, hiragana, alphabet_large, alphabet_small];
    get_random(mixed).copied()
}
