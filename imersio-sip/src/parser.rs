//! TODO

use std::borrow::Cow;

use nom::branch::alt;
use nom::bytes::complete::{tag, take};
use nom::character::complete::crlf;
use nom::character::{is_alphabetic, is_alphanumeric, is_digit};
use nom::combinator::{map, map_res, opt, recognize, verify};
use nom::error::{ErrorKind, VerboseError};
use nom::multi::{count, many0, many1, many_m_n};
use nom::sequence::{pair, preceded, tuple};
use nom::{IResult, InputTakeAtPosition};

use crate::common::wrapped_string::WrappedString;

pub(crate) type ParserResult<T, U> = IResult<T, U, VerboseError<T>>;

pub(crate) fn take1(input: &[u8]) -> ParserResult<&[u8], u8> {
    map(take(1usize), |b: &[u8]| b[0])(input)
}

pub(crate) fn is_reserved(b: u8) -> bool {
    b";/?:@&=+$,".contains(&b)
}

pub(crate) fn is_mark(b: u8) -> bool {
    b"-_.!~*'()".contains(&b)
}

#[inline]
pub(crate) fn is_unreserved(b: u8) -> bool {
    is_alphanumeric(b) || is_mark(b)
}

#[inline]
pub(crate) fn is_utf8_cont(b: u8) -> bool {
    (0x80..=0xbf).contains(&b)
}

#[inline]
pub(crate) fn sp(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    tag(" ")(input)
}

#[inline]
pub(crate) fn tab(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    tag("\t")(input)
}

#[inline]
pub(crate) fn wsp(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    alt((sp, tab))(input)
}

// This definition diverges from RFC3261, see https://www.rfc-editor.org/errata/eid7529
pub(crate) fn lws(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(pair(opt(crlf), many1(wsp)))(input)
}

#[inline]
pub(crate) fn sws(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(opt(lws))(input)
}

pub(crate) fn hcolon(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(tuple((many0(alt((sp, tab))), tag(":"), sws)))(input)
}

pub(crate) fn comma(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(tuple((sws, tag(","), sws)))(input)
}

#[inline]
pub(crate) fn dquote(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    tag("\"")(input)
}

pub(crate) fn equal(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(tuple((sws, tag("="), sws)))(input)
}

pub(crate) fn laquot(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(pair(sws, tag("<")))(input)
}

pub(crate) fn ldquot(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(pair(sws, dquote))(input)
}

pub(crate) fn raquot(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(pair(tag(">"), sws))(input)
}

pub(crate) fn rdquot(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(pair(dquote, sws))(input)
}

pub(crate) fn semi(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(tuple((sws, tag(";"), sws)))(input)
}

pub(crate) fn slash(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(tuple((sws, tag("/"), sws)))(input)
}

pub(crate) fn star(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(tuple((sws, tag("*"), sws)))(input)
}

pub(crate) fn alpha(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(verify(take1, |b| is_alphabetic(*b)))(input)
}

pub(crate) fn digit(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(verify(take1, |b| is_digit(*b)))(input)
}

pub(crate) fn hex_digit(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(verify(take1, |b| {
        is_digit(*b) || b"ABCDEFabcdef".contains(b)
    }))(input)
}

pub(crate) fn lhex(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(verify(take1, |b| is_digit(*b) || b"abcdef".contains(b)))(input)
}

#[inline]
fn is_qdtext_first_range(b: u8) -> bool {
    (0x23..=0x5b).contains(&b)
}

fn qdtext_first_range(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(verify(take1, |b| is_qdtext_first_range(*b)))(input)
}

#[inline]
fn is_qdtext_second_range(b: u8) -> bool {
    (0x5d..=0x7e).contains(&b)
}

fn qdtext_second_range(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(verify(take1, |b| is_qdtext_second_range(*b)))(input)
}

fn qdtext(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    alt((
        lws,
        tag("!"),
        qdtext_first_range,
        qdtext_second_range,
        utf8_nonascii,
    ))(input)
}

#[inline]
fn is_quoted_pair_first_range(b: u8) -> bool {
    (0x00..=0x09).contains(&b)
}

fn quoted_pair_first_range(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(verify(take1, |b| is_quoted_pair_first_range(*b)))(input)
}

#[inline]
fn is_quoted_pair_second_range(b: u8) -> bool {
    (0x0b..=0x0c).contains(&b)
}

fn quoted_pair_second_range(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(verify(take1, |b| is_quoted_pair_second_range(*b)))(input)
}

#[inline]
fn is_quoted_pair_third_range(b: u8) -> bool {
    (0x0e..=0x7f).contains(&b)
}

fn quoted_pair_third_range(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(verify(take1, |b| is_quoted_pair_third_range(*b)))(input)
}

fn quoted_pair(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(pair(
        tag("\\"),
        alt((
            quoted_pair_first_range,
            quoted_pair_second_range,
            quoted_pair_third_range,
        )),
    ))(input)
}

pub(crate) fn quoted_string(input: &[u8]) -> ParserResult<&[u8], WrappedString> {
    map(
        tuple((
            sws,
            dquote,
            recognize(many0(alt((qdtext, quoted_pair)))),
            dquote,
        )),
        |(_, _, value, _)| WrappedString::new_quoted(String::from_utf8_lossy(value)),
    )(input)
}

pub(crate) fn escaped(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    map_res(preceded(tag("%"), count(hex_digit, 2)), |digits| {
        let idx = usize::from_str_radix(
            &digits.into_iter().map(|d| d[0] as char).collect::<String>(),
            16,
        )
        .unwrap();
        if ESCAPED_CHARS[idx] == 0 {
            Err(nom::Err::Error(ErrorKind::MapRes))
        } else {
            Ok(&ESCAPED_CHARS[idx..idx + 1])
        }
    })(input)
}

pub(crate) fn reserved(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(verify(take1, |b| is_reserved(*b)))(input)
}

pub(crate) fn unreserved(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(verify(take1, |b| is_unreserved(*b)))(input)
}

pub(crate) fn utf8_cont(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(verify(take1, |b| is_utf8_cont(*b)))(input)
}

fn utf8_nonascii_1byte(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(pair(
        verify(take1, |b| (0xc0..=0xdf).contains(b)),
        utf8_cont,
    ))(input)
}

fn utf8_nonascii_2bytes(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(pair(
        verify(take1, |b| (0xe0..=0xef).contains(b)),
        count(utf8_cont, 2),
    ))(input)
}

fn utf8_nonascii_3bytes(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(pair(
        verify(take1, |b| (0xf0..=0xf7).contains(b)),
        count(utf8_cont, 3),
    ))(input)
}

fn utf8_nonascii_4bytes(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(pair(
        verify(take1, |b| (0xf8..=0xfb).contains(b)),
        count(utf8_cont, 4),
    ))(input)
}

fn utf8_nonascii_5bytes(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(pair(
        verify(take1, |b| (0xfc..=0xfd).contains(b)),
        count(utf8_cont, 5),
    ))(input)
}

pub(crate) fn utf8_nonascii(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    alt((
        utf8_nonascii_1byte,
        utf8_nonascii_2bytes,
        utf8_nonascii_3bytes,
        utf8_nonascii_4bytes,
        utf8_nonascii_5bytes,
    ))(input)
}

pub(crate) fn text_utf8char(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    alt((
        recognize(verify(take1, |b| (0x21..=0x7e).contains(b))),
        utf8_nonascii,
    ))(input)
}

pub(crate) fn text_utf8_trim(input: &[u8]) -> ParserResult<&[u8], String> {
    map(
        recognize(pair(
            many1(text_utf8char),
            many0(pair(many1(lws), many0(text_utf8char))),
        )),
        |v| String::from_utf8_lossy(v).trim_end().to_string(),
    )(input)
}

pub(crate) fn token(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    input
        .split_at_position1_complete(
            |item| !(is_alphanumeric(item) || b"-.!%*_+`'~".contains(&item)),
            ErrorKind::AlphaNumeric,
        )
        .map(|(rest, value)| (rest, String::from_utf8_lossy(value)))
}

pub(crate) fn word(input: &[u8]) -> ParserResult<&[u8], Cow<'_, str>> {
    input
        .split_at_position1_complete(
            |item| !(is_alphanumeric(item) || b"-.!%*_+`'~()<>:\\\"/[]?{}".contains(&item)),
            ErrorKind::AlphaNumeric,
        )
        .map(|(rest, value)| (rest, String::from_utf8_lossy(value)))
}

pub(crate) fn ttl(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
    recognize(many_m_n(1, 3, digit))(input)
}

#[rustfmt::skip]
const ESCAPED_CHARS: [u8; 256] = [
    //  0      1      2      3      4      5      6      7      8      9
    b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', //   x
    b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', //  1x
    b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', //  2x
    b'\0', b'\0',  b' ',  b'!',  b'"',  b'#',  b'$',  b'%',  b'&', b'\'', //  3x
     b'(',  b')',  b'*',  b'+',  b',',  b'-',  b'.',  b'/',  b'0',  b'1', //  4x
     b'2',  b'3',  b'4',  b'5',  b'6',  b'7',  b'8',  b'9',  b':',  b';', //  5x
     b'<',  b'=',  b'>',  b'?',  b'@',  b'A',  b'B',  b'C',  b'D',  b'E', //  6x
     b'F',  b'G',  b'H',  b'I',  b'J',  b'K',  b'L',  b'M',  b'N',  b'O', //  7x
     b'P',  b'Q',  b'R',  b'S',  b'T',  b'U',  b'V',  b'W',  b'X',  b'Y', //  8x
     b'Z',  b'[', b'\\',  b']',  b'^',  b'_',  b'`',  b'a',  b'b',  b'c', //  9x
     b'd',  b'e',  b'f',  b'g',  b'h',  b'i',  b'j',  b'k',  b'l',  b'm', // 10x
     b'n',  b'o',  b'p',  b'q',  b'r',  b's',  b't',  b'u',  b'v',  b'w', // 11x
     b'x',  b'y',  b'z',  b'{',  b'|',  b'}',  b'~', b'\0', b'\0', b'\0', // 12x
    b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', // 13x
    b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', // 14x
    b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', // 15x
    b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', // 16x
    b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', // 17x
    b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', // 18x
    b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', // 19x
    b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', // 20x
    b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', // 21x
    b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', // 22x
    b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', // 23x
    b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', // 24x
    b'\0', b'\0', b'\0', b'\0', b'\0', b'\0'                              // 25x
];

#[cfg(test)]
mod tests {
    use super::*;
    use claims::assert_ok;

    #[test]
    fn test_quoted_string() {
        let input = b"\"47364c23432d2e131a5fb210812c\"";
        let result = quoted_string(input);
        assert_ok!(result);
    }
}
