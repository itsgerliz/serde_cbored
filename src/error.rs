//! Provides types representing possible errors when encoding/decoding  
//!
//! For simplicity and clarity the specific error cases both for encoding and decoding are directly  
//! declared in its respective enums ([EncodeError] and [DecodeError]) so the [EncodeError::Serialization]  
//! and [DecodeError::Deserialization] variants, while needed by [serde::ser::Error] and [serde::de::Error]  
//! trait contracts, are left unused

use serde::{de, ser};
use std::{fmt::Display, io};
use thiserror::Error;

/// The main error type, holds either an [EncodeError] or a [DecodeError]
#[derive(Error, Debug)]
pub enum Error {
    /// Error while encoding a CBOR data sequence
    #[error("Error while encoding a CBOR data sequence")]
    Encode(#[from] EncodeError),
    /// Error while decoding a CBOR data sequence
    #[error("Error while decoding a CBOR data sequence")]
    Decode(#[from] DecodeError),
}

/// Represents an error while encoding a CBOR data sequence
#[derive(Error, Debug)]
pub enum EncodeError {
    /// Unused variant, needed because of [serde::ser::Error] trait contract (read module docs)
    #[error("Error when serializing")]
    Serialization(String),
    /// Represents an Input/Output error while encoding, usually an error when writing to the  
    /// [Encoder](crate::ser)'s output)
    #[error("Input/Output error")]
    IO(#[from] io::Error),
    /// The CBOR RFC which this codec is based on does not allow data items with lengths above
    /// 2^64 bytes, this number is absurdly big so it should not be reached
    #[error("Cannot encode lengths above 2^64 bytes")]
    LengthOutOfBounds,
}

/// Represents an error while decoding a CBOR data sequence
#[derive(Error, Debug)]
pub enum DecodeError {
    /// Unused variant, needed because of [serde::de::Error] trait contract (read module docs)
    #[error("Error when deserializing")]
    Deserialization(String),
    /// Represents an Input/Output error while decoding, usually an error when reading
    /// from [Decoder](crate::de)'s input
    #[error("Input/Output error")]
    IO(#[from] io::Error),
    /// The decoded data type is not the expected one
    #[error("Invalid type")]
    InvalidType,
    /// The decoded integer is out of the bounds of the expected type
    #[error("Integer out of bounds")]
    IntegerOutOfBounds,
}

impl ser::Error for EncodeError {
    fn custom<T: Display>(msg: T) -> Self {
        Self::Serialization(msg.to_string())
    }
}

impl de::Error for DecodeError {
    fn custom<T: Display>(msg: T) -> Self {
        Self::Deserialization(msg.to_string())
    }
}
