

/// A structure for serializing Rust values to CBOR.
pub struct Serializer<W> {
    writer: W,
    encode_most_common: bool,
}

/// Serializes a value without names to a vector.
///
/// Struct fields and enum variants are identified by their numeric indices rather than names to
/// save space.
// pub fn to_vec_packed<T>(value: &T) -> Result<Vec<u8>>
// where
//     T: ser::Serialize,
// {
//     let mut ser = Serializer::packed(&mut writer);
//     ser.self_describe()?;
//     value.serialize(&mut ser)

//     let mut vec = Vec::new();
//     to_writer_packed(&mut vec, value)?;
//     Ok(vec)
// }


impl<'a, W> ser::Serializer for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = CollectionSerializer<'a, W>;
    type SerializeTuple = &'a mut Serializer<W>;
    type SerializeTupleStruct = &'a mut Serializer<W>;
    type SerializeTupleVariant = &'a mut Serializer<W>;
    type SerializeMap = CollectionSerializer<'a, W>;
    type SerializeStruct = StructSerializer<'a, W>;
    type SerializeStructVariant = StructSerializer<'a, W>;

    // #[inline]
    // fn serialize_bool(self, value: bool) -> Result<()> {
    //     let value = if value { 0xf5 } else { 0xf4 };
    //     self.writer.write_all(&[value]).map_err(Error::io)
    // }

    // #[inline]
    // fn serialize_u8(self, value: u8) -> Result<()> {
    //     self.write_u8(0, value)
    // }

    // #[inline]
    // fn serialize_u16(self, value: u16) -> Result<()> {
    //     self.write_u16(0, value)
    // }

    // #[inline]
    // fn serialize_u32(self, value: u32) -> Result<()> {
    //     self.write_u32(0, value)
    // }

    // #[inline]
    // fn serialize_u64(self, value: u64) -> Result<()> {
    //     self.write_u64(0, value)
    // }

    #[inline]
    fn serialize_bytes(self, value: &[u8]) -> Result<()> {
        self.write_u64(2, value.len() as u64)?;
        self.writer.write_all(value).map_err(Error::io)
    }

    #[inline]
    fn serialize_unit(self) -> Result<()> {
        self.serialize_none()
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_none(self) -> Result<()> {
        self.writer.write_all(&[0xf6]).map_err(Error::io)
    }


    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<CollectionSerializer<'a, W>> {
        self.serialize_collection(4, len)
    }

    #[inline]
    fn serialize_map(self, len: Option<usize>) -> Result<CollectionSerializer<'a, W>> {
        self.serialize_collection(5, len)
    }

    #[inline]
    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<StructSerializer<'a, W>> {
        self.write_u64(5, len as u64)?;
        Ok(StructSerializer { ser: self, idx: 0 })
    }
}