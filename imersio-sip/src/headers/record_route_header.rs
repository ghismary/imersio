//! SIP Record-Route header parsing and generation.

use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;
use partial_eq_refs::PartialEqRefs;

use crate::headers::{GenericHeader, HeaderAccessor};
use crate::{Route, Routes};

/// Representation of a Record-Route header.
///
/// The Record-Route header field is inserted by proxies in a request to force future requests in
/// the dialog to be routed through the proxy.
///
/// [[RFC3261, Section 20.30](https://datatracker.ietf.org/doc/html/rfc3261#section-20.30)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras, PartialEqRefs)]
#[display(fmt = "{}", header)]
pub struct RecordRouteHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    routes: Routes,
}

impl RecordRouteHeader {
    pub(crate) fn new(header: GenericHeader, routes: Vec<Route>) -> Self {
        Self {
            header,
            routes: routes.into(),
        }
    }

    /// Get a reference to the routes from the Record-Route header.
    pub fn routes(&self) -> &Routes {
        &self.routes
    }
}

impl HeaderAccessor for RecordRouteHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Record-Route")
    }
    fn normalized_value(&self) -> String {
        self.routes.to_string()
    }
}

pub(crate) mod parser {
    use crate::common::route::parser::route;
    use crate::headers::GenericHeader;
    use crate::parser::{comma, hcolon, ParserResult};
    use crate::{Header, RecordRouteHeader};
    use nom::{
        bytes::complete::tag_no_case,
        combinator::{consumed, cut, map},
        error::context,
        multi::separated_list1,
        sequence::tuple,
    };

    pub(crate) fn record_route(input: &str) -> ParserResult<&str, Header> {
        context(
            "Record-Route header",
            map(
                tuple((
                    tag_no_case("Record-Route"),
                    hcolon,
                    cut(consumed(separated_list1(comma, route))),
                )),
                |(name, separator, (value, routes))| {
                    Header::RecordRoute(RecordRouteHeader::new(
                        GenericHeader::new(name, separator, value),
                        routes,
                    ))
                },
            ),
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        headers::{
            tests::{header_equality, header_inequality, invalid_header, valid_header},
            HeaderAccessor,
        },
        Header, RecordRouteHeader, Uri,
    };
    use claims::assert_ok;

    valid_header!(RecordRoute, RecordRouteHeader, "Record-Route");
    header_equality!(RecordRoute, "Record-Route");
    header_inequality!(RecordRoute, "Record-Route");

    #[test]
    fn test_valid_record_route_header() {
        valid_header(
            r#"Record-Route: <sip:server10.biloxi.com;lr>, <sip:bigbox3.site3.atlanta.com;lr>"#,
            |header| {
                assert_eq!(header.routes().len(), 2);
                let mut routes = header.routes().iter();
                let first_route = routes.next().unwrap();
                assert_eq!(first_route.name_address().display_name(), None);
                assert_eq!(
                    first_route.name_address().uri(),
                    Uri::try_from("sip:server10.biloxi.com;lr").unwrap()
                );
                let second_route = routes.next().unwrap();
                assert_eq!(second_route.name_address().display_name(), None);
                assert_eq!(
                    second_route.name_address().uri(),
                    Uri::try_from("sip:bigbox3.site3.atlanta.com;lr").unwrap()
                );
            },
        );
    }

    #[test]
    fn test_invalid_record_route_header_empty() {
        invalid_header("Record-Route:");
    }

    #[test]
    fn test_invalid_record_route_header_empty_with_space_characters() {
        invalid_header("Record-Route:    ");
    }

    #[test]
    fn test_invalid_record_route_header_with_invalid_character() {
        invalid_header("Record-Route: 😁");
    }

    #[test]
    fn test_record_route_header_equality_same_header_with_space_characters_differences() {
        header_equality(
            r#"Record-Route: <sip:server10.biloxi.com;lr>"#,
            r#"Record-Route:    <sip:server10.biloxi.com;lr>"#,
        );
    }

    #[test]
    fn test_record_route_header_equality_same_header_with_different_cases() {
        header_equality(
            r#"Record-Route: <sip:bigbox3.site3.atlanta.com;lr>"#,
            r#"Record-Route: <SIP:bigbox3.site3.atlanta.com;LR>"#,
        );
    }

    #[test]
    fn test_record_route_header_inequality_different_uris() {
        header_inequality(
            r#"Record-Route: <sip:server10.biloxi.com;lr>"#,
            r#"Record-Route: <sip:bigbox3.site3.atlanta.com;lr>"#,
        );
    }

    #[test]
    fn test_record_route_header_inequality_with_first_having_more_uris_than_the_second() {
        header_inequality(
            r#"Record-Route: <sip:server10.biloxi.com;lr>, <sip:bigbox3.site3.atlanta.com;lr>"#,
            r#"Record-Route: <sip:server10.biloxi.com;lr>"#,
        );
    }

    #[test]
    fn test_record_route_header_inequality_with_first_having_less_uris_than_the_second() {
        header_inequality(
            r#"Record-Route: <sip:server10.biloxi.com;lr>"#,
            r#"Record-Route: <sip:server10.biloxi.com;lr>, <sip:bigbox3.site3.atlanta.com;lr>"#,
        );
    }

    #[test]
    fn test_record_route_header_to_string() {
        let header = Header::try_from(r#"record-route :    <Sip:bigbox3.site3.atlanta.com;LR>"#);
        if let Header::From(header) = header.unwrap() {
            assert_eq!(
                header.to_string(),
                r#"record-route :    <Sip:bigbox3.site3.atlanta.com;LR>"#
            );
            assert_eq!(
                header.to_normalized_string(),
                r#"Record-Route: <sip:bigbox3.site3.atlanta.com;lr>"#
            );
            assert_eq!(
                header.to_compact_string(),
                r#"Record-Route: <sip:bigbox3.site3.atlanta.com;lr>"#
            );
        }
    }
}
