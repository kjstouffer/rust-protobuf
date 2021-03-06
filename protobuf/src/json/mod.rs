//! JSON printer and parser which tries to follow
//! [protobuf conventions](https://developers.google.com/protocol-buffers/docs/proto3#json)

mod print;
mod parse;
mod float;
mod base64;

pub use self::print::print_to_string;
pub use self::parse::merge_from_str;
