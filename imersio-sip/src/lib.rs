//! A library providing common SIP types and allowing generation and parsing of SIP messages.
//!
//! TODO

#![deny(warnings, missing_docs, missing_debug_implementations)]

pub mod header;
pub mod method;
pub mod reason;
pub mod request;
pub mod response;
pub mod uri;
pub mod version;

mod common;
mod error;
mod parser;
mod utils;

pub use bytes::Bytes;

pub use crate::common::GenericParameter;
pub use crate::error::Error;
pub use crate::header::HeaderAccessor;

pub use crate::header::Header;
pub use crate::method::Method;
pub use crate::reason::{Reason, StatusCode};
pub use crate::request::Request;
pub use crate::response::Response;
pub use crate::uri::Uri;
pub use crate::version::Version;
