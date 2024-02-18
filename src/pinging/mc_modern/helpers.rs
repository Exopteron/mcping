use std::{io, ops::Deref};

use byteorder::{ReadBytesExt, WriteBytesExt};
use thiserror::Error;


pub trait McModernValue: Sized {
    fn read_from(data: &mut impl ReadBytesExt) -> Result<Self, ProtocolError>;

    fn write_to(&self, target: &mut impl WriteBytesExt) -> Result<(), ProtocolError>;
}


#[derive(Debug)]
pub struct VarInt(pub i32);

impl VarInt {
    const SEGMENT_BITS: i32 = 0x7F;
    const CONTINUE_BIT: i32 = 0x80;
    const MAX_LEN: i32 = 32;
}
impl Deref for VarInt {
    type Target = i32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl McModernValue for VarInt {
    fn read_from(data: &mut impl ReadBytesExt) -> Result<Self, ProtocolError> {
        let mut value = 0;
        let mut position = 0;
        loop {
            let current_byte = data.read_u8()? as i32;
            value |= (current_byte & Self::SEGMENT_BITS) << position;
            if (current_byte & Self::CONTINUE_BIT) == 0 {
                return Ok(Self(value));
            }
            position += 7;
            if position >= Self::MAX_LEN {
                return Err(ProtocolError::VarIntTooLarge);
            }
        }
    }

    fn write_to(&self, target: &mut impl WriteBytesExt) -> Result<(), ProtocolError> {
        let mut value = self.0;
        loop {
            if (value & !Self::SEGMENT_BITS) == 0 {
                target.write_u8(value as u8)?;
                return Ok(());
            }

            target.write_u8(((value & Self::SEGMENT_BITS) | Self::CONTINUE_BIT) as u8)?;

            value = ((value as u32) >> 7) as i32;
        }
    }
}

impl McModernValue for String {
    fn read_from(data: &mut impl ReadBytesExt) -> Result<Self, ProtocolError> {
        let len = VarInt::read_from(data)?;
        let mut string_data = vec![0; len.0 as usize];
        data.read_exact(&mut string_data)?;

        Ok(String::from_utf8_lossy(&string_data).to_string())

    }

    fn write_to(&self, target: &mut impl WriteBytesExt) -> Result<(), ProtocolError> {
        let string_data = self.as_bytes();

        VarInt(string_data.len() as i32).write_to(target)?;
        target.write_all(string_data)?;
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("VarInt too large!")]
    VarIntTooLarge,
    #[error("IO error")]
    IoError(#[from] io::Error)
}