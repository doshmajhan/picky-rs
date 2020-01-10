use crate::{misc::WriteExt, Result, Serializer};
use picky_asn1::tag::Tag;

/// A serializer for raw der values
pub struct Raw;
impl Raw {
    /// Serializes `value` into `writer`
    pub fn serialize(value: Vec<u8>, ser: &mut Serializer) -> Result<usize> {
        written += ser.writer.write_exact(&value)?;
        Ok(written)
    }
}
