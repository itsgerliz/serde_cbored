//! The CBOR encoder

use crate::{error::EncodeError, {BYTE_STRING, NEGATIVE_INTEGER, TEXT_STRING, UNSIGNED_INTEGER}};
use serde::ser::{
    Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant, Serializer,
};
use std::io::{BufWriter, Write};

/// The encoder type, contains an inner writer where the encoded CBOR data will be written
/// # Considerations
/// - The inner writer is buffered
struct Encoder<W: Write> {
    writer: BufWriter<W>,
}

struct ComplexEncoder<'a, W: Write> {
    encoder: &'a mut Encoder<W>,
    indefinite_length: bool,
    kind: ComplexKind,
}

enum ComplexKind {
    Array,
    Map,
}

impl<W: Write> Encoder<W> {
    /// Construct a new encoder, which will write its output into `W`
    pub fn new(destination: W) -> Self {
        Self {
            writer: BufWriter::new(destination),
        }
    }

    /// The [Encoder]'s inner writer is buffered, this means that while you
    /// might have finished encoding data, this inner buffer could have CBOR data
    /// pending to be written to its writer, this method tries to flush this buffer,
    /// ensuring all pending data is written to its writer
    /// # Considerations
    /// When the [Encoder] is dropped, flush() will be called automatically by the
    /// [std::ops::Drop] trait, but any errors that occur during this process
    /// will be ignored, therefore, its highly recommendable to call this method
    pub fn flush(&mut self) -> Result<(), EncodeError> {
        Ok(self.writer.flush()?)
    }
}

impl<'a, W: Write> Serializer for &'a mut Encoder<W> {
    type Ok = ();
    type Error = EncodeError;

    type SerializeSeq = ComplexEncoder<'a, W>;
    type SerializeTuple = ComplexEncoder<'a, W>;
    type SerializeTupleStruct = ComplexEncoder<'a, W>;
    type SerializeTupleVariant = ComplexEncoder<'a, W>;
    type SerializeMap = ComplexEncoder<'a, W>;
    type SerializeStruct = ComplexEncoder<'a, W>;
    type SerializeStructVariant = ComplexEncoder<'a, W>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        // 0xF5 = true | 0xF4 = false
        let byte: u8 = if v { 0xF5 } else { 0xF4 };

        Ok(self.writer.write_all(&[byte])?)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        if v < 0 {
            let encoded_value: u8 = (-1 - v) as u8;
            if encoded_value < 24 {
                Ok(self.writer.write_all(&[NEGATIVE_INTEGER | encoded_value])?)
            } else {
                // 0x38 = negative integer in the next byte
                Ok(self.writer.write_all(&[0x38, encoded_value])?)
            }
        } else {
            self.serialize_u8(v as u8)
        }
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        if v < 0 {
            if v >= i8::MIN as i16 {
                self.serialize_i8(v as i8)
            } else {
                let encoded_value: u16 = (-1 - v) as u16;
                // 0x39 = negative integer in the next two bytes
                self.writer.write_all(&[0x39])?;
                self.writer.write_all(&encoded_value.to_be_bytes())?;

                Ok(())
            }
        } else {
            self.serialize_u16(v as u16)
        }
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        if v < 0 {
            if v >= i16::MIN as i32 {
                self.serialize_i16(v as i16)
            } else {
                let encoded_value: u32 = (-1 - v) as u32;
                // 0x3A = negative integer in the next four bytes
                self.writer.write_all(&[0x3A])?;
                self.writer.write_all(&encoded_value.to_be_bytes())?;

                Ok(())
            }
        } else {
            self.serialize_u32(v as u32)
        }
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        if v < 0 {
            if v >= i32::MIN as i64 {
                self.serialize_i32(v as i32)
            } else {
                let encoded_value: u64 = (-1 - v) as u64;
                // 0x3B = negative integer in the next eight bytes
                self.writer.write_all(&[0x3B])?;
                self.writer.write_all(&encoded_value.to_be_bytes())?;

                Ok(())
            }
        } else {
            self.serialize_u64(v as u64)
        }
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        if v < 24 {
            Ok(self.writer.write_all(&[UNSIGNED_INTEGER | v])?)
        } else {
            // 0x18 = unsigned integer in the next byte
            Ok(self.writer.write_all(&[0x18, v])?)
        }
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        if v <= u8::MAX as u16 {
            self.serialize_u8(v as u8)
        } else {
            // 0x19 = unsigned integer in the next two bytes
            self.writer.write_all(&[0x19])?;
            self.writer.write_all(&v.to_be_bytes())?;

            Ok(())
        }
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        if v <= u16::MAX as u32 {
            self.serialize_u16(v as u16)
        } else {
            // 0x1A = unsigned integer in the next four bytes
            self.writer.write_all(&[0x1A])?;
            self.writer.write_all(&v.to_be_bytes())?;

            Ok(())
        }
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        if v <= u32::MAX as u64 {
            self.serialize_u32(v as u32)
        } else {
            // 0x1B = unsigned integer in the next eight bytes
            self.writer.write_all(&[0x1B])?;
            self.writer.write_all(&v.to_be_bytes())?;

            Ok(())
        }
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        // TODO
        todo!("Will be implemented in future versions")
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        // TODO
        todo!("Will be implemented in future versions")
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        let data_item_header: u8 = TEXT_STRING | (v.len_utf8() as u8);

        let mut buf: [u8; 4] = [0; 4];
        let utf8_bytes = v.encode_utf8(&mut buf).as_bytes();

        self.writer.write_all(&[data_item_header])?;
        self.writer.write_all(utf8_bytes)?;

        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        let string_len = v.len();

        if string_len < 24 {
            self.writer.write_all(&[TEXT_STRING | (string_len as u8)])?;
            self.writer.write_all(v.as_bytes())?;

            Ok(())
        } else if string_len <= u8::MAX as usize {
            // 0x78 = text string, length in the next byte
            self.writer.write_all(&[0x78, (string_len as u8)])?;
            self.writer.write_all(v.as_bytes())?;

            Ok(())
        } else if string_len <= u16::MAX as usize {
            // 0x79 = text string, length in the next two bytes
            self.writer.write_all(&[0x79])?;
            self.writer.write_all(&(string_len as u16).to_be_bytes())?;
            self.writer.write_all(v.as_bytes())?;

            Ok(())

        } else if string_len <= u32::MAX as usize {
            // 0x7A = text string, length in the next four bytes
            self.writer.write_all(&[0x7A])?;
            self.writer.write_all(&(string_len as u32).to_be_bytes())?;
            self.writer.write_all(v.as_bytes())?;

            Ok(())
        } else if string_len <= u64::MAX as usize {
            // 0x7B = text string, length in the next eight bytes
            self.writer.write_all(&[0x7B])?;
            self.writer.write_all(&(string_len as u64).to_be_bytes())?;
            self.writer.write_all(v.as_bytes())?;

            Ok(())
        } else {
            // The CBOR RFC which this codec is based on does not support text strings longer than
            // 2^64 bytes long, this block is here just for compliance with the RFC
            Err(EncodeError::TextStringTooLong)
        }
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let bytestring_len = v.len();

        if bytestring_len < 24 {
            self.writer.write_all(&[BYTE_STRING | (bytestring_len as u8)])?;
            self.writer.write_all(v)?;

            Ok(())
        } else if bytestring_len <= u8::MAX as usize {
            // 0x58 = byte string, length in the next byte
            self.writer.write_all(&[0x58, (bytestring_len as u8)])?;
            self.writer.write_all(v)?;

            Ok(())
        } else if bytestring_len <= u16::MAX as usize {
            // 0x59 = byte string, length in the next two bytes
            self.writer.write_all(&[0x59])?;
            self.writer.write_all(&(bytestring_len as u16).to_be_bytes())?;
            self.writer.write_all(v)?;

            Ok(())
        } else if bytestring_len <= u32::MAX as usize {
            // 0x5A = byte string, length in the next four bytes
            self.writer.write_all(&[0x5A])?;
            self.writer.write_all(&(bytestring_len as u32).to_be_bytes())?;
            self.writer.write_all(v)?;

            Ok(())
        } else if bytestring_len <= u64::MAX as usize {
            // 0x5B = byte string, length in the next eight bytes
            self.writer.write_all(&[0x5B])?;
            self.writer.write_all(&(bytestring_len as u64).to_be_bytes())?;
            self.writer.write_all(v)?;

            Ok(())
        } else {
            // The CBOR RFC which this codec is based on does not support byte strings longer than
            // 2^64 bytes long, this block is here just for compliance with the RFC
            Err(EncodeError::ByteStringTooLong)
        }
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        // 0xF6 = simple value null
        Ok(self.writer.write_all(&[0xF6])?)
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        // 0xF6 = simple value null
        Ok(self.writer.write_all(&[0xF6])?)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        // 0xF6 = simple value null
        Ok(self.writer.write_all(&[0xF6])?)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        variant.serialize(self)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let mut tuple_encoder = self.serialize_tuple(2)?;
        tuple_encoder.serialize_element(variant)?;
        tuple_encoder.serialize_element(value)?;
        tuple_encoder.end()?;
        Ok(())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        todo!()
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        todo!()
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        todo!()
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        todo!()
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        todo!()
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        todo!()
    }
}
