use crate::{Error, Request, Response};
use nom::error::convert_error;
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
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match parser::sip_message_raw(value) {
            Ok((body, message_head)) => match from_utf8(message_head) {
                Ok(message_head) => match parser::sip_message(message_head) {
                    Ok((rest, mut message)) => {
                        if !rest.is_empty() {
                            Err(Error::RemainingUnparsedData(rest.to_string()))
                        } else {
                            message.set_body(body);
                            Ok(message)
                        }
                    }
                    Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                        Err(Error::InvalidResponse(convert_error(message_head, e)))
                    }
                    Err(nom::Err::Incomplete(_)) => Err(Error::InvalidResponse(format!(
                        "Incomplete message `{}`",
                        message_head
                    ))),
                },
                Err(_) => Err(Error::InvalidMessage(format!(
                    "Invalid message head is not UTF-8 encoded `{:?}`",
                    value
                ))),
            },
            Err(nom::Err::Error(_)) | Err(nom::Err::Failure(_)) => Err(Error::InvalidMessage(
                format!("Invalid message `{:?}`", value),
            )),
            Err(nom::Err::Incomplete(_)) => Err(Error::InvalidMessage(format!(
                "Incomplete message `{:?}`",
                value
            ))),
        }
    }
}

mod parser {
    use crate::messages::request::parser::request;
    use crate::messages::response::parser::response;
    use crate::parser::ParserResult;
    use crate::Message;
    use nom::{
        branch::alt,
        bytes::complete::{tag, take_until1},
        combinator::{map, recognize},
        sequence::pair,
    };

    pub(super) fn sip_message_raw(input: &[u8]) -> ParserResult<&[u8], &[u8]> {
        recognize(pair(take_until1("\r\n\r\n"), tag("\r\n\r\n")))(input)
    }

    pub(super) fn sip_message(input: &str) -> ParserResult<&str, Message> {
        alt((
            map(request, Message::Request),
            map(response, Message::Response),
        ))(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Header, Host, MediaRange, Message, Method, Methods, StatusCode, Transport, Uri, Version,
    };
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
                assert_eq!(request.method(), Method::OPTIONS);
                assert_eq!(request.version(), Version::SIP_2);
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
                        assert_eq!(via.protocol().transport(), Transport::Udp);
                        assert_eq!(via.host(), Host::Name("pc33.atlanta.com".to_string()));
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
                        assert_eq!(cseq_header.method(), Method::OPTIONS);
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
                        assert_eq!(range.media_range(), MediaRange::new("application", "sdp"));
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
                assert_eq!(response.version(), Version::SIP_2);
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
                        assert_eq!(via.protocol().transport(), Transport::Udp);
                        assert_eq!(via.host(), Host::Name("pc33.atlanta.com".to_string()));
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
                        assert_eq!(cseq_header.method(), Method::OPTIONS);
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
                            Method::INVITE,
                            Method::ACK,
                            Method::CANCEL,
                            Method::OPTIONS,
                            Method::BYE,
                        ]
                        .into();
                        assert_eq!(allow_header.methods(), expected_methods);
                    }
                    _ => panic!("Should be an Allow header!"),
                }
                let header = it.next().unwrap();
                match header {
                    Header::Accept(accept_header) => {
                        assert_eq!(accept_header.ranges().len(), 1);
                        let range = accept_header.ranges().first().unwrap();
                        assert_eq!(range.media_range(), MediaRange::new("application", "sdp"));
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
                            MediaRange::new("application", "sdp")
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
                assert_eq!(request.method(), Method::INVITE);
                assert_eq!(request.version(), Version::SIP_2);
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
                        assert_eq!(via.protocol().transport(), Transport::Udp);
                        assert_eq!(via.host(), Host::Name("pc33.atlanta.com".to_string()));
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
                        assert_eq!(cseq_header.method(), Method::INVITE);
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
                assert_eq!(request.method(), Method::ACK);
                assert_eq!(request.version(), Version::SIP_2);
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
                        assert_eq!(via.protocol().transport(), Transport::Udp);
                        assert_eq!(via.host(), Host::Name("pc33.atlanta.com".to_string()));
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
                        assert_eq!(cseq_header.method(), Method::ACK);
                    }
                    _ => panic!("Should be a CSeq header!"),
                }
                assert_eq!(it.next(), None);
                assert_eq!(request.body().len(), 0);
            }
            _ => panic!("Should be a request!"),
        }
    }
}