use nom::error::convert_error;
use std::convert::TryFrom;

use crate::Error;

/// A SIP response status code (`Status-Code` in RFC3261).
///
/// Specific constants are provided for known status codes, described in the
/// [section 21 of RFC3261](https://datatracker.ietf.org/doc/html/rfc3261#section-21).
///
/// Status code values in the range 100-999 (inclusive) are supported by this
/// type. Values in the range 100-699 are semantically classified by the most
/// significant digit, either as provisional, success, redirection, request
/// failure, server failure or global failure. Values above 699 are
/// unclassified but allowed for compatibility, though their use is
/// discouraged. These would probably be interpreted as protocol errors by the
/// application.
///
/// # Examples
///
/// ```
/// use imersio_sip::StatusCode;
///
/// assert_eq!(StatusCode::from_u16(200).unwrap(), StatusCode::OK);
/// assert_eq!(StatusCode::NOT_FOUND.code(), 404);
/// assert!(StatusCode::TRYING.is_provisional());
/// assert!(StatusCode::OK.is_final());
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusCode(pub(crate) u16);

impl StatusCode {
    /// Convert an u16 to a status code.
    ///
    /// The function validates the correctness of the supplied u16. It must be
    /// greater or equal to 100 and less than 1000.
    ///
    /// # Example
    ///
    /// ```
    /// use imersio_sip::StatusCode;
    ///
    /// let trying = StatusCode::from_u16(100).unwrap();
    /// assert_eq!(trying, StatusCode::TRYING);
    ///
    /// let err1 = StatusCode::from_u16(99);
    /// assert!(err1.is_err());
    ///
    /// let err2 = StatusCode::from_u16(2738);
    /// assert!(err2.is_err());
    /// ```
    pub fn from_u16(src: u16) -> Result<StatusCode, Error> {
        if !(100..1000).contains(&src) {
            return Err(Error::InvalidStatusCode(
                "not between 100 & 1000".to_string(),
            ));
        }

        Ok(Self(src))
    }

    /// Get a &str representation of the `StatusCode`.
    ///
    /// # Example
    ///
    /// ```
    /// let status = imersio_sip::StatusCode::OK;
    /// assert_eq!(status.as_str(), "200");
    /// ```
    pub fn as_str(&self) -> &str {
        let offset = (self.code() - 100) as usize;
        let offset = offset * 3;

        // Invariant: self has checked range [100, 999] and CODE_DIGITS is
        // ASCII-only, of length 900 * 3 = 2700 bytes.
        &CODE_DIGITS[offset..offset + 3]
    }

    /// Get the `u16` corresponding to this `StatusCode`.
    ///
    /// # Note
    ///
    /// The same can be achieved with the `From<StatusCode>` implementation.
    ///
    /// # Example
    ///
    /// ```
    /// let status = imersio_sip::StatusCode::NOT_FOUND;
    /// assert_eq!(status.code(), 404);
    /// ```
    pub fn code(&self) -> u16 {
        self.0
    }

    /// Check if the status code is provisional (between 100 and 199).
    ///
    /// # Example
    ///
    /// ```
    /// assert!(imersio_sip::StatusCode::RINGING.is_provisional());
    /// assert!(!imersio_sip::StatusCode::REQUEST_TIMEOUT.is_provisional());
    /// ```
    #[inline]
    pub fn is_provisional(&self) -> bool {
        self.code() >= 100 && self.code() < 200
    }

    /// Check if the status code is final (between 200 and 699).
    ///
    /// # Example
    ///
    /// ```
    /// assert!(imersio_sip::StatusCode::USE_PROXY.is_final());
    /// assert!(!imersio_sip::StatusCode::TRYING.is_final());
    /// ```
    #[inline]
    pub fn is_final(&self) -> bool {
        self.code() >= 200 && self.code() < 700
    }

    /// Check if the status code is a success (between 200 and 299).
    ///
    /// # Example
    ///
    /// ```
    /// assert!(imersio_sip::StatusCode::OK.is_success());
    /// assert!(!imersio_sip::StatusCode::SERVER_INTERNAL_ERROR.is_success());
    /// ```
    #[inline]
    pub fn is_success(&self) -> bool {
        self.code() >= 200 && self.code() < 300
    }

    /// Check if the status code is a redirection (between 300 and 399).
    ///
    /// # Example
    ///
    /// ```
    /// assert!(imersio_sip::StatusCode::MOVED_TEMPORARILY.is_redirection());
    /// assert!(!imersio_sip::StatusCode::BUSY_EVERYWHERE.is_redirection());
    /// ```
    #[inline]
    pub fn is_redirection(&self) -> bool {
        self.code() >= 300 && self.code() < 400
    }

    /// Check if the status code is a request failure (between 400 and 499).
    ///
    /// # Example
    ///
    /// ```
    /// assert!(imersio_sip::StatusCode::NOT_FOUND.is_request_failure());
    /// assert!(!imersio_sip::StatusCode::ALTERNATE_SERVICE.is_request_failure());
    /// ```
    #[inline]
    pub fn is_request_failure(&self) -> bool {
        self.code() >= 400 && self.code() < 500
    }

    /// Check if the status code is a server failure (between 500 and 599).
    ///
    /// # Example
    ///
    /// ```
    /// assert!(imersio_sip::StatusCode::NOT_IMPLEMENTED.is_server_failure());
    /// assert!(!imersio_sip::StatusCode::QUEUED.is_server_failure());
    /// ```
    #[inline]
    pub fn is_server_failure(&self) -> bool {
        self.code() >= 500 && self.code() < 600
    }

    /// Check if the status code is a global failure (between 600 and 699).
    ///
    /// # Example
    ///
    /// ```
    /// assert!(imersio_sip::StatusCode::DOES_NOT_EXIST_ANYWHERE.is_global_failure());
    /// assert!(!imersio_sip::StatusCode::LOOP_DETECTED.is_global_failure());
    /// ```
    #[inline]
    pub fn is_global_failure(&self) -> bool {
        self.code() >= 600 && self.code() < 699
    }
}

impl Default for StatusCode {
    #[inline]
    fn default() -> Self {
        StatusCode::OK
    }
}

impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

impl TryFrom<&str> for StatusCode {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match parser::status_code(value) {
            Ok((rest, status_code)) => {
                if !rest.is_empty() {
                    Err(Error::RemainingUnparsedData(rest.to_string()))
                } else {
                    Ok(status_code)
                }
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                Err(Error::InvalidStatusCode(convert_error(value, e)))
            }
            Err(nom::Err::Incomplete(_)) => Err(Error::InvalidStatusCode(format!(
                "Incomplete status code `{}`",
                value
            ))),
        }
    }
}

impl From<StatusCode> for u16 {
    #[inline]
    fn from(value: StatusCode) -> Self {
        value.code()
    }
}

impl From<&StatusCode> for StatusCode {
    #[inline]
    fn from(value: &StatusCode) -> Self {
        value.to_owned()
    }
}

impl TryFrom<u16> for StatusCode {
    type Error = Error;

    #[inline]
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        StatusCode::from_u16(value)
    }
}

impl PartialEq<u16> for StatusCode {
    #[inline]
    fn eq(&self, other: &u16) -> bool {
        self.code() == *other
    }
}

impl PartialEq<StatusCode> for u16 {
    #[inline]
    fn eq(&self, other: &StatusCode) -> bool {
        *self == other.code()
    }
}

impl PartialEq<&StatusCode> for StatusCode {
    fn eq(&self, other: &&StatusCode) -> bool {
        self == *other
    }
}

impl PartialEq<StatusCode> for &StatusCode {
    fn eq(&self, other: &StatusCode) -> bool {
        *self == other
    }
}

// A string of packed 3-ASCII-digit status code values for the supported range
// of [100, 999] (900 codes, 2700 bytes).
const CODE_DIGITS: &str = "\
100101102103104105106107108109110111112113114115116117118119\
120121122123124125126127128129130131132133134135136137138139\
140141142143144145146147148149150151152153154155156157158159\
160161162163164165166167168169170171172173174175176177178179\
180181182183184185186187188189190191192193194195196197198199\
200201202203204205206207208209210211212213214215216217218219\
220221222223224225226227228229230231232233234235236237238239\
240241242243244245246247248249250251252253254255256257258259\
260261262263264265266267268269270271272273274275276277278279\
280281282283284285286287288289290291292293294295296297298299\
300301302303304305306307308309310311312313314315316317318319\
320321322323324325326327328329330331332333334335336337338339\
340341342343344345346347348349350351352353354355356357358359\
360361362363364365366367368369370371372373374375376377378379\
380381382383384385386387388389390391392393394395396397398399\
400401402403404405406407408409410411412413414415416417418419\
420421422423424425426427428429430431432433434435436437438439\
440441442443444445446447448449450451452453454455456457458459\
460461462463464465466467468469470471472473474475476477478479\
480481482483484485486487488489490491492493494495496497498499\
500501502503504505506507508509510511512513514515516517518519\
520521522523524525526527528529530531532533534535536537538539\
540541542543544545546547548549550551552553554555556557558559\
560561562563564565566567568569570571572573574575576577578579\
580581582583584585586587588589590591592593594595596597598599\
600601602603604605606607608609610611612613614615616617618619\
620621622623624625626627628629630631632633634635636637638639\
640641642643644645646647648649650651652653654655656657658659\
660661662663664665666667668669670671672673674675676677678679\
680681682683684685686687688689690691692693694695696697698699\
700701702703704705706707708709710711712713714715716717718719\
720721722723724725726727728729730731732733734735736737738739\
740741742743744745746747748749750751752753754755756757758759\
760761762763764765766767768769770771772773774775776777778779\
780781782783784785786787788789790791792793794795796797798799\
800801802803804805806807808809810811812813814815816817818819\
820821822823824825826827828829830831832833834835836837838839\
840841842843844845846847848849850851852853854855856857858859\
860861862863864865866867868869870871872873874875876877878879\
880881882883884885886887888889890891892893894895896897898899\
900901902903904905906907908909910911912913914915916917918919\
920921922923924925926927928929930931932933934935936937938939\
940941942943944945946947948949950951952953954955956957958959\
960961962963964965966967968969970971972973974975976977978979\
980981982983984985986987988989990991992993994995996997998999";

pub(crate) mod parser {
    use super::*;
    use crate::parser::{digit, positive_digit, ParserResult};
    use nom::{
        branch::alt,
        bytes::complete::tag,
        combinator::{recognize, value},
        error::context,
        multi::count,
        sequence::pair,
    };

    #[inline]
    fn informational(input: &str) -> ParserResult<&str, StatusCode> {
        alt((
            value(StatusCode::TRYING, tag("100")),
            value(StatusCode::RINGING, tag("180")),
            value(StatusCode::CALL_IS_BEING_FORWARDED, tag("181")),
            value(StatusCode::QUEUED, tag("182")),
            value(StatusCode::SESSION_PROGRESS, tag("183")),
        ))(input)
    }

    #[inline]
    fn success(input: &str) -> ParserResult<&str, StatusCode> {
        value(StatusCode::OK, tag("200"))(input)
    }

    #[inline]
    fn redirection(input: &str) -> ParserResult<&str, StatusCode> {
        alt((
            value(StatusCode::MULTIPLE_CHOICES, tag("300")),
            value(StatusCode::MOVED_PERMANENTLY, tag("301")),
            value(StatusCode::MOVED_TEMPORARILY, tag("302")),
            value(StatusCode::USE_PROXY, tag("305")),
            value(StatusCode::ALTERNATE_SERVICE, tag("380")),
        ))(input)
    }

    #[inline]
    fn client_error(input: &str) -> ParserResult<&str, StatusCode> {
        alt((
            value(StatusCode::BAD_REQUEST, tag("400")),
            value(StatusCode::UNAUTHORIZED, tag("401")),
            value(StatusCode::PAYMENT_REQUIRED, tag("402")),
            value(StatusCode::FORBIDDEN, tag("403")),
            value(StatusCode::NOT_FOUND, tag("404")),
            value(StatusCode::METHOD_NOT_ALLOWED, tag("405")),
            value(StatusCode::NOT_ACCEPTABLE, tag("406")),
            value(StatusCode::PROXY_AUTHENTICATION_REQUIRED, tag("407")),
            value(StatusCode::REQUEST_TIMEOUT, tag("408")),
            value(StatusCode::GONE, tag("410")),
            value(StatusCode::REQUEST_ENTITY_TOO_LARGE, tag("413")),
            value(StatusCode::REQUEST_URI_TOO_LONG, tag("414")),
            value(StatusCode::UNSUPPORTED_MEDIA_TYPE, tag("415")),
            value(StatusCode::UNSUPPORTED_URI_SCHEME, tag("416")),
            value(StatusCode::BAD_EXTENSION, tag("420")),
            value(StatusCode::EXTENSION_REQUIRED, tag("421")),
            value(StatusCode::INTERVAL_TOO_BRIEF, tag("423")),
            alt((
                value(StatusCode::TEMPORARILY_UNAVAILABLE, tag("480")),
                value(StatusCode::CALL_TRANSACTION_DOES_NOT_EXIST, tag("481")),
                value(StatusCode::LOOP_DETECTED, tag("482")),
                value(StatusCode::TOO_MANY_HOPS, tag("483")),
                value(StatusCode::ADDRESS_INCOMPLETE, tag("484")),
                value(StatusCode::AMBIGUOUS, tag("485")),
                value(StatusCode::BUSY_HERE, tag("486")),
                value(StatusCode::REQUEST_TERMINATED, tag("487")),
                value(StatusCode::NOT_ACCEPTABLE_HERE, tag("488")),
                value(StatusCode::REQUEST_PENDING, tag("491")),
                value(StatusCode::UNDECIPHERABLE, tag("493")),
            )),
        ))(input)
    }

    #[inline]
    fn server_error(input: &str) -> ParserResult<&str, StatusCode> {
        alt((
            value(StatusCode::SERVER_INTERNAL_ERROR, tag("500")),
            value(StatusCode::NOT_IMPLEMENTED, tag("501")),
            value(StatusCode::BAD_GATEWAY, tag("502")),
            value(StatusCode::SERVICE_UNAVAILABLE, tag("503")),
            value(StatusCode::SERVER_TIMEOUT, tag("504")),
            value(StatusCode::VERSION_NOT_SUPPORTED, tag("505")),
            value(StatusCode::MESSAGE_TOO_LARGE, tag("513")),
        ))(input)
    }

    #[inline]
    fn global_failure(input: &str) -> ParserResult<&str, StatusCode> {
        alt((
            value(StatusCode::BUSY_EVERYWHERE, tag("600")),
            value(StatusCode::DECLINE, tag("603")),
            value(StatusCode::DOES_NOT_EXIST_ANYWHERE, tag("604")),
            value(StatusCode::NOT_ACCEPTABLE_GLOBAL, tag("606")),
        ))(input)
    }

    #[inline]
    fn extension_code(input: &str) -> ParserResult<&str, StatusCode> {
        recognize(pair(positive_digit, count(digit, 2)))(input).map(|(rest, result)| {
            let status = result.parse::<u16>().unwrap();
            (rest, StatusCode(status))
        })
    }

    pub(crate) fn status_code(input: &str) -> ParserResult<&str, StatusCode> {
        context(
            "status_code",
            alt((
                informational,
                success,
                redirection,
                client_error,
                server_error,
                global_failure,
                extension_code,
            )),
        )(input)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use claims::{assert_err, assert_ok};

    #[test]
    fn test_status_code_eq() {
        assert_eq!(StatusCode::OK, 200);
        assert_eq!(200, StatusCode::OK);

        assert_eq!(StatusCode::RINGING, StatusCode::from_u16(180).unwrap());
        assert_eq!(StatusCode::from_u16(180).unwrap(), StatusCode::RINGING);
    }

    #[test]
    fn test_valid_status_code_informational() {
        assert!(StatusCode::try_from("180").is_ok_and(|v| v == StatusCode::RINGING));
        assert!(StatusCode::from_u16(182).is_ok_and(|v| v == StatusCode::QUEUED));
    }

    #[test]
    fn test_valid_status_code_success() {
        assert!(StatusCode::try_from("200").is_ok_and(|v| v == StatusCode::OK));
        assert!(StatusCode::from_u16(200).is_ok_and(|v| v == StatusCode::OK));
    }

    #[test]
    fn test_valid_status_code_redirection() {
        assert!(StatusCode::try_from("300").is_ok_and(|v| v == StatusCode::MULTIPLE_CHOICES));
        assert!(StatusCode::from_u16(302).is_ok_and(|v| v == StatusCode::MOVED_TEMPORARILY));
    }

    #[test]
    fn test_valid_status_code_client_error() {
        assert!(StatusCode::try_from("410").is_ok_and(|v| v == StatusCode::GONE));
        assert!(StatusCode::try_from(423).is_ok_and(|v| v == StatusCode::INTERVAL_TOO_BRIEF));
    }

    #[test]
    fn test_valid_status_code_server_error() {
        assert!(StatusCode::try_from("501").is_ok_and(|v| v == StatusCode::NOT_IMPLEMENTED));
        assert!(StatusCode::try_from(513).is_ok_and(|v| v == StatusCode::MESSAGE_TOO_LARGE));
    }

    #[test]
    fn test_valid_status_code_global_failure() {
        assert!(StatusCode::try_from("600").is_ok_and(|v| v == StatusCode::BUSY_EVERYWHERE));
        assert!(StatusCode::try_from(603).is_ok_and(|v| v == StatusCode::DECLINE));
    }

    #[test]
    fn test_valid_status_code_extension() {
        assert_ok!(StatusCode::try_from("829"));
        assert_ok!(StatusCode::try_from(157));
    }

    #[test]
    fn test_invalid_status_code_under_100() {
        assert_err!(StatusCode::try_from("99"));
        assert_err!(StatusCode::from_u16(10));
    }

    #[test]
    fn test_invalid_status_code_over_999() {
        assert_err!(StatusCode::from_u16(3478));
        assert_err!(StatusCode::try_from("9273"));
        assert_err!(StatusCode::try_from("4629"));
    }

    #[test]
    fn test_invalid_status_code_0() {
        assert_err!(StatusCode::from_u16(0));
        assert_err!(StatusCode::try_from("000"));
    }

    #[test]
    fn test_invalid_status_code_not_a_number() {
        assert_err!(StatusCode::try_from("bob"));
    }

    #[test]
    fn test_valid_status_code_but_with_remaining_data() {
        assert!(StatusCode::try_from("200 anything")
            .is_err_and(|e| e == Error::RemainingUnparsedData(" anything".to_string())));
    }
}
