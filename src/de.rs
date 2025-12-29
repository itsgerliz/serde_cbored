//! The CBOR decoder

use std::io::{BufReader, Read};

use serde::de::{Deserializer, Visitor};

use crate::error::DecodeError;

/// The encoder type
pub struct Decoder<R: Read> {
    reader: BufReader<R>,
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
}

impl<'de, R: Read> Deserializer<'de> for &mut Decoder<R> {
	type Error = DecodeError;

	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de> {
		todo!()
	}

	fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de> {
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
		V: Visitor<'de> {
		let initial_byte = self.read_u8()?;
		match initial_byte {
			// 0x20..=0x37 negative integer with additional information 0 to 23
			0x20..=0x37 => {
				let encoded_value = -1 - ((initial_byte & 0x1F) as i8);
				visitor.visit_i8(encoded_value)
			},
			// 0x38 = negative integer, value in the next byte
			0x38 => {
				// When reading the next byte we could read something out of i8 bounds,
				// for example, 234, what would result in -1 - (234 as i8), what becomes
				// -1 - (-22), therefore computing 21, a wrong value
				// We fix this by computing the encoded value in a wide signed integer
				// and then doing a bound check
				let encoded_value = -1 - (self.read_u8()? as i64);
				if encoded_value < i8::MIN as i64 {
					Err(DecodeError::IntegerUnderflow)
				} else if encoded_value > i8::MAX as i64 {
					Err(DecodeError::IntegerOverflow)
				} else {
					visitor.visit_i8(encoded_value as i8)
				}
			},
			_ => Err(DecodeError::InvalidType)
		}
	}

	fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de> {
		todo!()
	}

	fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de> {
		todo!()
	}

	fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de> {
		todo!()
	}

	fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de> {
		todo!()
	}

	fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de> {
		todo!()
	}

	fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de> {
		todo!()
	}

	fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de> {
		todo!()
	}

	fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de> {
		todo!()
	}

	fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de> {
		todo!()
	}

	fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de> {
		todo!()
	}

	fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de> {
		todo!()
	}

	fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de> {
		todo!()
	}

	fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de> {
		todo!()
	}

	fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de> {
		todo!()
	}

	fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de> {
		todo!()
	}

	fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de> {
		todo!()
	}

	fn deserialize_unit_struct<V>(
		self,
		name: &'static str,
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de> {
		todo!()
	}

	fn deserialize_newtype_struct<V>(
		self,
		name: &'static str,
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de> {
		todo!()
	}

	fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de> {
		todo!()
	}

	fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de> {
		todo!()
	}

	fn deserialize_tuple_struct<V>(
		self,
		name: &'static str,
		len: usize,
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de> {
		todo!()
	}

	fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de> {
		todo!()
	}

	fn deserialize_struct<V>(
		self,
		name: &'static str,
		fields: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de> {
		todo!()
	}

	fn deserialize_enum<V>(
		self,
		name: &'static str,
		variants: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de> {
		todo!()
	}

	fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de> {
		todo!()
	}

	fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de> {
		todo!()
	}
}
