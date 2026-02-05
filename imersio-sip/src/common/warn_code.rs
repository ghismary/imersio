use crate::common::status_code::CODE_DIGITS;
use nom_language::error::convert_error;
use std::convert::TryFrom;

use crate::SipError;

/// A SIP warning code.
///
/// Specific constants are provided for known warning codes, described in the
/// [section 20.43 of RFC3261](https://datatracker.ietf.org/doc/html/rfc3261#section-20.43).
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct WarnCode(pub(crate) u16);

impl WarnCode {
    /// Convert an u16 to a warning code.
    ///
    /// The function validates the correctness of the supplied u16. It must be
    /// greater or equal to 100 and less than 1000.
    /// ```
    pub fn from_u16(src: u16) -> Result<WarnCode, SipError> {
        if !(100..1000).contains(&src) {
            return Err(SipError::InvalidWarnCode(
                "not between 100 & 1000".to_string(),
            ));
        }

        Ok(Self(src))
    }

    /// Get a &str representation of the `WarnCode`.
    /// ```
    pub fn as_str(&self) -> &str {
        let offset = (self.code() - 100) as usize;
        let offset = offset * 3;

        // Invariant: self has checked range [100, 999], and CODE_DIGITS is
        // ASCII-only, of length 900 * 3 = 2700 bytes.
        &CODE_DIGITS[offset..offset + 3]
    }

    /// Get the `u16` corresponding to this `WarnCode`.
    ///
    /// # Note
    ///
    /// The same can be achieved with the `From<WarnCode>` implementation.
    /// ```
    pub fn code(&self) -> u16 {
        self.0
    }
}

impl std::fmt::Display for WarnCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

impl TryFrom<&str> for WarnCode {
    type Error = SipError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match parser::warn_code(value) {
            Ok((rest, warn_code)) => {
                if !rest.is_empty() {
                    Err(SipError::RemainingUnparsedData(rest.to_string()))
                } else {
                    Ok(warn_code)
                }
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                Err(SipError::InvalidWarnCode(convert_error(value, e)))
            }
            Err(nom::Err::Incomplete(_)) => Err(SipError::InvalidWarnCode(format!(
                "Incomplete warning code `{}`",
                value
            ))),
        }
    }
}

impl From<WarnCode> for u16 {
    #[inline]
    fn from(value: WarnCode) -> Self {
        value.code()
    }
}

impl From<&WarnCode> for WarnCode {
    #[inline]
    fn from(value: &WarnCode) -> Self {
        value.to_owned()
    }
}

impl TryFrom<u16> for WarnCode {
    type Error = SipError;

    #[inline]
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        WarnCode::from_u16(value)
    }
}

impl PartialEq<u16> for WarnCode {
    #[inline]
    fn eq(&self, other: &u16) -> bool {
        self.code() == *other
    }
}

impl PartialEq<WarnCode> for u16 {
    #[inline]
    fn eq(&self, other: &WarnCode) -> bool {
        *self == other.code()
    }
}

impl PartialEq<&WarnCode> for WarnCode {
    fn eq(&self, other: &&WarnCode) -> bool {
        self == *other
    }
}

impl PartialEq<WarnCode> for &WarnCode {
    fn eq(&self, other: &WarnCode) -> bool {
        *self == other
    }
}

macro_rules! warn_codes {
    (
        $(
            $(#[$docs:meta])*
            ($code:expr, $konst:ident),
        )+
    ) => {
        impl WarnCode {
            $(
                $(#[$docs])*
                pub const $konst: WarnCode = WarnCode($code);
            )+
        }
    }
}

warn_codes! {
    /// 110 Response is stale
    (110, RESPONSE_IS_STALE),
    /// 111 Revalidation failed
    (111, REVALIDATION_FAILED),
    /// 112 Disconnected operation
    (112, DISCONNECTED_OPERATION),
    /// 113 Heuristic expiration
    (113, HEURISTIC_EXPIRATION),
    /// 199 Miscellaneous warning
    (199, MISCELLANEOUS_WARNING),
    /// 214 Transformation applied
    (214, TRANSFORMATION_APPLIED),
    /// 299 Miscellaneous persistent warning
    (299, MISCELLANEOUS_PERSISTENT_WARNING),
    /// 300 Incompatible network protocol
    (300, INCOMPATIBLE_NETWORK_PROTOCOL),
    /// 301 Incompatible network address formats
    (301, INCOMPATIBLE_NETWORK_ADDRESS_FORMATS),
    /// 302 Incompatible transport protocol
    (302, INCOMPATIBLE_TRANSPORT_PROTOCOL),
    /// 303 Incompatible bandwidth units
    (303, INCOMPATIBLE_BANDWIDTH_UNITS),
    /// 304 Media type not available
    (304, MEDIA_TYPE_NOT_AVAILABLE),
    /// 305 Incompatible media format
    (305, INCOMPATIBLE_MEDIA_FORMAT),
    /// 306 Attribute not understood
    (306, ATTRITBUTE_NOT_UNDERSTOOD),
    /// 307 Session description parameter not understood
    (307, SESSION_DESCRIPTION_PARAMETER_NOT_UNDERSTOOD),
    /// 330 Multicast not available
    (330, MULTICAST_NOT_AVAILABLE),
    /// 331 Unicast not available
    (331, UNICAST_NOT_AVAILABLE),
    /// 370 Insufficient bandwidth
    (370, INSUFFICIENT_BANDWIDTH),
    /// 399 Miscellaneous warning
    (399, MISCELLANEOUS_WARNING_3),
}

pub(crate) mod parser {
    use nom::{
        Parser,
        combinator::{map, recognize},
        error::context,
        multi::count,
        sequence::pair,
    };

    use super::*;
    use crate::parser::{ParserResult, digit, positive_digit};

    pub(crate) fn warn_code(input: &str) -> ParserResult<&str, WarnCode> {
        context(
            "warn_code",
            map(recognize(pair(positive_digit, count(digit, 2))), |result| {
                let status = result.parse::<u16>().unwrap();
                WarnCode(status)
            }),
        )
        .parse(input)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use claims::{assert_err, assert_ok};

    #[test]
    fn test_warn_code_eq() {
        assert_eq!(WarnCode::TRANSFORMATION_APPLIED, 214);
        assert_eq!(299, WarnCode::MISCELLANEOUS_PERSISTENT_WARNING);

        assert_eq!(
            WarnCode::DISCONNECTED_OPERATION,
            WarnCode::from_u16(112).unwrap()
        );
        assert_eq!(
            WarnCode::from_u16(370).unwrap(),
            WarnCode::INSUFFICIENT_BANDWIDTH
        );
    }

    #[test]
    fn test_valid_warn_code_incompatible_media_format() {
        assert!(WarnCode::try_from("305").is_ok_and(|v| v == WarnCode::INCOMPATIBLE_MEDIA_FORMAT));
    }

    #[test]
    fn test_valid_warn_code_incompatible_network_protocol() {
        assert!(
            WarnCode::try_from("300").is_ok_and(|v| v == WarnCode::INCOMPATIBLE_NETWORK_PROTOCOL)
        );
    }

    #[test]
    fn test_valid_status_code_transformation_applied() {
        assert!(WarnCode::try_from("214").is_ok_and(|v| v == WarnCode::TRANSFORMATION_APPLIED));
    }

    #[test]
    fn test_valid_warn_code_extension() {
        assert_ok!(WarnCode::try_from("829"));
        assert_ok!(WarnCode::try_from(157));
    }

    #[test]
    fn test_invalid_warn_code_under_100() {
        assert_err!(WarnCode::try_from("99"));
        assert_err!(WarnCode::from_u16(10));
    }

    #[test]
    fn test_invalid_warn_code_over_999() {
        assert_err!(WarnCode::from_u16(3478));
        assert_err!(WarnCode::try_from("9273"));
        assert_err!(WarnCode::try_from("4629"));
    }

    #[test]
    fn test_invalid_warn_code_0() {
        assert_err!(WarnCode::from_u16(0));
        assert_err!(WarnCode::try_from("000"));
    }

    #[test]
    fn test_invalid_warn_code_not_a_number() {
        assert_err!(WarnCode::try_from("bob"));
    }

    #[test]
    fn test_valid_warn_code_but_with_remaining_data() {
        assert!(
            WarnCode::try_from("306 anything")
                .is_err_and(|e| e == SipError::RemainingUnparsedData(" anything".to_string()))
        );
    }
}
