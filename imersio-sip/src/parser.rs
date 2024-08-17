//! Generic parser rules used throughout the whole crate.

use nom::branch::alt;
use nom::bytes::complete::{tag, take};
use nom::character::complete::crlf;
use nom::combinator::{map, map_res, opt, recognize, verify};
use nom::error::{context, ErrorKind, VerboseError};
use nom::multi::{count, many0, many1, many_m_n, separated_list1};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::{IResult, InputTakeAtPosition};

use crate::common::wrapped_string::WrappedString;

pub(crate) type ParserResult<T, U> = IResult<T, U, VerboseError<T>>;

pub(crate) fn take1(input: &str) -> ParserResult<&str, char> {
    map(take(1usize), |c: &str| c.chars().next().unwrap())(input)
}

pub(crate) fn is_reserved(c: char) -> bool {
    ";/?:@&=+$,".contains(c)
}

pub(crate) fn is_mark(c: char) -> bool {
    "-_.!~*'()".contains(c)
}

#[inline]
pub(crate) fn is_unreserved(c: char) -> bool {
    c.is_alphanumeric() || is_mark(c)
}

#[inline]
pub(crate) fn is_utf8char(c: char) -> bool {
    matches!(c as u32, 0x21..=0x7e | 0x80..)
}

#[inline]
pub(crate) fn is_utf8nonascii(c: char) -> bool {
    matches!(c as u32, 0x80..)
}

#[inline]
pub(crate) fn sp(input: &str) -> ParserResult<&str, &str> {
    tag(" ")(input)
}

#[inline]
pub(crate) fn tab(input: &str) -> ParserResult<&str, &str> {
    tag("\t")(input)
}

#[inline]
pub(crate) fn wsp(input: &str) -> ParserResult<&str, &str> {
    alt((sp, tab))(input)
}

// This definition diverges from RFC3261, see https://www.rfc-editor.org/errata/eid7529
pub(crate) fn lws(input: &str) -> ParserResult<&str, &str> {
    recognize(pair(opt(crlf), many1(wsp)))(input)
}

#[inline]
pub(crate) fn sws(input: &str) -> ParserResult<&str, &str> {
    recognize(opt(lws))(input)
}

pub(crate) fn hcolon(input: &str) -> ParserResult<&str, &str> {
    recognize(tuple((many0(alt((sp, tab))), tag(":"), sws)))(input)
}

pub(crate) fn colon(input: &str) -> ParserResult<&str, &str> {
    recognize(tuple((sws, tag(":"), sws)))(input)
}

pub(crate) fn comma(input: &str) -> ParserResult<&str, &str> {
    recognize(tuple((sws, tag(","), sws)))(input)
}

#[inline]
pub(crate) fn dquote(input: &str) -> ParserResult<&str, &str> {
    tag("\"")(input)
}

pub(crate) fn equal(input: &str) -> ParserResult<&str, &str> {
    recognize(tuple((sws, tag("="), sws)))(input)
}

pub(crate) fn lparen(input: &str) -> ParserResult<&str, &str> {
    recognize(tuple((sws, tag("("), sws)))(input)
}

pub(crate) fn rparen(input: &str) -> ParserResult<&str, &str> {
    recognize(tuple((sws, tag(")"), sws)))(input)
}

pub(crate) fn laquot(input: &str) -> ParserResult<&str, &str> {
    recognize(pair(sws, tag("<")))(input)
}

pub(crate) fn ldquot(input: &str) -> ParserResult<&str, &str> {
    recognize(pair(sws, dquote))(input)
}

pub(crate) fn raquot(input: &str) -> ParserResult<&str, &str> {
    recognize(pair(tag(">"), sws))(input)
}

pub(crate) fn rdquot(input: &str) -> ParserResult<&str, &str> {
    recognize(pair(dquote, sws))(input)
}

pub(crate) fn semi(input: &str) -> ParserResult<&str, &str> {
    recognize(tuple((sws, tag(";"), sws)))(input)
}

pub(crate) fn slash(input: &str) -> ParserResult<&str, &str> {
    recognize(tuple((sws, tag("/"), sws)))(input)
}

pub(crate) fn star(input: &str) -> ParserResult<&str, &str> {
    recognize(tuple((sws, tag("*"), sws)))(input)
}

pub(crate) fn alpha(input: &str) -> ParserResult<&str, char> {
    verify(take1, |c| c.is_alphabetic())(input)
}

pub(crate) fn digit(input: &str) -> ParserResult<&str, char> {
    verify(take1, |c| c.is_ascii_digit())(input)
}

pub(crate) fn positive_digit(input: &str) -> ParserResult<&str, char> {
    verify(take1, |c| c.is_ascii_digit() && *c != '0')(input)
}

pub(crate) fn hex_digit(input: &str) -> ParserResult<&str, char> {
    verify(take1, |c| c.is_ascii_hexdigit())(input)
}

pub(crate) fn lhex(input: &str) -> ParserResult<&str, &str> {
    recognize(verify(take1, |c| {
        c.is_ascii_digit() || "abcdef".contains(*c)
    }))(input)
}

#[inline]
fn is_qdtext_first_range(c: char) -> bool {
    matches!(c as u32, 0x23..=0x5b)
}

fn qdtext_first_range(input: &str) -> ParserResult<&str, &str> {
    recognize(verify(take1, |c| is_qdtext_first_range(*c)))(input)
}

#[inline]
fn is_qdtext_second_range(c: char) -> bool {
    matches!(c as u32, 0x5d..=0x7e)
}

fn qdtext_second_range(input: &str) -> ParserResult<&str, &str> {
    recognize(verify(take1, |c| is_qdtext_second_range(*c)))(input)
}

#[inline]
fn is_obs_text(c: char) -> bool {
    matches!(c as u32, 0x80..=0xff)
}

fn obs_text(input: &str) -> ParserResult<&str, &str> {
    recognize(verify(take1, |c| is_obs_text(*c)))(input)
}

fn qdtext(input: &str) -> ParserResult<&str, &str> {
    context(
        "qdtext",
        alt((
            lws,
            tag("!"),
            qdtext_first_range,
            qdtext_second_range,
            obs_text,
        )),
    )(input)
}

#[inline]
fn is_quoted_pair_first_range(c: char) -> bool {
    matches!(c as u32, 0x00..=0x09)
}

fn quoted_pair_first_range(input: &str) -> ParserResult<&str, &str> {
    recognize(verify(take1, |c| is_quoted_pair_first_range(*c)))(input)
}

#[inline]
fn is_quoted_pair_second_range(c: char) -> bool {
    matches!(c as u32, 0x0b..=0x0c)
}

fn quoted_pair_second_range(input: &str) -> ParserResult<&str, &str> {
    recognize(verify(take1, |c| is_quoted_pair_second_range(*c)))(input)
}

#[inline]
fn is_quoted_pair_third_range(c: char) -> bool {
    matches!(c as u32, 0x0e..=0x7f)
}

fn quoted_pair_third_range(input: &str) -> ParserResult<&str, &str> {
    recognize(verify(take1, |c| is_quoted_pair_third_range(*c)))(input)
}

fn quoted_pair(input: &str) -> ParserResult<&str, &str> {
    context(
        "quoted_pair",
        recognize(pair(
            tag("\\"),
            alt((
                quoted_pair_first_range,
                quoted_pair_second_range,
                quoted_pair_third_range,
            )),
        )),
    )(input)
}

pub(crate) fn comment(input: &str) -> ParserResult<&str, &str> {
    context(
        "comment",
        delimited(
            lparen,
            recognize(many0(alt((ctext, quoted_pair, comment)))),
            rparen,
        ),
    )(input)
}

#[inline]
fn is_ctext(c: char) -> bool {
    matches!(c as u32, 0x21..=0x27 | 0x2a..=0x5b | 0x5d..=0x7e)
}

fn ctext(input: &str) -> ParserResult<&str, &str> {
    recognize(alt((
        recognize(verify(take1, |c| is_ctext(*c))),
        recognize(utf8_nonascii),
        lws,
    )))(input)
}

pub(crate) fn quoted_string(input: &str) -> ParserResult<&str, WrappedString> {
    context(
        "quoted_string",
        map(
            tuple((
                sws,
                dquote,
                recognize(many0(alt((qdtext, quoted_pair)))),
                dquote,
            )),
            |(_, _, value, _)| WrappedString::new_quoted(value),
        ),
    )(input)
}

pub(crate) fn escaped(input: &str) -> ParserResult<&str, char> {
    map_res(
        preceded(tag("%"), recognize(count(hex_digit, 2))),
        |digits| {
            let idx = usize::from_str_radix(digits, 16).unwrap();
            if ESCAPED_CHARS[idx] == '\0' {
                Err(nom::Err::Error(ErrorKind::MapRes))
            } else {
                Ok(ESCAPED_CHARS[idx])
            }
        },
    )(input)
}

pub(crate) fn reserved(input: &str) -> ParserResult<&str, char> {
    verify(take1, |c| is_reserved(*c))(input)
}

pub(crate) fn unreserved(input: &str) -> ParserResult<&str, char> {
    verify(take1, |c| is_unreserved(*c))(input)
}

pub(crate) fn text_utf8_trim(input: &str) -> ParserResult<&str, String> {
    context(
        "text_utf8_trim",
        map(
            terminated(
                separated_list1(many1(lws), recognize(many1(text_utf8char))),
                many0(lws),
            ),
            |words| words.join(" "),
        ),
    )(input)
}

pub(crate) fn text_utf8char(input: &str) -> ParserResult<&str, char> {
    verify(take1, |c| is_utf8char(*c))(input)
}

pub(crate) fn utf8_nonascii(input: &str) -> ParserResult<&str, char> {
    verify(take1, |c| is_utf8nonascii(*c))(input)
}

pub(crate) fn token(input: &str) -> ParserResult<&str, &str> {
    input.split_at_position1_complete(
        |item| !(item.is_alphanumeric() || "-.!%*_+`'~".contains(item)),
        ErrorKind::AlphaNumeric,
    )
}

pub(crate) fn word(input: &str) -> ParserResult<&str, &str> {
    input.split_at_position1_complete(
        |c| !(c.is_alphanumeric() || "-.!%*_+`'~()<>:\\\"/[]?{}".contains(c)),
        ErrorKind::AlphaNumeric,
    )
}

pub(crate) fn ttl(input: &str) -> ParserResult<&str, &str> {
    recognize(many_m_n(1, 3, digit))(input)
}

pub(crate) fn pchar(input: &str) -> ParserResult<&str, char> {
    alt((
        unreserved,
        escaped,
        verify(take1, |c| ":@&=+$,".contains(*c)),
    ))(input)
}

pub(crate) fn param(input: &str) -> ParserResult<&str, &str> {
    recognize(many0(pchar))(input)
}

#[rustfmt::skip]
const ESCAPED_CHARS: [char; 256] = [
    //  0      1      2      3      4      5      6      7      8      9
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', //   x
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', //  1x
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', //  2x
    '\0', '\0',  ' ',  '!',  '"',  '#',  '$',  '%',  '&', '\'', //  3x
     '(',  ')',  '*',  '+',  ',',  '-',  '.',  '/',  '0',  '1', //  4x
     '2',  '3',  '4',  '5',  '6',  '7',  '8',  '9',  ':',  ';', //  5x
     '<',  '=',  '>',  '?',  '@',  'A',  'B',  'C',  'D',  'E', //  6x
     'F',  'G',  'H',  'I',  'J',  'K',  'L',  'M',  'N',  'O', //  7x
     'P',  'Q',  'R',  'S',  'T',  'U',  'V',  'W',  'X',  'Y', //  8x
     'Z',  '[', '\\',  ']',  '^',  '_',  '`',  'a',  'b',  'c', //  9x
     'd',  'e',  'f',  'g',  'h',  'i',  'j',  'k',  'l',  'm', // 10x
     'n',  'o',  'p',  'q',  'r',  's',  't',  'u',  'v',  'w', // 11x
     'x',  'y',  'z',  '{',  '|',  '}',  '~', '\0', '\0', '\0', // 12x
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 13x
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 14x
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 15x
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 16x
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 17x
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 18x
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 19x
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 20x
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 21x
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 22x
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 23x
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 24x
    '\0', '\0', '\0', '\0', '\0', '\0'                              // 25x
];

#[cfg(test)]
mod tests {
    use super::*;
    use claims::assert_ok;

    #[test]
    fn test_quoted_string() {
        let input = "\"47364c23432d2e131a5fb210812c\"";
        let result = quoted_string(input);
        assert_ok!(result);
    }

    #[test]
    fn test_text_utf8_trim() {
        let input = "Boxes by Bob";
        let result = text_utf8_trim(input);
        assert_ok!(result);
    }
}
