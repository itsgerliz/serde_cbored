//! Error types
//!
//! For simplicity and clarity the specific error cases both for encoding and decoding are directly  
//! declared in its respective enums ([EncodeError] and [DecodeError]) so the [EncodeError::Serialization]  
//! and [DecodeError::Deserialization] variants, while needed by [serde::ser::Error] and [serde::de::Error]  
//! trait contracts, are left unused

use serde::{de, ser};
use std::{fmt::Display, io};
use thiserror::Error;

/// Represents an error while encoding a CBOR data sequence
#[cfg(feature = "ser")]
#[derive(Error, Debug)]
pub enum EncodeError {
    /// Unused variant, needed because of [serde::ser::Error] trait contract (read module docs)
    #[error("Error when serializing")]
    Serialization(String),
    /// Input/Output error while encoding, usually an error when writing to the [Encoder](crate::ser)'s output
    #[error("Input/Output error")]
    IO(#[from] io::Error),
    /// The CBOR RFC which this codec is based on does not allow data items with lengths above
    /// 2^64 bytes, this number is absurdly big so it should not be reached
    #[error("Cannot encode lengths above 2^64 bytes")]
    LengthOutOfBounds,
}

/// Represents an error while decoding a CBOR data sequence
#[cfg(feature = "de")]
#[derive(Error, Debug)]
pub enum DecodeError {
    /// Unused variant, needed because of [serde::de::Error] trait contract (read module docs)
    #[error("Error when deserializing")]
    Deserialization(String),
    /// Input/Output error while decoding, usually an error when reading from [Decoder](crate::de)'s input
    #[error("Input/Output error")]
    IO(#[from] io::Error),
}

#[cfg(feature = "ser")]
impl ser::Error for EncodeError {
    fn custom<T: Display>(msg: T) -> Self {
        Self::Serialization(msg.to_string())
    }
}

#[cfg(feature = "de")]
impl de::Error for DecodeError {
    fn custom<T: Display>(msg: T) -> Self {
        Self::Deserialization(msg.to_string())
    }
}
