use crate::{Request, Response, SipError};
use nom_language::error::convert_error;
use std::str::from_utf8;

/// Representation of a SIP message (either a request or a response).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Message {
    /// A SIP request.
    Request(Request),
    /// A SIP response.
    Response(Response),
}

impl Message {
    fn set_body(&mut self, body: &[u8]) {
        match self {
            Self::Request(request) => request.set_body(body),
            Self::Response(response) => response.set_body(body),
        }
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Request(request) => request.to_string(),
                Self::Response(response) => response.to_string(),
            }
        )
    }
}

impl TryFrom<&[u8]> for Message {
    type Error = SipError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match parser::sip_message_raw(value) {
            Ok((body, message_head)) => match from_utf8(message_head) {
                Ok(message_head) => match parser::sip_message(message_head) {
                    Ok((rest, mut message)) => {
                        if !rest.is_empty() {
                            Err(SipError::RemainingUnparsedData(rest.to_string()))
                        } else {
                            message.set_body(body);
                            Ok(message)
                        }
                    }
                    Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                        Err(SipError::InvalidResponse(convert_error(message_head, e)))
                    }
                    Err(nom::Err::Incomplete(_)) => Err(SipError::InvalidResponse(format!(
                        "Incomplete message `{}`",
                        message_head
                    ))),
                },
                Err(_) => Err(SipError::InvalidMessage(format!(
                    "Invalid message head is not UTF-8 encoded `{:?}`",
                    value
                ))),
            },
            Err(nom::Err::Error(_)) | Err(nom::Err::Failure(_)) => Err(SipError::InvalidMessage(
                format!("Invalid message `{:?}`", value),
            )),
            Err(nom::Err::Incomplete(_)) => Err(SipError::InvalidMessage(format!(
                "Incomplete message `{:?}`",
                value
            ))),
        }
    }
}

mod parser {
    use nom::{
        branch::alt,
        bytes::complete::{tag, take_until1},
        combinator::{map, recognize},
        sequence::pair,
        Parser,
    };

    use crate::{
        messages::{request::parser::request, response::parser::response},
        parser::ParserResult,
        Message,
    };

    pub(super) fn sip_message_raw(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
        recognize(pair(take_until1("\r\n\r\n"), tag("\r\n\r\n"))).parse(input)
    }

    pub(super) fn sip_message(input: &str) -> ParserResult<&str, Message> {
        alt((
            map(request, Message::Request),
            map(response, Message::Response),
        ))
        .parse(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::common::wrapped_string::WrappedString;
    use crate::{
        Header, Host, HostnameString, MediaRange, Message, Method, Methods, StatusCode,
        TokenString, Transport, Uri, Version,
    };
    use chrono::{TimeDelta, TimeZone, Utc};
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_valid_options_request() {
        let message = Message::try_from(
            b"\
OPTIONS sip:carol@chicago.com SIP/2.0\r\n\
Via: SIP/2.0/UDP pc33.atlanta.com;branch=z9hG4bKhjhs8ass877\r\n\
Max-Forwards: 70\r\n\
To: <sip:carol@chicago.com>\r\n\
From: Alice <sip:alice@atlanta.com>;tag=1928301774\r\n\
Call-ID: a84b4c76e66710\r\n\
CSeq: 63104 OPTIONS\r\n\
Contact: <sip:alice@pc33.atlanta.com>\r\n\
Accept: application/sdp\r\n\
Content-Length: 0\r\n\
\r\n"
                .as_slice(),
        );
        assert!(message.is_ok());
        let message = message.unwrap();
        match message {
            Message::Request(request) => {
                assert_eq!(request.method(), &Method::Options);
                assert_eq!(request.version(), &Version::Sip2);
                assert_eq!(
                    request.uri(),
                    Uri::try_from("sip:carol@chicago.com").unwrap()
                );
                assert_eq!(request.headers().len(), 9);
                let mut it = request.headers().iter();
                let header = it.next().unwrap();
                match header {
                    Header::Via(via_header) => {
                        assert_eq!(via_header.vias().len(), 1);
                        let via = via_header.vias().first().unwrap();
                        assert_eq!(via.protocol().name(), "SIP");
                        assert_eq!(via.protocol().version(), "2.0");
                        assert_eq!(via.protocol().transport(), &Transport::Udp);
                        assert_eq!(
                            via.host(),
                            &Host::Name(HostnameString::try_from("pc33.atlanta.com").unwrap())
                        );
                        assert_eq!(via.port(), None);
                        assert_eq!(via.parameters().len(), 1);
                        assert!(via.parameters().first().unwrap().is_branch());
                        assert_eq!(
                            via.parameters().first().unwrap().branch(),
                            Some("z9hG4bKhjhs8ass877".to_string())
                        );
                    }
                    _ => panic!("Should be a Via header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::MaxForwards(max_forwards_header) => {
                        assert_eq!(max_forwards_header.max_forwards(), 70);
                    }
                    _ => panic!("Should be a Max-Forwards header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::To(to_header) => {
                        assert_eq!(to_header.address().display_name(), None);
                        assert_eq!(
                            to_header.address().uri(),
                            Uri::try_from("sip:carol@chicago.com").unwrap()
                        );
                        assert_eq!(to_header.parameters().len(), 0);
                    }
                    _ => panic!("Should be a To header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::From(from_header) => {
                        assert_eq!(from_header.address().display_name(), Some("Alice"));
                        assert_eq!(
                            from_header.address().uri(),
                            Uri::try_from("sip:alice@atlanta.com").unwrap()
                        );
                        assert_eq!(from_header.parameters().len(), 1);
                        assert_eq!(from_header.tag(), Some("1928301774"));
                    }
                    _ => panic!("Should be a From header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::CallId(call_id_header) => {
                        assert_eq!(call_id_header.call_id(), "a84b4c76e66710");
                    }
                    _ => panic!("Should be Call-ID header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::CSeq(cseq_header) => {
                        assert_eq!(cseq_header.cseq(), 63104);
                        assert_eq!(cseq_header.method(), &Method::Options);
                    }
                    _ => panic!("Should be a CSeq header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::Contact(contact_header) => {
                        assert_eq!(contact_header.contacts().len(), 1);
                        let contact = contact_header.contacts().first().unwrap();
                        assert_eq!(contact.address().display_name(), None);
                        assert_eq!(
                            contact.address().uri(),
                            Uri::try_from("sip:alice@pc33.atlanta.com").unwrap()
                        );
                        assert_eq!(contact.parameters().len(), 0);
                    }
                    _ => panic!("Should be a Contact header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::Accept(accept_header) => {
                        assert_eq!(accept_header.ranges().len(), 1);
                        let range = accept_header.ranges().first().unwrap();
                        assert_eq!(
                            range.media_range(),
                            &MediaRange::new(
                                TokenString::new("application"),
                                TokenString::new("sdp")
                            )
                        );
                        assert_eq!(range.parameters().len(), 0);
                    }
                    _ => panic!("Should be an Accept header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::ContentLength(content_length_header) => {
                        assert_eq!(content_length_header.content_length(), 0);
                    }
                    _ => panic!("Should be a Content-Length header!"),
                }
                assert_eq!(it.next(), None);
                assert_eq!(request.body().len(), 0);
            }
            _ => panic!("Should be a request!"),
        }
    }

    #[test]
    fn test_valid_200_response_to_options_request() {
        let message = Message::try_from(
            b"\
SIP/2.0 200 OK\r\n\
Via: SIP/2.0/UDP pc33.atlanta.com;branch=z9hG4bKhjhs8ass877;received=192.0.2.4\r\n\
To: <sip:carol@chicago.com>;tag=93810874\r\n\
From: Alice <sip:alice@atlanta.com>;tag=1928301774\r\n\
Call-ID: a84b4c76e66710\r\n\
CSeq: 63104 OPTIONS\r\n\
Contact: <sip:carol@chicago.com>\r\n\
Contact: <mailto:carol@chicago.com>\r\n\
Allow: INVITE, ACK, CANCEL, OPTIONS, BYE\r\n\
Accept: application/sdp\r\n\
Accept-Encoding: gzip\r\n\
Accept-Language: en\r\n\
Supported: foo\r\n\
Content-Type: application/sdp\r\n\
Content-Length: 274\r\n\
\r\n"
                .as_slice(),
        );
        assert!(message.is_ok());
        let message = message.unwrap();
        match message {
            Message::Response(response) => {
                assert_eq!(response.version(), &Version::Sip2);
                assert_eq!(response.reason().status(), StatusCode::OK);
                assert_eq!(response.reason().phrase(), "OK");
                assert_eq!(response.headers().len(), 14);
                let mut it = response.headers().iter();
                let header = it.next().unwrap();
                match header {
                    Header::Via(via_header) => {
                        assert_eq!(via_header.vias().len(), 1);
                        let via = via_header.vias().first().unwrap();
                        assert_eq!(via.protocol().name(), "SIP");
                        assert_eq!(via.protocol().version(), "2.0");
                        assert_eq!(via.protocol().transport(), &Transport::Udp);
                        assert_eq!(
                            via.host(),
                            &Host::Name(HostnameString::try_from("pc33.atlanta.com").unwrap())
                        );
                        assert_eq!(via.port(), None);
                        assert_eq!(via.parameters().len(), 2);
                        assert!(via.parameters().first().unwrap().is_branch());
                        assert_eq!(
                            via.parameters().first().unwrap().branch(),
                            Some("z9hG4bKhjhs8ass877".to_string())
                        );
                        assert!(via.parameters().last().unwrap().is_received());
                        assert_eq!(
                            via.parameters().last().unwrap().received(),
                            Some(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 4)))
                        );
                    }
                    _ => panic!("Should be a Via header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::To(to_header) => {
                        assert_eq!(to_header.address().display_name(), None);
                        assert_eq!(
                            to_header.address().uri(),
                            Uri::try_from("sip:carol@chicago.com").unwrap()
                        );
                        assert_eq!(to_header.parameters().len(), 1);
                        assert!(to_header.parameters().first().unwrap().is_tag());
                        assert_eq!(
                            to_header.parameters().first().unwrap().tag(),
                            Some("93810874")
                        );
                    }
                    _ => panic!("Should be a To header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::From(from_header) => {
                        assert_eq!(from_header.address().display_name(), Some("Alice"));
                        assert_eq!(
                            from_header.address().uri(),
                            Uri::try_from("sip:alice@atlanta.com").unwrap()
                        );
                        assert_eq!(from_header.parameters().len(), 1);
                        assert_eq!(from_header.tag(), Some("1928301774"));
                    }
                    _ => panic!("Should be a From header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::CallId(call_id_header) => {
                        assert_eq!(call_id_header.call_id(), "a84b4c76e66710");
                    }
                    _ => panic!("Should be Call-ID header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::CSeq(cseq_header) => {
                        assert_eq!(cseq_header.cseq(), 63104);
                        assert_eq!(cseq_header.method(), &Method::Options);
                    }
                    _ => panic!("Should be a CSeq header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::Contact(contact_header) => {
                        assert_eq!(contact_header.contacts().len(), 1);
                        let contact = contact_header.contacts().first().unwrap();
                        assert_eq!(contact.address().display_name(), None);
                        assert_eq!(
                            contact.address().uri(),
                            Uri::try_from("sip:carol@chicago.com").unwrap()
                        );
                        assert_eq!(contact.parameters().len(), 0);
                    }
                    _ => panic!("Should be a Contact header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::Contact(contact_header) => {
                        assert_eq!(contact_header.contacts().len(), 1);
                        let contact = contact_header.contacts().first().unwrap();
                        assert_eq!(contact.address().display_name(), None);
                        assert_eq!(
                            contact.address().uri(),
                            Uri::try_from("mailto:carol@chicago.com").unwrap()
                        );
                        assert_eq!(contact.parameters().len(), 0);
                    }
                    _ => panic!("Should be a Contact header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::Allow(allow_header) => {
                        let expected_methods: Methods = vec![
                            Method::Invite,
                            Method::Ack,
                            Method::Cancel,
                            Method::Options,
                            Method::Bye,
                        ]
                        .into();
                        assert_eq!(allow_header.methods(), &expected_methods);
                    }
                    _ => panic!("Should be an Allow header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::Accept(accept_header) => {
                        assert_eq!(accept_header.ranges().len(), 1);
                        let range = accept_header.ranges().first().unwrap();
                        assert_eq!(
                            range.media_range(),
                            &MediaRange::new(
                                TokenString::new("application"),
                                TokenString::new("sdp")
                            )
                        );
                        assert_eq!(range.parameters().len(), 0);
                    }
                    _ => panic!("Should be an Accept header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::AcceptEncoding(accept_encoding_header) => {
                        assert_eq!(accept_encoding_header.encodings().len(), 1);
                        let encoding = accept_encoding_header.encodings().first().unwrap();
                        assert_eq!(encoding.encoding(), "gzip");
                        assert_eq!(encoding.parameters().len(), 0);
                    }
                    _ => panic!("Should be an Accept-Encoding header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::AcceptLanguage(accept_language_header) => {
                        assert_eq!(accept_language_header.languages().len(), 1);
                        let language = accept_language_header.languages().first().unwrap();
                        assert_eq!(language.language(), "en");
                        assert_eq!(language.parameters().len(), 0);
                    }
                    _ => panic!("Should be an Accept-Language header"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::Supported(supported_header) => {
                        assert_eq!(supported_header.option_tags().len(), 1);
                        let option_tag = supported_header.option_tags().first().unwrap();
                        assert_eq!(option_tag.value(), "foo");
                    }
                    _ => panic!("Should be a Supported header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::ContentType(content_type_header) => {
                        assert_eq!(
                            content_type_header.media_type().media_range(),
                            &MediaRange::new(
                                TokenString::new("application"),
                                TokenString::new("sdp")
                            )
                        );
                        assert_eq!(content_type_header.media_type().parameters().len(), 0);
                    }
                    _ => panic!("Should be an Accept header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::ContentLength(content_length_header) => {
                        assert_eq!(content_length_header.content_length(), 274);
                    }
                    _ => panic!("Should be a Content-Length header!"),
                }
                assert_eq!(it.next(), None);
            }
            _ => panic!("Should be a response!"),
        }
    }

    #[test]
    fn test_valid_invite_request() {
        let message = Message::try_from(
            b"\
INVITE sip:bob@biloxi.com SIP/2.0\r\n\
Via: SIP/2.0/UDP pc33.atlanta.com;branch=z9hG4bKkjshdyff\r\n\
To: Bob <sip:bob@biloxi.com>\r\n\
From: Alice <sip:alice@atlanta.com>;tag=88sja8x\r\n\
Max-Forwards: 70\r\n\
Call-ID: 987asjd97y7atg\r\n\
CSeq: 986759 INVITE\r\n\
\r\n"
                .as_slice(),
        );
        assert!(message.is_ok());
        let message = message.unwrap();
        match message {
            Message::Request(request) => {
                assert_eq!(request.method(), &Method::Invite);
                assert_eq!(request.version(), &Version::Sip2);
                assert_eq!(request.uri(), Uri::try_from("sip:bob@biloxi.com").unwrap());
                assert_eq!(request.headers().len(), 6);
                let mut it = request.headers().iter();
                let header = it.next().unwrap();
                match header {
                    Header::Via(via_header) => {
                        assert_eq!(via_header.vias().len(), 1);
                        let via = via_header.vias().first().unwrap();
                        assert_eq!(via.protocol().name(), "SIP");
                        assert_eq!(via.protocol().version(), "2.0");
                        assert_eq!(via.protocol().transport(), &Transport::Udp);
                        assert_eq!(
                            via.host(),
                            &Host::Name(HostnameString::try_from("pc33.atlanta.com").unwrap())
                        );
                        assert_eq!(via.port(), None);
                        assert_eq!(via.parameters().len(), 1);
                        assert!(via.parameters().first().unwrap().is_branch());
                        assert_eq!(
                            via.parameters().first().unwrap().branch(),
                            Some("z9hG4bKkjshdyff".to_string())
                        );
                    }
                    _ => panic!("Should be a Via header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::To(to_header) => {
                        assert_eq!(to_header.address().display_name(), Some("Bob"));
                        assert_eq!(
                            to_header.address().uri(),
                            Uri::try_from("sip:bob@biloxi.com").unwrap()
                        );
                        assert_eq!(to_header.parameters().len(), 0);
                    }
                    _ => panic!("Should be a To header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::From(from_header) => {
                        assert_eq!(from_header.address().display_name(), Some("Alice"));
                        assert_eq!(
                            from_header.address().uri(),
                            Uri::try_from("sip:alice@atlanta.com").unwrap()
                        );
                        assert_eq!(from_header.parameters().len(), 1);
                        assert_eq!(from_header.tag(), Some("88sja8x"));
                    }
                    _ => panic!("Should be a From header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::MaxForwards(max_forwards_header) => {
                        assert_eq!(max_forwards_header.max_forwards(), 70);
                    }
                    _ => panic!("Should be a Max-Forwards header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::CallId(call_id_header) => {
                        assert_eq!(call_id_header.call_id(), "987asjd97y7atg");
                    }
                    _ => panic!("Should be Call-ID header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::CSeq(cseq_header) => {
                        assert_eq!(cseq_header.cseq(), 986759);
                        assert_eq!(cseq_header.method(), &Method::Invite);
                    }
                    _ => panic!("Should be a CSeq header!"),
                }
                assert_eq!(it.next(), None);
                assert_eq!(request.body().len(), 0);
            }
            _ => panic!("Should be a request!"),
        }
    }

    #[test]
    fn test_valid_ack_request_for_non_2xx_response_to_invite_request() {
        let message = Message::try_from(
            b"\
ACK sip:bob@biloxi.com SIP/2.0\r\n\
Via: SIP/2.0/UDP pc33.atlanta.com;branch=z9hG4bKkjshdyff\r\n\
To: Bob <sip:bob@biloxi.com>;tag=99sa0xk\r\n\
From: Alice <sip:alice@atlanta.com>;tag=88sja8x\r\n\
Max-Forwards: 70\r\n\
Call-ID: 987asjd97y7atg\r\n\
CSeq: 986759 ACK\r\n\
\r\n"
                .as_slice(),
        );
        assert!(message.is_ok());
        let message = message.unwrap();
        match message {
            Message::Request(request) => {
                assert_eq!(request.method(), &Method::Ack);
                assert_eq!(request.version(), &Version::Sip2);
                assert_eq!(request.uri(), Uri::try_from("sip:bob@biloxi.com").unwrap());
                assert_eq!(request.headers().len(), 6);
                let mut it = request.headers().iter();
                let header = it.next().unwrap();
                match header {
                    Header::Via(via_header) => {
                        assert_eq!(via_header.vias().len(), 1);
                        let via = via_header.vias().first().unwrap();
                        assert_eq!(via.protocol().name(), "SIP");
                        assert_eq!(via.protocol().version(), "2.0");
                        assert_eq!(via.protocol().transport(), &Transport::Udp);
                        assert_eq!(
                            via.host(),
                            &Host::Name(HostnameString::try_from("pc33.atlanta.com").unwrap())
                        );
                        assert_eq!(via.port(), None);
                        assert_eq!(via.parameters().len(), 1);
                        assert!(via.parameters().first().unwrap().is_branch());
                        assert_eq!(
                            via.parameters().first().unwrap().branch(),
                            Some("z9hG4bKkjshdyff".to_string())
                        );
                    }
                    _ => panic!("Should be a Via header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::To(to_header) => {
                        assert_eq!(to_header.address().display_name(), Some("Bob"));
                        assert_eq!(
                            to_header.address().uri(),
                            Uri::try_from("sip:bob@biloxi.com").unwrap()
                        );
                        assert_eq!(to_header.parameters().len(), 1);
                        assert!(to_header.parameters().first().unwrap().is_tag());
                        assert_eq!(
                            to_header.parameters().first().unwrap().tag(),
                            Some("99sa0xk")
                        )
                    }
                    _ => panic!("Should be a To header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::From(from_header) => {
                        assert_eq!(from_header.address().display_name(), Some("Alice"));
                        assert_eq!(
                            from_header.address().uri(),
                            Uri::try_from("sip:alice@atlanta.com").unwrap()
                        );
                        assert_eq!(from_header.parameters().len(), 1);
                        assert_eq!(from_header.tag(), Some("88sja8x"));
                    }
                    _ => panic!("Should be a From header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::MaxForwards(max_forwards_header) => {
                        assert_eq!(max_forwards_header.max_forwards(), 70);
                    }
                    _ => panic!("Should be a Max-Forwards header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::CallId(call_id_header) => {
                        assert_eq!(call_id_header.call_id(), "987asjd97y7atg");
                    }
                    _ => panic!("Should be Call-ID header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::CSeq(cseq_header) => {
                        assert_eq!(cseq_header.cseq(), 986759);
                        assert_eq!(cseq_header.method(), &Method::Ack);
                    }
                    _ => panic!("Should be a CSeq header!"),
                }
                assert_eq!(it.next(), None);
                assert_eq!(request.body().len(), 0);
            }
            _ => panic!("Should be a request!"),
        }
    }

    #[test]
    fn test_valid_tunneled_invite_request() {
        let message = Message::try_from(
            b"\
INVITE sip:bob@biloxi.com SIP/2.0\r\n\
Via: SIP/2.0/UDP pc33.atlanta.com;branch=z9hG4bKnashds8\r\n\
To: Bob <sip:bob@biloxi.com>\r\n\
From: Alice <sip:alice@atlanta.com>;tag=1928301774\r\n\
Call-ID: a84b4c76e66710\r\n\
CSeq: 314159 INVITE\r\n\
Max-Forwards: 70\r\n\
Date: Thu, 21 Feb 2002 13:02:03 GMT\r\n\
Contact: <sip:alice@pc33.atlanta.com>\r\n\
Content-Type: multipart/signed;protocol=\"application/pkcs7-signature\"; micalg=sha1; boundary=boundary42\r\n\
Content-Length: 944\r\n\
\r\n\
--boundary42\r\n\
Content-Type: message/sip\r\n\
\r\n\
INVITE sip:bob@biloxi.com SIP/2.0\r\n\
Via: SIP/2.0/UDP pc33.atlanta.com;branch=z9hG4bKnashds8\r\n\
To: Bob <bob@biloxi.com>\r\n\
From: Alice <alice@atlanta.com>;tag=1928301774\r\n\
Call-ID: a84b4c76e66710\r\n\
CSeq: 314159 INVITE\r\n\
Max-Forwards: 70\r\n\
Date: Thu, 21 Feb 2002 13:02:03 GMT\r\n\
Contact: <sip:alice@pc33.atlanta.com>\r\n\
Content-Type: application/sdp\r\n\
Content-Length: 147\r\n\
\r\n\
v=0\r\n\
o=UserA 2890844526 2890844526 IN IP4 here.com\r\n\
s=Session SDP\r\n\
c=IN IP4 pc33.atlanta.com\r\n\
t=0 0\r\n\
m=audio 49172 RTP/AVP 0\r\n\
a=rtpmap:0 PCMU/8000\r\n\
\r\n\
--boundary42\r\n\
Content-Type: application/pkcs7-signature; name=smime.p7s\r\n\
Content-Transfer-Encoding: base64\r\n\
Content-Disposition: attachment; filename=smime.p7s;handling=required\r\n\
\r\n\
ghyHhHUujhJhjH77n8HHGTrfvbnj756tbB9HG4VQpfyF467GhIGfHfYT64VQpfyF467GhIGfHfYT6jH77n8HHGghyHhHUujhJh756tbB9HGTrfvbnjn8HHGTrfvhJhjH776tbB9HG4VQbnj7567GhIGfHfYT6ghyHhHUujpfyF47GhIGfHfYT64VQbnj756\r\n\
\r\n\
--boundary42-\r\n"
                .as_slice(),
        );
        assert!(message.is_ok());
        let message = message.unwrap();
        match message {
            Message::Request(request) => {
                assert_eq!(request.method(), &Method::Invite);
                assert_eq!(request.version(), &Version::Sip2);
                assert_eq!(request.uri(), Uri::try_from("sip:bob@biloxi.com").unwrap());
                assert_eq!(request.headers().len(), 10);
                let mut it = request.headers().iter();
                let header = it.next().unwrap();
                match header {
                    Header::Via(via_header) => {
                        assert_eq!(via_header.vias().len(), 1);
                        let via = via_header.vias().first().unwrap();
                        assert_eq!(via.protocol().name(), "SIP");
                        assert_eq!(via.protocol().version(), "2.0");
                        assert_eq!(via.protocol().transport(), &Transport::Udp);
                        assert_eq!(
                            via.host(),
                            &Host::Name(HostnameString::try_from("pc33.atlanta.com").unwrap())
                        );
                        assert_eq!(via.port(), None);
                        assert_eq!(via.parameters().len(), 1);
                        assert!(via.parameters().first().unwrap().is_branch());
                        assert_eq!(
                            via.parameters().first().unwrap().branch(),
                            Some("z9hG4bKnashds8".to_string())
                        );
                    }
                    _ => panic!("Should be a Via header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::To(to_header) => {
                        assert_eq!(to_header.address().display_name(), Some("Bob"));
                        assert_eq!(
                            to_header.address().uri(),
                            Uri::try_from("sip:bob@biloxi.com").unwrap()
                        );
                        assert_eq!(to_header.parameters().len(), 0);
                    }
                    _ => panic!("Should be a To header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::From(from_header) => {
                        assert_eq!(from_header.address().display_name(), Some("Alice"));
                        assert_eq!(
                            from_header.address().uri(),
                            Uri::try_from("sip:alice@atlanta.com").unwrap()
                        );
                        assert_eq!(from_header.parameters().len(), 1);
                        assert_eq!(from_header.tag(), Some("1928301774"));
                    }
                    _ => panic!("Should be a From header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::CallId(call_id_header) => {
                        assert_eq!(call_id_header.call_id(), "a84b4c76e66710");
                    }
                    _ => panic!("Should be Call-ID header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::CSeq(cseq_header) => {
                        assert_eq!(cseq_header.cseq(), 314159);
                        assert_eq!(cseq_header.method(), &Method::Invite);
                    }
                    _ => panic!("Should be a CSeq header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::MaxForwards(max_forwards_header) => {
                        assert_eq!(max_forwards_header.max_forwards(), 70);
                    }
                    _ => panic!("Should be a Max-Forwards header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::Date(date_header) => {
                        assert_eq!(
                            date_header.datetime(),
                            &Utc.with_ymd_and_hms(2002, 2, 21, 13, 2, 3).unwrap()
                        );
                    }
                    _ => panic!("Should be a Date header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::Contact(contact_header) => {
                        assert_eq!(contact_header.contacts().len(), 1);
                        let contact = contact_header.contacts().first().unwrap();
                        assert_eq!(contact.address().display_name(), None);
                        assert_eq!(
                            contact.address().uri(),
                            Uri::try_from("sip:alice@pc33.atlanta.com").unwrap()
                        );
                        assert_eq!(contact.parameters().len(), 0);
                    }
                    _ => panic!("Should be a Contact header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::ContentType(content_type_header) => {
                        assert_eq!(
                            content_type_header.media_type().media_range(),
                            &MediaRange::new(
                                TokenString::new("multipart"),
                                TokenString::new("signed")
                            )
                        );
                        let params = content_type_header.media_type().parameters();
                        assert_eq!(params.len(), 3);
                        let protocol_param = params.first().unwrap();
                        assert_eq!(protocol_param.key(), "protocol");
                        assert_eq!(
                            protocol_param.value(),
                            &WrappedString::Quoted("application/pkcs7-signature".to_string())
                        );
                        let micalg_param = params.get(1).unwrap();
                        assert_eq!(micalg_param.key(), "micalg");
                        assert_eq!(
                            micalg_param.value(),
                            &WrappedString::NotWrapped(TokenString::new("sha1"))
                        );
                        let boundary_param = params.last().unwrap();
                        assert_eq!(boundary_param.key(), "boundary");
                        assert_eq!(
                            boundary_param.value(),
                            &WrappedString::NotWrapped(TokenString::new("boundary42"))
                        );
                    }
                    _ => panic!("Should be an Accept header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::ContentLength(content_length_header) => {
                        assert_eq!(content_length_header.content_length(), 944);
                    }
                    _ => panic!("Should be a Content-Length header!"),
                }
                assert_eq!(it.next(), None);
                assert_eq!(request.body().len(), 944);
            }
            _ => panic!("Should be a request!"),
        }
    }

    #[test]
    fn test_valid_register_request() {
        let message = Message::try_from(
            b"\
REGISTER sip:registrar.biloxi.com SIP/2.0\r\n\
Via: SIP/2.0/UDP bobspc.biloxi.com:5060;branch=z9hG4bKnashds7\r\n\
Max-Forwards: 70\r\n\
To: Bob <sip:bob@biloxi.com>\r\n\
From: Bob <sip:bob@biloxi.com>;tag=456248\r\n\
Call-ID: 843817637684230@998sdasdh09\r\n\
CSeq: 1826 REGISTER\r\n\
Contact: <sip:bob@192.0.2.4>\r\n\
Expires: 7200\r\n\
Content-Length: 0\r\n\
\r\n"
                .as_slice(),
        );
        assert!(message.is_ok());
        let message = message.unwrap();
        match message {
            Message::Request(request) => {
                assert_eq!(request.method(), &Method::Register);
                assert_eq!(request.version(), &Version::Sip2);
                assert_eq!(
                    request.uri(),
                    Uri::try_from("sip:registrar.biloxi.com").unwrap()
                );
                assert_eq!(request.headers().len(), 9);
                let mut it = request.headers().iter();
                let header = it.next().unwrap();
                match header {
                    Header::Via(via_header) => {
                        assert_eq!(via_header.vias().len(), 1);
                        let via = via_header.vias().first().unwrap();
                        assert_eq!(via.protocol().name(), "SIP");
                        assert_eq!(via.protocol().version(), "2.0");
                        assert_eq!(via.protocol().transport(), &Transport::Udp);
                        assert_eq!(
                            via.host(),
                            &Host::Name(HostnameString::try_from("bobspc.biloxi.com").unwrap())
                        );
                        assert_eq!(via.port(), Some(5060));
                        assert_eq!(via.parameters().len(), 1);
                        assert!(via.parameters().first().unwrap().is_branch());
                        assert_eq!(
                            via.parameters().first().unwrap().branch(),
                            Some("z9hG4bKnashds7".to_string())
                        );
                    }
                    _ => panic!("Should be a Via header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::MaxForwards(max_forwards_header) => {
                        assert_eq!(max_forwards_header.max_forwards(), 70);
                    }
                    _ => panic!("Should be a Max-Forwards header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::To(to_header) => {
                        assert_eq!(to_header.address().display_name(), Some("Bob"));
                        assert_eq!(
                            to_header.address().uri(),
                            Uri::try_from("sip:bob@biloxi.com").unwrap()
                        );
                        assert_eq!(to_header.parameters().len(), 0);
                    }
                    _ => panic!("Should be a To header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::From(from_header) => {
                        assert_eq!(from_header.address().display_name(), Some("Bob"));
                        assert_eq!(
                            from_header.address().uri(),
                            Uri::try_from("sip:bob@biloxi.com").unwrap()
                        );
                        assert_eq!(from_header.parameters().len(), 1);
                        assert_eq!(from_header.tag(), Some("456248"));
                    }
                    _ => panic!("Should be a From header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::CallId(call_id_header) => {
                        assert_eq!(call_id_header.call_id(), "843817637684230@998sdasdh09");
                    }
                    _ => panic!("Should be Call-ID header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::CSeq(cseq_header) => {
                        assert_eq!(cseq_header.cseq(), 1826);
                        assert_eq!(cseq_header.method(), &Method::Register);
                    }
                    _ => panic!("Should be a CSeq header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::Contact(contact_header) => {
                        assert_eq!(contact_header.contacts().len(), 1);
                        let contact = contact_header.contacts().first().unwrap();
                        assert_eq!(contact.address().display_name(), None);
                        assert_eq!(
                            contact.address().uri(),
                            Uri::try_from("sip:bob@192.0.2.4").unwrap()
                        );
                        assert_eq!(contact.parameters().len(), 0);
                    }
                    _ => panic!("Should be a Contact header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::Expires(expires_header) => {
                        assert_eq!(expires_header.expires(), TimeDelta::new(7200, 0).unwrap());
                    }
                    _ => panic!("Should be an Expires header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::ContentLength(content_length_header) => {
                        assert_eq!(content_length_header.content_length(), 0);
                    }
                    _ => panic!("Should be a Content-Length header!"),
                }
                assert_eq!(it.next(), None);
                assert_eq!(request.body().len(), 0);
            }
            _ => panic!("Should be a request!"),
        }
    }

    #[test]
    fn test_valid_200_ok_response_to_register_request() {
        let message = Message::try_from(
            b"\
SIP/2.0 200 OK\r\n\
Via: SIP/2.0/UDP bobspc.biloxi.com:5060;branch=z9hG4bKnashds7 ;received=192.0.2.4\r\n\
To: Bob <sip:bob@biloxi.com>;tag=2493k59kd\r\n\
From: Bob <sip:bob@biloxi.com>;tag=456248\r\n\
Call-ID: 843817637684230@998sdasdh09\r\n\
CSeq: 1826 REGISTER\r\n\
Contact: <sip:bob@192.0.2.4>\r\n\
Expires: 7200\r\n\
Content-Length: 0\r\n\
\r\n"
                .as_slice(),
        );
        assert!(message.is_ok());
        let message = message.unwrap();
        match message {
            Message::Response(response) => {
                assert_eq!(response.version(), &Version::Sip2);
                assert_eq!(response.reason().status(), StatusCode::OK);
                assert_eq!(response.reason().phrase(), "OK");
                assert_eq!(response.headers().len(), 8);
                let mut it = response.headers().iter();
                let header = it.next().unwrap();
                match header {
                    Header::Via(via_header) => {
                        assert_eq!(via_header.vias().len(), 1);
                        let via = via_header.vias().first().unwrap();
                        assert_eq!(via.protocol().name(), "SIP");
                        assert_eq!(via.protocol().version(), "2.0");
                        assert_eq!(via.protocol().transport(), &Transport::Udp);
                        assert_eq!(
                            via.host(),
                            &Host::Name(HostnameString::try_from("bobspc.biloxi.com").unwrap())
                        );
                        assert_eq!(via.port(), Some(5060));
                        assert_eq!(via.parameters().len(), 2);
                        assert!(via.parameters().first().unwrap().is_branch());
                        assert_eq!(
                            via.parameters().first().unwrap().branch(),
                            Some("z9hG4bKnashds7".to_string())
                        );
                        assert!(via.parameters().last().unwrap().is_received());
                        assert_eq!(
                            via.parameters().last().unwrap().received(),
                            Some(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 4)))
                        );
                    }
                    _ => panic!("Should be a Via header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::To(to_header) => {
                        assert_eq!(to_header.address().display_name(), Some("Bob"));
                        assert_eq!(
                            to_header.address().uri(),
                            Uri::try_from("sip:bob@biloxi.com").unwrap()
                        );
                        assert_eq!(to_header.parameters().len(), 1);
                        assert!(to_header.parameters().first().unwrap().is_tag());
                        assert_eq!(
                            to_header.parameters().first().unwrap().tag(),
                            Some("2493k59kd")
                        );
                    }
                    _ => panic!("Should be a To header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::From(from_header) => {
                        assert_eq!(from_header.address().display_name(), Some("Bob"));
                        assert_eq!(
                            from_header.address().uri(),
                            Uri::try_from("sip:bob@biloxi.com").unwrap()
                        );
                        assert_eq!(from_header.parameters().len(), 1);
                        assert_eq!(from_header.tag(), Some("456248"));
                    }
                    _ => panic!("Should be a From header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::CallId(call_id_header) => {
                        assert_eq!(call_id_header.call_id(), "843817637684230@998sdasdh09");
                    }
                    _ => panic!("Should be Call-ID header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::CSeq(cseq_header) => {
                        assert_eq!(cseq_header.cseq(), 1826);
                        assert_eq!(cseq_header.method(), &Method::Register);
                    }
                    _ => panic!("Should be a CSeq header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::Contact(contact_header) => {
                        assert_eq!(contact_header.contacts().len(), 1);
                        let contact = contact_header.contacts().first().unwrap();
                        assert_eq!(contact.address().display_name(), None);
                        assert_eq!(
                            contact.address().uri(),
                            Uri::try_from("sip:bob@192.0.2.4").unwrap()
                        );
                        assert_eq!(contact.parameters().len(), 0);
                    }
                    _ => panic!("Should be a Contact header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::Expires(expires_header) => {
                        assert_eq!(expires_header.expires(), TimeDelta::new(7200, 0).unwrap());
                    }
                    _ => panic!("Should be an Expires header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::ContentLength(content_length_header) => {
                        assert_eq!(content_length_header.content_length(), 0);
                    }
                    _ => panic!("Should be a Content-Length header!"),
                }
                assert_eq!(it.next(), None);
            }
            _ => panic!("Should be a response!"),
        }
    }
}
