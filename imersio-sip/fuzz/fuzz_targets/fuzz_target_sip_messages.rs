#![no_main]

use imersio_sip::Message;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = Message::try_from(data);
});
