mod bytes;
pub mod de;
pub mod io;
pub mod never;
pub mod ser;

pub use crate::{
    de::{from_bytes, Deserializer},
    ser::{to_bytes, Serializer},
};
