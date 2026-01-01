//! The CBOR decoder

use crate::error::DecodeError;
use serde::de::{Deserializer, Visitor};
use std::io::{BufReader, Read};

/// The encoder type
pub struct Decoder<R: Read> {
    reader: BufReader<R>,
}

enum SignedIntegerTarget {
    Signed8,
    Signed16,
    Signed32,
    Signed64
}

impl<R: Read> Decoder<R> {
    /// Construct a new decoder, which will read its input from `R`
    pub fn new(source: R) -> Self {
        Self {
            reader: BufReader::new(source),
        }
    }

    fn read_u8(&mut self) -> Result<u8, DecodeError> {
        let mut u8_buf: [u8; 1] = [0; 1];
        self.reader.read_exact(&mut u8_buf)?;
        Ok(u8_buf[0])
    }

    fn read_u16(&mut self) -> Result<u16, DecodeError> {
        let mut u16_buf: [u8; 2] = [0; 2];
        self.reader.read_exact(&mut u16_buf)?;
        Ok(u16::from_be_bytes(u16_buf))
    }

    fn read_u32(&mut self) -> Result<u32, DecodeError> {
        let mut u32_buf: [u8; 4] = [0; 4];
        self.reader.read_exact(&mut u32_buf)?;
        Ok(u32::from_be_bytes(u32_buf))
    }

    fn read_u64(&mut self) -> Result<u64, DecodeError> {
        let mut u64_buf: [u8; 8] = [0; 8];
        self.reader.read_exact(&mut u64_buf)?;
        Ok(u64::from_be_bytes(u64_buf))
    }

    fn dispatch_signed_integer(&mut self, target: SignedIntegerTarget) -> Result<i64, DecodeError> {
        let initial_byte = self.read_u8()?;
        let raw_value = match initial_byte {
            // 0x20..=0x37 negative integer with additional information 0 to 23
            // We early return here instead of dispatching since its the same logic for all targets
            0x20..=0x37 => return Ok(-1 - ((initial_byte & 0x1F) as i64)),
            // 0x38 = negative integer, value in the next byte
            0x38 => self.read_u8()? as u64,
            // 0x39 = negative integer, value in the next two bytes
            0x39 => self.read_u16()? as u64,
            // 0x3A = negative integer, value in the next four bytes
            0x3A => self.read_u32()? as u64,
            // 0x3B = negative integer, value in the next eight bytes
            0x3B => self.read_u64()?,
            _ => return Err(DecodeError::InvalidType),
        };
        let upper_bound = match target {
            SignedIntegerTarget::Signed8 => i8::MAX as u64,
            SignedIntegerTarget::Signed16 => i16::MAX as u64,
            SignedIntegerTarget::Signed32 => i32::MAX as u64,
            SignedIntegerTarget::Signed64 => i64::MAX as u64,
        };
        Self::decode_signed_integer_with_bounds(raw_value, upper_bound)
    }

    fn decode_signed_integer_with_bounds(
        raw_value: u64,
        upper_bound: u64,
    ) -> Result<i64, DecodeError> {
        if raw_value > upper_bound {
            Err(DecodeError::IntegerOutOfBounds)
        } else {
            Ok(-1 - (raw_value as i64))
        }
    }
}

impl<'de, R: Read> Deserializer<'de> for &mut Decoder<R> {
    type Error = DecodeError;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let byte = self.read_u8()?;
        // 0xF5 = true | 0xF4 = false
        match byte {
            0xF5 => visitor.visit_bool(true),
            0xF4 => visitor.visit_bool(false),
            _ => Err(DecodeError::InvalidType),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(
            self.dispatch_signed_integer(SignedIntegerTarget::Signed8)? as i8
        )
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(
            self.dispatch_signed_integer(SignedIntegerTarget::Signed16)? as i16
        )
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(
            self.dispatch_signed_integer(SignedIntegerTarget::Signed32)? as i32
        )
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(
            self.dispatch_signed_integer(SignedIntegerTarget::Signed64)?
        )
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }
}
