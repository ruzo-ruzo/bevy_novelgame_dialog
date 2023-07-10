use nom::branch::*;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::*;
use nom::multi::*;
use nom::sequence::*;
use nom::*;
use std::collections::HashMap;

use super::Order;

#[derive(Clone)]
enum ParsedOrder {
    OrderWrapper(Order),
    SectionLine(String),
}

pub fn read_script() -> HashMap<String, Vec<Order>> {
    todo!()
}

fn read_bms<S: AsRef<str>>(input: S) -> HashMap<String, Vec<Order>> {
    let mut section_map = HashMap::new();
    let mut next_head = "".to_string();
    let mut next_list = vec![];
    for p in parse_bms(input.as_ref()) {
        match p {
            ParsedOrder::SectionLine(s) => {
                section_map.insert(s, next_list);
                next_list = vec![]
            }
            ParsedOrder::OrderWrapper(o) => next_list.push(o)
        }
    }
    section_map
}

fn parse_bms(input: &str) -> Vec<ParsedOrder> {
    let mut bms_parser = many0(alt((section_head, next_line, simple_char)));
    if let Ok((_, parsed_order_list)) = bms_parser(input) {
        parsed_order_list
    } else {
        vec![]
    }
}

fn next_line(input: &str) -> IResult<&str, ParsedOrder> {
    let cr = ParsedOrder::OrderWrapper(Order::CarriageReturn);
    let br_tag = value(cr.clone(), pair(tag("<br"), end_tag_include_slash));
    let more_then_2_spaces = pair(space1, space1);
    let space_end = value(cr, pair(more_then_2_spaces, line_ending));
    alt((space_end, br_tag))(input)
}

fn simple_char(input: &str) -> IResult<&str, ParsedOrder> {
    take(1usize)(input).map(|(rem, c)| {
        (
            rem,
            ParsedOrder::OrderWrapper(Order::Type {
                character: c.chars().next().unwrap(),
            }),
        )
    })
}

fn end_tag_include_slash(input: &str) -> IResult<&str, bool> {
    let end_tag = pair(many0(not(is_a("/>"))), alt((tag(">"), tag("/>"))));
    value(true, end_tag)(input)
}

fn end_tag(input: &str) -> IResult<&str, bool> {
    value(true, pair(many0(not(char('>'))), char('>')))(input)
}

fn section_head(input: &str) -> IResult<&str, ParsedOrder> {
    let h1_open = pair(tag("<h1"), end_tag);
    let h1_close = "</h1>";
    let h1_taged = delimited(h1_open, take_until(h1_close), tag(h1_close));
    let h1 = map(h1_taged, |s| ParsedOrder::SectionLine(s.to_string()));
    let sharp_head = delimited(pair(char('#'), space1), many0(take(1usize)), line_ending);
    let sharp = map(sharp_head, |v| ParsedOrder::SectionLine(v.join("")));
    let under_line = tuple((line_ending, char('='), many1(char('='))));
    let under_lined = terminated(many0(take(1usize)), under_line);
    let lined = map(under_lined, |v|ParsedOrder::SectionLine(v.join("")));
    alt((h1, sharp, lined))(input)
}

#[test]
fn parser_test(){
    assert_eq!(read_bms("こんにちは"), HashMap::from([("".to_string(), vec![
        Order::Type{character: 'こ'},
        Order::Type{character: 'ん'},
        Order::Type{character: 'に'},
        Order::Type{character: 'ち'},
        Order::Type{character: 'は'},
    ])]));
}
