// Major types constants, used to avoid writing major types everywhere
pub const UNSIGNED_INTEGER: u8   = 0b000_00000;
pub const NEGATIVE_INTEGER: u8   = 0b001_00000;
pub const BYTE_STRING: u8        = 0b010_00000;
pub const TEXT_STRING: u8        = 0b011_00000;
pub const ARRAY_OF_ITEMS: u8     = 0b100_00000;
pub const MAP_OF_ITEMS: u8       = 0b101_00000;
pub const TAGGED_ITEM: u8        = 0b110_00000;
pub const FLOAT_SIMPLE_BREAK: u8 = 0b111_00000;
