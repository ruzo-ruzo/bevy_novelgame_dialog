use ab_glyph::Font as AFont;
use bevy::prelude::*;
use rand::{distributions::uniform::SampleRange, Rng};

#[allow(dead_code)]
pub fn get_random<T, R: AsRef<[T]>>(list: &R) -> Option<&T> {
    let list_ref: &[T] = list.as_ref();
    list_ref.get(rand::thread_rng().gen_range(0..list_ref.len()))
}

pub fn choice_font<R: AsRef<[Handle<Font>]>>(
    list: &R,
    target: char,
    fonts: &Assets<Font>,
) -> Option<Handle<Font>> {
    let finded = list
        .as_ref()
        .iter()
        .find(|h| {
            fonts
                .get(h)
                .map(|f| glyph_exists_in_font(f.clone(), target))
                .unwrap_or(false)
        })
        .cloned();
    finded.or(list.as_ref().iter().last().cloned())
}

fn glyph_exists_in_font(font: Font, target: char) -> bool {
    let font_id = font.font.glyph_id(target);
    let outline = font.font.outline(font_id);
    let raster = font.font.glyph_raster_image(font_id, 1);
    outline.is_some() || raster.is_some()
}

#[allow(dead_code)]
pub fn random_char() -> Option<char> {
    fn range_to_char<R: SampleRange<u32>>(range: R) -> Option<char> {
        std::char::from_u32(rand::thread_rng().gen_range(range))
    }

    let _emoji = range_to_char(0x1F300..0x1F5FF)?;
    let _kanji = range_to_char(0x4E00..0x9FFF)?;
    let hiragana = range_to_char(0x3040..0x309F)?;
    let alphabet_large = range_to_char(0x41..0x5A)?;
    let alphabet_small = range_to_char(0x61..0x7A)?;
    let mixed = &[_kanji, hiragana, alphabet_large, alphabet_small];
    get_random(mixed).copied()
}
