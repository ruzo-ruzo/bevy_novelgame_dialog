use nom::branch::*;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::*;
use nom::multi::*;
use nom::sequence::*;
use nom::*;
use std::collections::HashMap;

use super::Order;

#[derive(Clone, Debug, PartialEq)]
enum ParsedOrder {
    OrderWrapper(Order),
    SectionLine(String),
    Empty,
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
                section_map.insert(next_head, next_list);
                next_head = s;
                next_list = vec![]
            }
            ParsedOrder::OrderWrapper(o) => next_list.push(o),
            ParsedOrder::Empty => (),
        }
    }
    section_map.insert(next_head, next_list);
    section_map
}

fn parse_bms(input: &str) -> Vec<ParsedOrder> {
    let mut bms_parser = many0(alt((section_head, next_line, erase_useless_tag, simple_char)));
    if let Ok((_, parsed_order_list)) = bms_parser(input) {
        parsed_order_list.into_iter().filter(|x|*x != ParsedOrder::Empty).collect()
    } else {
        vec![]
    }
}

fn erase_useless_tag(input: &str) -> IResult<&str, ParsedOrder> {
    let mut useless_tag = tuple((is_not("\\"), tag("<"), end_tag_include_slash));
    println!("{:?}", useless_tag(input));
    value(ParsedOrder::Empty, useless_tag)(input)
}

fn next_line(input: &str) -> IResult<&str, ParsedOrder> {
    let cr = ParsedOrder::OrderWrapper(Order::CarriageReturn);
    let br_tag = value(cr.clone(), pair(tag("<br"), end_tag_include_slash));
    let more_then_2_spaces = pair(one_of(" \t"), space1);
    let space_end = value(cr, pair(more_then_2_spaces, line_ending));
    alt((br_tag, space_end))(input)
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
    let complex_end_tag = value(
        true,
        tuple((space1, many0(none_of("/>")), alt((tag("/>"), tag(">"))))),
    );
    alt((value(true, tag(">")), complex_end_tag))(input)
}

fn end_tag(input: &str) -> IResult<&str, bool> {
    let complex_end_tag = value(true, tuple((space1, many0(not(tag(">"))), tag(">"))));
    alt((value(true, tag(">")), complex_end_tag))(input)
}

fn section_head(input: &str) -> IResult<&str, ParsedOrder> {
    let h1_open = pair(tag("<h1"), end_tag);
    let h1_close = "</h1>";
    let h1_taged = delimited(h1_open, take_until(h1_close), tag(h1_close));
    let h1 = map(h1_taged, |s| ParsedOrder::SectionLine(s.to_string()));
    let sharp_head = preceded(
        tuple((line_ending, char('#'), space1)),
        many_till(take(1usize), line_ending),
    );
    let sharp = map(sharp_head, |(v, _)| ParsedOrder::SectionLine(v.concat()));
    let under_line = tuple((line_ending, char('='), many1(char('=')), line_ending));
    let under_lined = preceded(line_ending, many_till(take(1usize), under_line));
    let lined = map(under_lined, |(v, _)| ParsedOrder::SectionLine(v.concat()));
    alt((h1, sharp, lined))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    const HELLO: &[Order] = &[
        Order::Type { character: 'こ' },
        Order::Type { character: 'ん' },
        Order::Type { character: 'に' },
        Order::Type { character: 'ち' },
        Order::Type { character: 'は' },
        Order::CarriageReturn,
        Order::Type { character: 'は' },
        Order::Type { character: 'じ' },
        Order::Type { character: 'め' },
        Order::Type { character: 'ま' },
        Order::Type { character: 'し' },
        Order::Type { character: 'て' },
    ];

    const ILL: &[Order] = &[
        Order::Type { character: 'こ' },
        Order::Type { character: 'の' },
        Order::Type { character: '家' },
        Order::Type { character: 'の' },
        Order::Type { character: '主' },
        Order::Type { character: '人' },
        Order::Type { character: 'は' },
        Order::Type { character: '病' },
        Order::Type { character: '気' },
        Order::Type { character: 'で' },
        Order::Type { character: 'す' },
    ];

    #[test]
    fn test_hello_br() {
        let hello_po = HELLO
            .iter()
            .map(|o| ParsedOrder::OrderWrapper(o.clone()))
            .collect::<Vec<_>>();
        assert_eq!(parse_bms("こんにちは<br />はじめまして"), hello_po);
    }

    #[test]
    fn test_hello_double_space_end() {
        let hello_vec = HELLO.into();
        assert_eq!(
            read_bms("こんにちは  \r\nはじめまして"),
            HashMap::from([("".to_string(), hello_vec)])
        );
    }

    #[test]
    fn test_h1() {
        let sectioned_phrase = HashMap::from([
            ("".to_string(), HELLO.into()),
            ("二つ目".to_string(), ILL.into()),
        ]);
        assert_eq!(
            read_bms("こんにちは<br>はじめまして<h1>二つ目</h1>この家の主人は病気です"),
            sectioned_phrase
        );
    }

    #[test]
    fn test_under_line() {
        let sectioned_phrase = HashMap::from([
            ("".to_string(), HELLO.into()),
            ("二つ目".to_string(), ILL.into()),
        ]);
        let read =
            read_bms("こんにちは<br css='';/>はじめまして\n二つ目\n======\nこの家の主人は病気です");
        assert_eq!(read, sectioned_phrase);
    }

    #[test]
    fn test_sharp_head() {
        let sectioned_phrase = HashMap::from([
            ("".to_string(), HELLO.into()),
            ("二つ目".to_string(), ILL.into()),
        ]);
        let read = read_bms("こんにちは    \r\nはじめまして\r\n# 二つ目\r\nこの家の主人は病気です");
        assert_eq!(read, sectioned_phrase);
    }
    
    #[test]
    fn test_useless_tag(){
        let useless_taged = vec![
            ParsedOrder::OrderWrapper(Order::Type { character: 'a' }),
            ParsedOrder::OrderWrapper(Order::Type { character: 'a' }),
            ParsedOrder::OrderWrapper(Order::Type { character: 'b' }),
            ParsedOrder::OrderWrapper(Order::Type { character: 'c' }),
            ParsedOrder::OrderWrapper(Order::Type { character: 'd' }),
            ParsedOrder::OrderWrapper(Order::Type { character: '\\' }),
            ParsedOrder::OrderWrapper(Order::Type { character: '<' }),
            ParsedOrder::OrderWrapper(Order::Type { character: 'a' }),
            ParsedOrder::OrderWrapper(Order::Type { character: 'b' }),
            ParsedOrder::OrderWrapper(Order::Type { character: '>' }),
        ];
        assert_eq!(parse_bms("a<abc>abcd\\<ab>"), useless_taged);
    }
}
