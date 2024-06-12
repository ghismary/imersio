//! TODO

use crate::common::header_value_collection::HeaderValueCollection;

pub mod accept_encoding;
pub mod accept_language;
pub mod accept_parameter;
pub mod accept_range;
pub mod alert;
pub mod algorithm;
pub mod auth_parameter;
pub mod authentication_info;
pub mod call_info;
pub mod call_info_parameter;
pub mod challenge;
pub mod contact;
pub mod contact_parameter;
pub mod content_encoding;
pub mod content_language;
pub mod credentials;
pub mod disposition_parameter;
pub mod disposition_type;
pub mod domain_uri;
pub mod error_uri;
pub mod from_parameter;
pub mod generic_parameter;
pub mod handling;
pub mod header_value_collection;
pub mod media_parameter;
pub mod media_range;
pub mod media_type;
pub mod message_qop;
pub mod name_address;
pub mod priority;
pub mod stale;
pub mod wrapped_string;

/// Representation of the list of call IDs in a `In-Reply-To` header.
///
/// This is usable as an iterator.
pub type CallIds = HeaderValueCollection<String>;
