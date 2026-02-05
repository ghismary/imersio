use criterion::{Criterion, criterion_group, criterion_main};

use imersio_sip::Uri;

fn simple_sip_uri_parsing() {
    let _ = Uri::try_from("sip:registrar.biloxi.com");
}

fn simple_sips_uri_parsing() {
    let _ = Uri::try_from("sips:registrar.biloxi.com");
}

fn simple_sip_uri_ipv4_parsing() {
    let _ = Uri::try_from("sip:192.168.0.1");
}

// fn simple_sip_uri_ipv6_parsing() {
//     let _ = Uri::try_from("sip:[fe80::1]");
// }

fn complex_sip_uri_parsing() {
    let _ = Uri::try_from("sip:alice@atlanta.com:5060;transport=tcp");
}

fn complex_sip_uri_ipv6_parsing() {
    let _ = Uri::try_from("sip:bob@[2a01:e35:1387:1020:6233:4bff:fe0b:5663]:5060;transport=tcp");
}

fn sip_uri_with_phone_number_parsing() {
    let _ = Uri::try_from("sip:+331231231231@sip.example.org;user=phone");
}

fn sip_uri_with_parameters_parsing() {
    let _ = Uri::try_from(
        "sip:maddr=@192.168.0.1;lr;maddr=192.168.0.1;user=ip;ttl=140;transport=sctp;method=INVITE;rport=5060",
    );
}

fn sip_uri_with_headers_parsing() {
    let _ = Uri::try_from(
        "sip:eNgwBpkNcH6EdTHlX0cq8@example.org?P-Group-Id=Fu0hHIQ23H4hveVT:New%20Group&P-Expert-Profile-Id=zKQOBOB2jTmUOjkB:New%20Group&P-Reverse-Charging=0&P-Campaign-Id=none&P-Embed-Url=https://example.org/caller/?1.4.0-dev-42-91bdf0c%26id%3DFu0hHIQ23H4hveVT%26CAMPAIGN_ID%3Dnone",
    );
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Simple SIP uri parsing", |b| {
        b.iter(|| simple_sip_uri_parsing())
    });
    c.bench_function("Simple SIPS uri parsing", |b| {
        b.iter(|| simple_sips_uri_parsing())
    });
    c.bench_function("Simple SIP uri IPv4 parsing", |b| {
        b.iter(|| simple_sip_uri_ipv4_parsing())
    });
    // c.bench_function("Simple SIP uri IPv6 parsing", |b| {
    //     b.iter(|| simple_sip_uri_ipv6_parsing())
    // });
    c.bench_function("Complex SIP uri parsing", |b| {
        b.iter(|| complex_sip_uri_parsing())
    });
    c.bench_function("Complex SIP uri IPv6 parsing", |b| {
        b.iter(|| complex_sip_uri_ipv6_parsing())
    });
    c.bench_function("SIP uri with phone number parsing", |b| {
        b.iter(|| sip_uri_with_phone_number_parsing())
    });
    c.bench_function("SIP uri with parameters parsing", |b| {
        b.iter(|| sip_uri_with_parameters_parsing())
    });
    c.bench_function("SIP uri with headers parsing", |b| {
        b.iter(|| sip_uri_with_headers_parsing())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
