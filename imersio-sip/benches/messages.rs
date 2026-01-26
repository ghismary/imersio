use criterion::{criterion_group, criterion_main, Criterion};

use imersio_sip::Message;

fn options_request_parsing() {
    let _ = Message::try_from(
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
}

fn ok_response_to_options_request_parsing() {
    let _ = Message::try_from(
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
}

fn invite_request_parsing() {
    let _ = Message::try_from(
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
}

fn ack_request_for_non_2xx_response_to_invite_request_parsing() {
    let _ = Message::try_from(
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
}

fn tunneled_invite_request_parsing() {
    let _ = Message::try_from(
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
}

fn register_request_parsing() {
    let _ = Message::try_from(
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
}

fn ok_response_to_register_request_parsing() {
    let _ = Message::try_from(
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
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("OPTIONS request parsing", |b| {
        b.iter(|| options_request_parsing())
    });
    c.bench_function("200 OK response to OPTIONS request parsing", |b| {
        b.iter(|| ok_response_to_options_request_parsing())
    });
    c.bench_function("INVITE request parsing", |b| {
        b.iter(|| invite_request_parsing())
    });
    c.bench_function(
        "ACK request for non-2xx response to INVITE request parsing",
        |b| b.iter(|| ack_request_for_non_2xx_response_to_invite_request_parsing()),
    );
    c.bench_function("Tunneled INVITE request parsing", |b| {
        b.iter(|| tunneled_invite_request_parsing())
    });
    c.bench_function("REGISTER request parsing", |b| {
        b.iter(|| register_request_parsing())
    });
    c.bench_function("200 OK response to REGISTER request parsing", |b| {
        b.iter(|| ok_response_to_register_request_parsing())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
