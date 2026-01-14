//! The CBOR encoder

use crate::{
    error::EncodeError,
};
use serde::ser::{
    Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant, Serializer,
};
use std::io::{BufWriter, Write};

/// The encoder type
/// # Considerations
/// - This type is buffered, read [Encoder::flush]
pub struct Encoder<W: Write> {
    writer: BufWriter<W>,
}

/// The complex encoder type
pub struct ComplexEncoder<'encoder, W: Write> {
    encoder: &'encoder mut Encoder<W>,
    indefinite_length: bool,
}

enum ArgumentPlacement {
    AdditionalInformation,
    NextByte,
    NextTwoBytes,
    NextFourBytes,
    NextEightBytes,
}

impl<W: Write> Encoder<W> {
    /// Construct a new encoder, which will write its output into `W`
    pub fn new(destination: W) -> Self {
        Self {
            writer: BufWriter::new(destination),
        }
    }

    /// The [Encoder] is buffered, this means that while you might have finished
    /// encoding data, this inner buffer could have CBOR data pending to be written
    /// to the output, this method tries to flush this buffer, ensuring all pending
    /// data is written to its output
    /// # Considerations
    /// When the [Encoder] is dropped, flush() will be called automatically by the
    /// [std::ops::Drop] trait, but any errors that might occur during this process
    /// will be ignored, therefore, its highly recommendable to call this method
    pub fn flush(&mut self) -> Result<(), EncodeError> {
        Ok(self.writer.flush()?)
    }

    fn write_u8(&mut self, data: u8) -> Result<(), EncodeError> {
        Ok(self.writer.write_all(&[data])?)
    }

    fn write_u16(&mut self, data: u16) -> Result<(), EncodeError> {
        Ok(self.writer.write_all(&data.to_be_bytes())?)
    }

    fn write_u32(&mut self, data: u32) -> Result<(), EncodeError> {
        Ok(self.writer.write_all(&data.to_be_bytes())?)
    }

    fn write_u64(&mut self, data: u64) -> Result<(), EncodeError> {
        Ok(self.writer.write_all(&data.to_be_bytes())?)
    }

    fn write_bytes(&mut self, data: &[u8]) -> Result<(), EncodeError> {
        Ok(self.writer.write_all(data)?)
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
        let boolean = if v { 0xF5 } else { 0xF4 };
        self.write_u8(boolean)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        if v < 0 {
            let encoded_value = (-1 - v) as u8;
            if encoded_value < 24 {
                // 0x20 = negative integer major type
                self.write_u8(0x20 | encoded_value)
            } else {
                // 0x38 = negative integer in the next byte
                self.write_bytes(&[0x38, encoded_value])
            }
        } else {
            self.serialize_u8(v as u8)
        }
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        if v < 0 {
            let encoded_value = (-1 - v) as u16;
            // 0x39 = negative integer in the next two bytes
            self.write_u8(0x39);
            self.write_u16(encoded_value)
        } else {
            self.serialize_u16(v as u16)
        }
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        if v < 0 {
            let encoded_value = (-1 - v) as u32;
            // 0x3A = negative integer in the next four bytes
            self.write_u8(0x3A);
            self.write_u32(encoded_value)
        } else {
            self.serialize_u32(v as u32)
        }
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        if v < 0 {
            let encoded_value = (-1 - v) as u64;
            // 0x3B = negative integer in the next eight bytes
            self.write_u8(0x3B);
            self.write_u64(encoded_value)
        } else {
            self.serialize_u64(v as u64)
        }
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        if v < 24 {
            // 0x00 = unsigned integer major type
            self.write_u8(0x00 | v)
        } else {
            // 0x18 = unsigned integer in the next byte
            self.write_bytes(&[0x18, v])
        }
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        // 0x19 = unsigned integer in the next two bytes
        self.write_u8(0x19);
        self.write_u16(v)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        // 0x1A = unsigned integer in the next four bytes
        self.write_u8(0x1A);
        self.write_u32(v)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        // 0x1B = unsigned integer in the next eight bytes
        self.write_u8(0x1B);
        self.write_u64(v)
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        todo!("Will be implemented in future versions")
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        todo!("Will be implemented in future versions")
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        let header = 0x60 | v.len_utf8() as u8;

        let mut buf: [u8; 4] = [0; 4];
        let utf8_bytes = v.encode_utf8(&mut buf).as_bytes();

        self.write_u8(header);
        self.write_bytes(utf8_bytes)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        let v_length = v.len();
        // 0x78 = text string, length in the next byte
        // 0x79 = text string, length in the next two bytes
        // 0x7A = text string, length in the next four bytes
        // 0x7B = text string, length in the next eight bytes
        match Encoder::<W>::calculate_argument_placement(v_length)? {
            ArgumentPlacement::AdditionalInformation => {
                self.writer.write_all(&[TEXT_STRING | (v_length as u8)])?
            }
            ArgumentPlacement::NextByte => self.writer.write_all(&[0x78, (v_length as u8)])?,
            ArgumentPlacement::NextTwoBytes => {
                self.writer.write_all(&[0x79])?;
                self.writer.write_all(&(v_length as u16).to_be_bytes())?;
            }
            ArgumentPlacement::NextFourBytes => {
                self.writer.write_all(&[0x7A])?;
                self.writer.write_all(&(v_length as u32).to_be_bytes())?;
            }
            ArgumentPlacement::NextEightBytes => {
                self.writer.write_all(&[0x7B])?;
                self.writer.write_all(&(v_length as u64).to_be_bytes())?;
            }
        }
        self.writer.write_all(v.as_bytes())?;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let v_length = v.len();
        // 0x58 = byte string, length in the next byte
        // 0x59 = byte string, length in the next two bytes
        // 0x5A = byte string, length in the next four bytes
        // 0x5B = byte string, length in the next eight bytes
        match Encoder::<W>::calculate_argument_placement(v_length)? {
            ArgumentPlacement::AdditionalInformation => {
                self.writer.write_all(&[BYTE_STRING | (v_length as u8)])?
            }
            ArgumentPlacement::NextByte => self.writer.write_all(&[0x58, (v_length as u8)])?,
            ArgumentPlacement::NextTwoBytes => {
                self.writer.write_all(&[0x59])?;
                self.writer.write_all(&(v_length as u16).to_be_bytes())?;
            }
            ArgumentPlacement::NextFourBytes => {
                self.writer.write_all(&[0x5A])?;
                self.writer.write_all(&(v_length as u32).to_be_bytes())?;
            }
            ArgumentPlacement::NextEightBytes => {
                self.writer.write_all(&[0x5B])?;
                self.writer.write_all(&(v_length as u64).to_be_bytes())?;
            }
        }
        self.writer.write_all(v)?;
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        // 0xF6 = null
        Ok(self.writer.write_all(&[0xF6])?)
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        // 0xF6 = null
        Ok(self.writer.write_all(&[0xF6])?)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        // 0xF6 = null
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
        SerializeTuple::serialize_element(&mut tuple_encoder, variant)?;
        SerializeTuple::serialize_element(&mut tuple_encoder, value)?;
        SerializeTuple::end(tuple_encoder)?;
        Ok(())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        match len {
            Some(length) => self.serialize_tuple(length),
            None => {
                // 0x9F = array of data items, indefinite length
                self.writer.write_all(&[0x9F])?;
                Ok(ComplexEncoder {
                    encoder: self,
                    indefinite_length: true,
                })
            }
        }
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        // 0x98 = array of data items, length in the next byte
        // 0x99 = array of data items, length in the next two bytes
        // 0x9A = array of data items, length in the next four bytes
        // 0x9B = array of data items, length in the next eight bytes
        match Encoder::<W>::calculate_argument_placement(len)? {
            ArgumentPlacement::AdditionalInformation => {
                self.writer.write_all(&[ARRAY_OF_ITEMS | (len as u8)])?
            }
            ArgumentPlacement::NextByte => self.writer.write_all(&[0x98, (len as u8)])?,
            ArgumentPlacement::NextTwoBytes => {
                self.writer.write_all(&[0x99])?;
                self.writer.write_all(&(len as u16).to_be_bytes())?;
            }
            ArgumentPlacement::NextFourBytes => {
                self.writer.write_all(&[0x9A])?;
                self.writer.write_all(&(len as u32).to_be_bytes())?;
            }
            ArgumentPlacement::NextEightBytes => {
                self.writer.write_all(&[0x9B])?;
                self.writer.write_all(&(len as u64).to_be_bytes())?;
            }
        }
        Ok(ComplexEncoder {
            encoder: self,
            indefinite_length: false,
        })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        // We serialize it as a tuple with len elements + 1 for the variant name
        // then return the tuple_encoder so the caller can keep serializing the
        // remaining fields
        let mut tuple_encoder = self.serialize_tuple(len + 1)?;
        SerializeTupleVariant::serialize_field(&mut tuple_encoder, variant)?;
        Ok(tuple_encoder)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        match len {
            Some(length) => {
                // 0xB8 = map of pairs of data items, length in the next byte
                // 0xB9 = map of pairs of data items, length in the next two bytes
                // 0xBA = map of pairs of data items, length in the next four bytes
                // 0xBB = map of pairs of data items, length in the next eight bytes
                match Encoder::<W>::calculate_argument_placement(length)? {
                    ArgumentPlacement::AdditionalInformation => {
                        self.writer.write_all(&[MAP_OF_ITEMS | (length as u8)])?
                    }
                    ArgumentPlacement::NextByte => {
                        self.writer.write_all(&[0xB8, (length as u8)])?
                    }
                    ArgumentPlacement::NextTwoBytes => {
                        self.writer.write_all(&[0xB9])?;
                        self.writer.write_all(&(length as u16).to_be_bytes())?;
                    }
                    ArgumentPlacement::NextFourBytes => {
                        self.writer.write_all(&[0xBA])?;
                        self.writer.write_all(&(length as u32).to_be_bytes())?;
                    }
                    ArgumentPlacement::NextEightBytes => {
                        self.writer.write_all(&[0xBB])?;
                        self.writer.write_all(&(length as u64).to_be_bytes())?;
                    }
                }
                Ok(ComplexEncoder {
                    encoder: self,
                    indefinite_length: false,
                })
            }
            None => {
                // 0xBF = array of data items, indefinite length
                self.writer.write_all(&[0xBF])?;
                Ok(ComplexEncoder {
                    encoder: self,
                    indefinite_length: true,
                })
            }
        }
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        // We serialize it as a tuple with two elements, the variant name
        // and a map containing the struct fields
        // First, write the header of the tuple
        // Then, write the variant name as the first element of the tuple
        // Finally, write the header of the map as second element and return
        // the map_encoder so the caller can serialize the remaining fields
        let mut tuple_encoder = self.serialize_tuple(2)?;
        SerializeTuple::serialize_element(&mut tuple_encoder, variant)?;
        let map_encoder = self.serialize_map(Some(len))?;
        Ok(map_encoder)
    }
}

impl<'a, W: Write> SerializeSeq for ComplexEncoder<'a, W> {
    type Ok = ();
    type Error = EncodeError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut *self.encoder)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // 0xFF = break byte
        if self.indefinite_length {
            self.encoder.writer.write_all(&[0xFF])?;
        }
        Ok(())
    }
}

impl<'a, W: Write> SerializeTuple for ComplexEncoder<'a, W> {
    type Ok = ();
    type Error = EncodeError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut *self.encoder)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W: Write> SerializeTupleStruct for ComplexEncoder<'a, W> {
    type Ok = ();
    type Error = EncodeError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut *self.encoder)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W: Write> SerializeTupleVariant for ComplexEncoder<'a, W> {
    type Ok = ();
    type Error = EncodeError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut *self.encoder)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W: Write> SerializeMap for ComplexEncoder<'a, W> {
    type Ok = ();
    type Error = EncodeError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut *self.encoder)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut *self.encoder)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // 0xFF = break byte
        if self.indefinite_length {
            self.encoder.writer.write_all(&[0xFF])?;
        }
        Ok(())
    }
}

impl<'a, W: Write> SerializeStruct for ComplexEncoder<'a, W> {
    type Ok = ();
    type Error = EncodeError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut *self.encoder)?;
        value.serialize(&mut *self.encoder)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W: Write> SerializeStructVariant for ComplexEncoder<'a, W> {
    type Ok = ();
    type Error = EncodeError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut *self.encoder)?;
        value.serialize(&mut *self.encoder)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
