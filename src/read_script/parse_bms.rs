use nom::branch::*;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::*;
use nom::multi::*;
use nom::sequence::*;
use nom::*;
use std::collections::HashMap;

use super::regex::replace_by_template;
use super::Order;

#[derive(Clone, Debug, PartialEq)]
enum ParsedOrder {
    OrderWrapper(Order),
    SectionLine(String),
    Empty,
}

pub fn read_script<S1: AsRef<str>, S2: AsRef<str>>(
    input: S1,
    template: S2,
) -> HashMap<String, Vec<Order>> {
    let replaced = replace_by_template(input, template);
    read_bms(replaced)
}

pub fn read_bms<S: AsRef<str>>(input: S) -> HashMap<String, Vec<Order>> {
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
    let mut bms_parser = many0(alt((
        backslash,
        ampersand,
        section_head,
        next_paragraph,
        throw_event,
        next_line,
        erase_useless_tag,
        simple_char,
    )));
    if let Ok((_, parsed_order_list)) = bms_parser(input) {
        parsed_order_list
            .into_iter()
            .filter(|x| *x != ParsedOrder::Empty)
            .collect()
    } else {
        vec![]
    }
}

fn backslash(input: &str) -> IResult<&str, ParsedOrder> {
    preceded(
        char('\\'),
        alt((
            value(
                ParsedOrder::OrderWrapper(Order::Type { character: '\\' }),
                char('\\'),
            ),
            value(
                ParsedOrder::OrderWrapper(Order::Type { character: '<' }),
                char('<'),
            ),
            value(
                ParsedOrder::OrderWrapper(Order::Type { character: '>' }),
                char('>'),
            ),
            value(
                ParsedOrder::OrderWrapper(Order::Type { character: '`' }),
                char('`'),
            ),
            value(
                ParsedOrder::OrderWrapper(Order::Type { character: '{' }),
                char('{'),
            ),
            value(
                ParsedOrder::OrderWrapper(Order::Type { character: '}' }),
                char('}'),
            ),
            value(
                ParsedOrder::OrderWrapper(Order::Type { character: '[' }),
                char('['),
            ),
            value(
                ParsedOrder::OrderWrapper(Order::Type { character: ']' }),
                char(']'),
            ),
            value(
                ParsedOrder::OrderWrapper(Order::Type { character: '_' }),
                char('_'),
            ),
            value(
                ParsedOrder::OrderWrapper(Order::Type { character: '*' }),
                char('*'),
            ),
            value(
                ParsedOrder::OrderWrapper(Order::Type { character: '+' }),
                char('+'),
            ),
            value(
                ParsedOrder::OrderWrapper(Order::Type { character: '(' }),
                char('('),
            ),
            value(
                ParsedOrder::OrderWrapper(Order::Type { character: ')' }),
                char(')'),
            ),
            value(
                ParsedOrder::OrderWrapper(Order::Type { character: '#' }),
                char('#'),
            ),
            value(
                ParsedOrder::OrderWrapper(Order::Type { character: '.' }),
                char('.'),
            ),
            value(
                ParsedOrder::OrderWrapper(Order::Type { character: '!' }),
                char('!'),
            ),
            value(
                ParsedOrder::OrderWrapper(Order::Type { character: '|' }),
                char('|'),
            ),
            value(
                ParsedOrder::OrderWrapper(Order::Type { character: '&' }),
                char('&'),
            ),
        )),
    )(input)
}

fn ampersand(input: &str) -> IResult<&str, ParsedOrder> {
    let nbsp = value(
        ParsedOrder::OrderWrapper(Order::Type { character: ' ' }),
        tag("&nbsp;"),
    );
    let emsp = value(
        ParsedOrder::OrderWrapper(Order::Type { character: '　' }),
        tag("&emsp;"),
    );
    alt((nbsp, emsp))(input)
}

fn erase_useless_tag(input: &str) -> IResult<&str, ParsedOrder> {
    let useless_tag = tuple((tag("<"), is_not(">"), end_tag_include_slash));
    value(ParsedOrder::Empty, useless_tag)(input)
}

fn next_line(input: &str) -> IResult<&str, ParsedOrder> {
    let cr = ParsedOrder::OrderWrapper(Order::CarriageReturn);
    let br_tag = value(cr.clone(), pair(tag("<br"), end_tag_include_slash));
    let more_then_2_spaces = pair(one_of(" \t"), space1);
    let space_end = value(cr, pair(more_then_2_spaces, line_ending));
    alt((br_tag, space_end))(input)
}

fn next_paragraph(input: &str) -> IResult<&str, ParsedOrder> {
    let p = ParsedOrder::OrderWrapper(Order::PageFeed);
    let end_p_tag = value(p.clone(), tag("</p>"));
    let more_then_2_lines = value(p, pair(line_ending, many1(line_ending)));
    alt((end_p_tag, more_then_2_lines))(input)
}

fn simple_char(input: &str) -> IResult<&str, ParsedOrder> {
    take(1usize)(input).map(|(rem, c)| {
        let order = if c == "\n" || c == "\r" || c == "\t" || c == " " {
            ParsedOrder::Empty
        } else {
            ParsedOrder::OrderWrapper(Order::Type {
                character: c.chars().next().unwrap(),
            })
        };
        (rem, order)
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

fn throw_event(input: &str) -> IResult<&str, ParsedOrder> {
    let script_open = pair(tag("<script"), end_tag);
    let script_close = "</script>";
    let mut script_taged = delimited(script_open, take_until(script_close), tag(script_close));
    script_taged(input).map(|(rem, parsed)| {
        (
            rem,
            ParsedOrder::OrderWrapper(Order::ThroghEvent {
                ron: parsed.to_string(),
            }),
        )
    })
}

#[cfg(test)]
mod parse_bms_tests {
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
    fn test_double_endline() {
        let pf = &[Order::PageFeed];
        let paged_phrase = [HELLO, pf, ILL]
            .into_iter()
            .map(|x| x.iter())
            .flatten()
            .map(|x| x.clone());
        let vec_pp = paged_phrase.collect::<Vec<Order>>();
        let read = read_bms("こんにちは    \r\nはじめまして\r\n\r\nこの家の主人は病気です");
        assert_eq!(read, HashMap::from([("".to_string(), vec_pp)]));
    }

    #[test]
    fn test_end_p_tag() {
        let pf = &[Order::PageFeed];
        let paged_phrase = [HELLO, pf, ILL]
            .into_iter()
            .map(|x| x.iter())
            .flatten()
            .map(|x| x.clone());
        let vec_pp = paged_phrase.collect::<Vec<Order>>();
        let read = read_bms("<p>こんにちは    \r\nはじめまして</p>この家の主人は病気です");
        assert_eq!(read, HashMap::from([("".to_string(), vec_pp)]));
    }

    #[test]
    fn test_script_tag() {
        let script = &[Order::ThroghEvent {
            ron: "test".to_string(),
        }];
        let with_script = [HELLO, script, ILL]
            .into_iter()
            .map(|x| x.iter())
            .flatten()
            .map(|x| x.clone());
        let vec_ws = with_script.collect::<Vec<Order>>();
        let read =
            read_bms("こんにちは    \r\nはじめまして<script>test</script>この家の主人は病気です");
        assert_eq!(read, HashMap::from([("".to_string(), vec_ws)]));
    }

    #[test]
    fn test_useless_tag() {
        let useless_taged = vec![
            ParsedOrder::OrderWrapper(Order::Type { character: 'a' }),
            ParsedOrder::OrderWrapper(Order::Type { character: 'a' }),
            ParsedOrder::OrderWrapper(Order::Type { character: 'b' }),
            ParsedOrder::OrderWrapper(Order::Type { character: 'c' }),
            ParsedOrder::OrderWrapper(Order::Type { character: 'd' }),
            ParsedOrder::OrderWrapper(Order::Type { character: '<' }),
            ParsedOrder::OrderWrapper(Order::Type { character: 'a' }),
            ParsedOrder::OrderWrapper(Order::Type { character: 'b' }),
            ParsedOrder::OrderWrapper(Order::Type { character: '\\' }),
            ParsedOrder::OrderWrapper(Order::Type { character: '>' }),
        ];
        assert_eq!(parse_bms("a<abc>abcd\\<ab\\\\>"), useless_taged);
    }
}
