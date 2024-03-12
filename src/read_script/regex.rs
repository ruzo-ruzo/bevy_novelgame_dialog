use nom::branch::*;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::*;
use nom::multi::*;
use nom::sequence::*;
use nom::*;
use regex::Regex;

//エラー処理がちょっと雑
pub fn replace_by_template<S1: AsRef<str>, S2: AsRef<str>>(input: S1, template: S2) -> String {
    let template_vec = read_template(template.as_ref());
    template_vec
        .into_iter()
        .fold(input.as_ref().to_string(), |base, (from, to)| {
            Regex::new(&from)
                .unwrap()
                .replace_all(&base, to)
                .to_string()
        })
}

fn read_template(input: &str) -> Vec<(String, String)> {
    let base = parse_csv(input).into_iter();
    base.filter_map(|mut v| {
        let (v1, v2) = (v.pop()?, v.pop()?);
        Some((v2, v1))
    })
    .collect()
}

fn parse_csv(input: &str) -> Vec<Vec<String>> {
    let last_char = input.chars().last();
    let is_eof_endline = last_char == Some('\n') || last_char == Some('\r');
    let added_last_line = if is_eof_endline {
        input.to_string()
    } else {
        [input, "\r\n"].concat()
    };
    let (_, csv) = many0(a_row)(added_last_line.as_str()).unwrap_or_default();
    csv.into_iter().filter(|v| !v.is_empty()).collect()
}

fn a_row(input: &str) -> IResult<&str, Vec<String>> {
    let cells = many_till(a_cell, verify(a_cell, |(_, b)| *b));
    let to_vec_string = |parsed: (Vec<(String, bool)>, (String, bool))| {
        let (succ, (last, _)) = parsed;
        let inits = succ.into_iter().map(|(s, _)| s);
        inits.chain([last]).collect::<Vec<_>>()
    };
    map(cells, to_vec_string)(input)
}

fn a_cell(input: &str) -> IResult<&str, (String, bool)> {
    alt((double_quoted_cell, single_quoted_cell, non_quoted_cell))(input)
}

fn double_quoted_cell(input: &str) -> IResult<&str, (String, bool)> {
    quoted_cell("\"")(input)
}

fn single_quoted_cell(input: &str) -> IResult<&str, (String, bool)> {
    quoted_cell("\'")(input)
}

fn quoted_cell(s: &str) -> impl FnMut(&str) -> IResult<&str, (String, bool)> + '_ {
    move |input| {
        let empty = pair(
            value("".to_string(), tuple((space0, tag(s), tag(s)))),
            cell_ending,
        );
        let double_quote_open = tuple((space0, tag(s), not(tag(s))));
        let escaped_double_quote = value(s.to_string(), preceded(tag(s), tag(s)));
        let non_closing = map(not(quote_close(s)).and(take(1usize)), |(_, x)| {
            x.to_string()
        });
        let inside_vec = many0(alt((escaped_double_quote, non_closing)));
        let inside = map(inside_vec, |v| v.concat());
        let non_empty = preceded(double_quote_open, pair(inside, quote_close(s)));
        alt((empty, non_empty))(input)
    }
}

fn non_quoted_cell(input: &str) -> IResult<&str, (String, bool)> {
    map(many_till(take(1usize), cell_ending), |(v, b)| {
        (v.concat(), b)
    })(input)
}

fn cell_ending(input: &str) -> IResult<&str, bool> {
    preceded(
        space0,
        alt((value(false, tag(",")), value(true, line_ending))),
    )(input)
}

fn quote_close(s: &str) -> impl FnMut(&str) -> IResult<&str, bool> + '_ {
    move |input| preceded(tag(s), cell_ending)(input)
}

#[cfg(test)]
mod parse_template_tests {
    use super::*;

    fn str_refs2strings(base: Vec<Vec<&str>>) -> Vec<Vec<String>> {
        base.iter()
            .map(|x| x.iter().map(|y| y.to_string()).collect())
            .collect()
    }

    #[test]
    fn test_parse_simple_csv() {
        let csv = "abc,defg\r\nhijk,lmn";
        let parsed_base = vec![vec!["abc", "defg"], vec!["hijk", "lmn"]];
        let parsed: Vec<Vec<String>> = str_refs2strings(parsed_base);
        assert_eq!(parse_csv(csv), parsed)
    }

    #[test]
    fn test_parse_quoted_csv() {
        let csv = r#""ab,c","'def""g'"
hijk,lmn"#;
        let parsed_base = vec![vec!["ab,c", "\'def\"g\'"], vec!["hijk", "lmn"]];
        let parsed: Vec<Vec<String>> = str_refs2strings(parsed_base);
        assert_eq!(parse_csv(csv), parsed)
    }

    #[test]
    fn test_parse_empty_csv() {
        let csv = r#"a
bc, "",de
hijk,,""
"#;
        let parsed_base = vec![vec!["a"], vec!["bc", "", "de"], vec!["hijk", "", ""]];
        let parsed: Vec<Vec<String>> = str_refs2strings(parsed_base);
        assert_eq!(parse_csv(csv), parsed)
    }

    #[test]
    fn test_replace_by_template() {
        let csv = r#"
"\*(?<t>.*?)\*", "<script>{
    "bevy_novelgame_dialog::dialog_box::public_events::bds_event::ChangeFontSize": (
        size: 35.0,
),}</script>
$t
<script>{
    "bevy_novelgame_dialog::dialog_box::public_events::bds_event::ChangeFontSize": (
        size: 27.0,
)}</script>"
"\[close\]","<script>{
    "bevy_novelgame_dialog::dialog_box::window_controller::sinkdown::SinkDownWindow": (
    sink_type: Scale(
            sec: 0.8,
        ),
    ),
}</script>"
"#;
        let base = "あい*うえ*お";
        let replaced = r#"あい<script>{
    "bevy_novelgame_dialog::dialog_box::public_events::bds_event::ChangeFontSize": (
        size: 35.0,
),}</script>
うえ
<script>{
    "bevy_novelgame_dialog::dialog_box::public_events::bds_event::ChangeFontSize": (
        size: 27.0,
)}</script>お"#;
        assert_eq!(replace_by_template(base, csv), replaced);
    }
}
