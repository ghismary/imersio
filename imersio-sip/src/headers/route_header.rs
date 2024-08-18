//! SIP Route header parsing and generation.

use derive_more::Display;
use derive_partial_eq_extras::PartialEqExtras;

use crate::headers::{GenericHeader, HeaderAccessor};
use crate::{Route, Routes};

/// Representation of a Route header.
///
/// The Route header field is used to force routing for a request through the listed set of proxies.
///
/// [[RFC3261, Section 20.34](https://datatracker.ietf.org/doc/html/rfc3261#section-20.34)]
#[derive(Clone, Debug, Display, Eq, PartialEqExtras)]
#[display("{}", header)]
pub struct RouteHeader {
    #[partial_eq_ignore]
    header: GenericHeader,
    routes: Routes,
}

impl RouteHeader {
    pub(crate) fn new(header: GenericHeader, routes: Vec<Route>) -> Self {
        Self {
            header,
            routes: routes.into(),
        }
    }

    /// Get a reference to the routes from the Route header.
    pub fn routes(&self) -> &Routes {
        &self.routes
    }
}

impl HeaderAccessor for RouteHeader {
    crate::headers::generic_header_accessors!(header);

    fn compact_name(&self) -> Option<&str> {
        None
    }
    fn normalized_name(&self) -> Option<&str> {
        Some("Route")
    }
    fn normalized_value(&self) -> String {
        self.routes.to_string()
    }
}

pub(crate) mod parser {
    use crate::common::route::parser::route_spec;
    use crate::headers::GenericHeader;
    use crate::parser::{comma, hcolon, ParserResult};
    use crate::{Header, RouteHeader, TokenString};
    use nom::{
        bytes::complete::tag_no_case,
        combinator::{consumed, cut, map},
        error::context,
        multi::separated_list1,
        sequence::tuple,
    };

    pub(crate) fn route(input: &str) -> ParserResult<&str, Header> {
        context(
            "Route header",
            map(
                tuple((
                    map(tag_no_case("Route"), TokenString::new),
                    hcolon,
                    cut(consumed(separated_list1(comma, route_spec))),
                )),
                |(name, separator, (value, routes))| {
                    Header::Route(RouteHeader::new(
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
        Header, RouteHeader, Uri,
    };
    use claims::assert_ok;

    valid_header!(Route, RouteHeader, "Route");
    header_equality!(Route, "Route");
    header_inequality!(Route, "Route");

    #[test]
    fn test_valid_route_header() {
        valid_header(
            r#"Route: <sip:bigbox3.site3.atlanta.com;lr>, <sip:server10.biloxi.com;lr>"#,
            |header| {
                assert_eq!(header.routes().len(), 2);
                let mut routes = header.routes().iter();
                let first_route = routes.next().unwrap();
                assert_eq!(first_route.name_address().display_name(), None);
                assert_eq!(
                    first_route.name_address().uri(),
                    Uri::try_from("sip:bigbox3.site3.atlanta.com;lr").unwrap()
                );
                let second_route = routes.next().unwrap();
                assert_eq!(second_route.name_address().display_name(), None);
                assert_eq!(
                    second_route.name_address().uri(),
                    Uri::try_from("sip:server10.biloxi.com;lr").unwrap()
                );
            },
        );
    }

    #[test]
    fn test_invalid_route_header_empty() {
        invalid_header("Route:");
    }

    #[test]
    fn test_invalid_route_header_empty_with_space_characters() {
        invalid_header("Route:    ");
    }

    #[test]
    fn test_invalid_route_header_with_invalid_character() {
        invalid_header("Route: 😁");
    }

    #[test]
    fn test_route_header_equality_same_header_with_space_characters_differences() {
        header_equality(
            r#"Route: <sip:server10.biloxi.com;lr>"#,
            r#"Route:    <sip:server10.biloxi.com;lr>"#,
        );
    }

    #[test]
    fn test_route_header_equality_same_header_with_different_cases() {
        header_equality(
            r#"Route: <sip:bigbox3.site3.atlanta.com;lr>"#,
            r#"Route: <SIP:bigbox3.site3.atlanta.com;LR>"#,
        );
    }

    #[test]
    fn test_route_header_inequality_different_uris() {
        header_inequality(
            r#"Route: <sip:server10.biloxi.com;lr>"#,
            r#"Route: <sip:bigbox3.site3.atlanta.com;lr>"#,
        );
    }

    #[test]
    fn test_route_header_inequality_with_first_having_more_uris_than_the_second() {
        header_inequality(
            r#"Route: <sip:server10.biloxi.com;lr>, <sip:bigbox3.site3.atlanta.com;lr>"#,
            r#"Route: <sip:server10.biloxi.com;lr>"#,
        );
    }

    #[test]
    fn test_route_header_inequality_with_first_having_less_uris_than_the_second() {
        header_inequality(
            r#"Route: <sip:server10.biloxi.com;lr>"#,
            r#"Route: <sip:server10.biloxi.com;lr>, <sip:bigbox3.site3.atlanta.com;lr>"#,
        );
    }

    #[test]
    fn test_route_header_to_string() {
        let header = Header::try_from(r#"route :    <Sip:bigbox3.site3.atlanta.com;LR>"#);
        if let Header::From(header) = header.unwrap() {
            assert_eq!(
                header.to_string(),
                r#"route :    <Sip:bigbox3.site3.atlanta.com;LR>"#
            );
            assert_eq!(
                header.to_normalized_string(),
                r#"Route: <sip:bigbox3.site3.atlanta.com;lr>"#
            );
            assert_eq!(
                header.to_compact_string(),
                r#"Route: <sip:bigbox3.site3.atlanta.com;lr>"#
            );
        }
    }
}
