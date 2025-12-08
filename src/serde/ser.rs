//! The CBOR encoder

use crate::{error::EncodeError, serde::common::{NEGATIVE_INTEGER, TEXT_STRING, UNSIGNED_INTEGER}};
use serde::ser::{
    Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant, Serializer,
};
use std::io::{BufWriter, Write};

/// The encoder type, contains an inner writer where the encoded CBOR data will be written
/// # Considerations
/// - The inner writer is buffered
pub struct Encoder<W: Write> {
    writer: BufWriter<W>,
}

/// The sequence encoder helper type, contains the main encoder type
pub struct SeqEncoder<'a, W: Write> {
    encoder: &'a mut Encoder<W>,
}

/// The tuple encoder helper type, contains the main encoder type
pub struct TupleEncoder<'a, W: Write> {
    encoder: &'a mut Encoder<W>,
}

/// The tuple struct encoder helper type, contains the main encoder type
pub struct TupleStructEncoder<'a, W: Write> {
    encoder: &'a mut Encoder<W>,
}

/// The tuple variant encoder helper type, contains the main encoder type
pub struct TupleVariantEncoder<'a, W: Write> {
    encoder: &'a mut Encoder<W>,
}

/// The map encoder helper type, contains the main encoder type
pub struct MapEncoder<'a, W: Write> {
    encoder: &'a mut Encoder<W>,
}

/// The struct encoder helper type, contains the main encoder type
pub struct StructEncoder<'a, W: Write> {
    encoder: &'a mut Encoder<W>,
}

/// The struct variant encoder helper type, contains the main encoder type
pub struct StructVariantEncoder<'a, W: Write> {
    encoder: &'a mut Encoder<W>,
}

impl<W: Write> Encoder<W> {
    /// Construct a new encoder, which will write its output into `W`
    pub fn into_writer(destination: W) -> Self {
        Self {
            writer: BufWriter::new(destination),
        }
    }

    /// As you can see in the Considerations section on this type,
    /// the [Encoder]'s inner writer is buffered, this means that while you
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

    type SerializeSeq = SeqEncoder<'a, W>;
    type SerializeTuple = TupleEncoder<'a, W>;
    type SerializeTupleStruct = TupleStructEncoder<'a, W>;
    type SerializeTupleVariant = TupleVariantEncoder<'a, W>;
    type SerializeMap = MapEncoder<'a, W>;
    type SerializeStruct = StructEncoder<'a, W>;
    type SerializeStructVariant = StructVariantEncoder<'a, W>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        // Here 0xF5 and 0xF4 represent true and false, respectively
        let byte: u8 = if v { 0xF5 } else { 0xF4 };

        Ok(self.writer.write_all(&[byte])?)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        if v < 0 {
            // Signed branch
            let encoded_value: u8 = (-1 - v) as u8;
            if encoded_value < 24 {
                // Does it fit in additional information?
                Ok(self.writer.write_all(&[NEGATIVE_INTEGER | encoded_value])?)
            } else {
                // Here 0x38 represents a negative integer, stored in the next byte
                Ok(self.writer.write_all(&[0x38, encoded_value])?)
            }
        } else {
            // Unsigned branch, forward to serialize_u8
            self.serialize_u8(v as u8)
        }
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        if v < 0 {
            // Signed branch
            if v >= i8::MIN as i16 {
                // Can this i16 fit in a i8?, if it can, forward to serialize_i8
                self.serialize_i8(v as i8)
            } else {
                let encoded_value: u16 = (-1 - v) as u16;
                // Here 0x39 represents a negative integer, stored in the next two bytes
                self.writer.write_all(&[0x39])?;
                self.writer.write_all(&encoded_value.to_be_bytes())?;

                Ok(())
            }
        } else {
            // Unsigned branch, forward to serialize_u16
            self.serialize_u16(v as u16)
        }
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        if v < 0 {
            // Signed branch
            if v >= i16::MIN as i32 {
                // Can this i32 fit in a i16?, if it can, forward to serialize_i16
                self.serialize_i16(v as i16)
            } else {
                let encoded_value: u32 = (-1 - v) as u32;
                // Here 0x3A represents a negative integer, stored in the next four bytes
                self.writer.write_all(&[0x3A])?;
                self.writer.write_all(&encoded_value.to_be_bytes())?;

                Ok(())
            }
        } else {
            // Unsigned branch, forward to serialize_u32
            self.serialize_u32(v as u32)
        }
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        if v < 0 {
            // Signed branch
            if v >= i32::MIN as i64 {
                // Can this i64 fit in a i32?, if it can, forward to serialize_i32
                self.serialize_i32(v as i32)
            } else {
                let encoded_value: u64 = (-1 - v) as u64;
                // Here 0x3B represents a negative integer, stored in the next eight bytes
                self.writer.write_all(&[0x3B])?;
                self.writer.write_all(&encoded_value.to_be_bytes())?;

                Ok(())
            }
        } else {
            // Unsigned branch, forward to serialize_u64
            self.serialize_u64(v as u64)
        }
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        if v < 24 {
            // Does it fit in additional information?
            Ok(self.writer.write_all(&[UNSIGNED_INTEGER | v])?)
        } else {
            // Here 0x18 represents an unsigned integer, stored in the next byte
            Ok(self.writer.write_all(&[0x18, v])?)
        }
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        if v <= u8::MAX as u16 {
            // Can this u16 fit in a u8?, if it can, forward to serialize_u8
            self.serialize_u8(v as u8)
        } else {
            // Here 0x19 represents an unsigned integer, stored in the next two bytes
            self.writer.write_all(&[0x19])?;
            self.writer.write_all(&v.to_be_bytes())?;

            Ok(())
        }
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        if v <= u16::MAX as u32 {
            // Can this u32 fit in a u16?, if it can, forward to serialize_u16
            self.serialize_u16(v as u16)
        } else {
            // Here 0x1A represents an unsigned integer, stored in the next four bytes
            self.writer.write_all(&[0x1A])?;
            self.writer.write_all(&v.to_be_bytes())?;

            Ok(())
        }
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        if v <= u32::MAX as u64 {
            // Can this u64 fit in a u32?, if it can, forward to serialize_u32
            self.serialize_u32(v as u32)
        } else {
            // Here 0x1B represents an unsigned integer, stored in the next eight bytes
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
            // Does the string length fit in additional information?
            self.writer.write_all(&[TEXT_STRING | (string_len as u8)])?;
            self.writer.write_all(v.as_bytes())?;

            Ok(())
        } else if string_len <= u8::MAX as usize {
            // Does the string length fit in a single byte?
            // Here 0x78 represents a text string, whose length is stored in the next byte
            self.writer.write_all(&[0x78, (string_len as u8)])?;
            self.writer.write_all(v.as_bytes())?;

            Ok(())
        } else if string_len <= u16::MAX as usize {
            // Does the string length fit in two bytes?
            // Here 0x79 represents a text string, whose length is stored in the next two bytes
            self.writer.write_all(&[0x79])?;
            self.writer.write_all(&(string_len as u16).to_be_bytes())?;
            self.writer.write_all(v.as_bytes())?;

            Ok(())

        } else if string_len <= u32::MAX as usize {
            // Does the string length fit in four bytes?
            // Here 0x7A represents a text string, whose length is stored in the next four bytes
            self.writer.write_all(&[0x7A])?;
            self.writer.write_all(&(string_len as u32).to_be_bytes())?;
            self.writer.write_all(v.as_bytes())?;

            Ok(())
        } else if string_len <= u64::MAX as usize {
            // Does the string length fit in eight bytes?
            // Here 0x7B represents a text string, whose length is stored in the next eight bytes
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
        let v_len = v.len();
        if v_len < 24 {
            self.writer.write_all(&[(0b010_00000 | (v_len as u8))])?;
            self.writer.write_all(v)?;
            Ok(())
        } else if v_len <= (u8::MAX as usize) {
            self.writer.write_all(&[0x58, (v_len as u8)])?;
            self.writer.write_all(v)?;
            Ok(())
        } else if v_len <= (u16::MAX as usize) {
            let encoded_value_bigend: [u8; 2] = (v_len as u16).to_be_bytes();
            self.writer
                .write_all(&[0x59, encoded_value_bigend[0], encoded_value_bigend[1]])?;
            self.writer.write_all(v)?;
            Ok(())
        } else if v_len <= (u32::MAX as usize) {
            let encoded_value_bigend: [u8; 4] = (v_len as u32).to_be_bytes();
            self.writer.write_all(&[
                0x5A,
                encoded_value_bigend[0],
                encoded_value_bigend[1],
                encoded_value_bigend[2],
                encoded_value_bigend[3],
            ])?;
            self.writer.write_all(v)?;
            Ok(())
        } else if v_len <= (u64::MAX as usize) {
            let encoded_value_bigend: [u8; 8] = (v_len as u64).to_be_bytes();
            self.writer.write_all(&[
                0x5B,
                encoded_value_bigend[0],
                encoded_value_bigend[1],
                encoded_value_bigend[2],
                encoded_value_bigend[3],
                encoded_value_bigend[4],
                encoded_value_bigend[5],
                encoded_value_bigend[6],
                encoded_value_bigend[7],
            ])?;
            self.writer.write_all(v)?;
            Ok(())
        } else {
            Err(EncodeError::ByteStringTooLong)
        }
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.writer.write_all(&[0xF6])?)
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.writer.write_all(&[0xF6])?)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(self.writer.write_all(&[0xF6])?)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        variant_index.serialize(self)
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
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        variant_index.serialize(&mut *self)?;
        value.serialize(&mut *self)?;
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

impl<'a, W: Write> SerializeSeq for SeqEncoder<'a, W> {
    type Ok = ();
    type Error = EncodeError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a, W: Write> SerializeTuple for TupleEncoder<'a, W> {
    type Ok = ();
    type Error = EncodeError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a, W: Write> SerializeTupleStruct for TupleStructEncoder<'a, W> {
    type Ok = ();
    type Error = EncodeError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a, W: Write> SerializeTupleVariant for TupleVariantEncoder<'a, W> {
    type Ok = ();
    type Error = EncodeError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a, W: Write> SerializeMap for MapEncoder<'a, W> {
    type Ok = ();
    type Error = EncodeError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a, W: Write> SerializeStruct for StructEncoder<'a, W> {
    type Ok = ();
    type Error = EncodeError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a, W: Write> SerializeStructVariant for StructVariantEncoder<'a, W> {
    type Ok = ();
    type Error = EncodeError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}
