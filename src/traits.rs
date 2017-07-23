use std;
use std::io::{Read, Write};
use futures::Future;

use {Result, Error};
use future::decode::{DecodeInto, DecodeTryInto, DecodeMessage};
use future::encode::EncodeMessage;
use wire::WireType;

pub trait Tag: Default {
    fn number() -> u32;
}

// TODO(?): s/FieldType/WireType/
pub trait FieldType: Default {
    fn wire_type() -> WireType;
}

pub trait MapKey: Default + FieldType {}

pub trait Map: Default {
    type Key: MapKey;
    type Value: FieldType;
    type IntoIter: Iterator<Item = (Self::Key, Self::Value)>;
    fn insert(&mut self, key: Self::Key, value: Self::Value);
    fn into_iter(self) -> Self::IntoIter;
    fn iter<'a>(&'a self) -> Box<Iterator<Item = (&'a Self::Key, &'a Self::Value)> + 'a>;
}
impl<K, V> Map for std::collections::BTreeMap<K, V>
where
    K: MapKey + Ord,
    V: FieldType,
{
    type Key = K;
    type Value = V;
    type IntoIter = std::collections::btree_map::IntoIter<K, V>;
    fn insert(&mut self, key: Self::Key, value: Self::Value) {
        std::collections::BTreeMap::insert(self, key, value);
    }
    fn into_iter(self) -> Self::IntoIter {
        std::iter::IntoIterator::into_iter(self)
    }
    fn iter<'a>(&'a self) -> Box<Iterator<Item = (&'a Self::Key, &'a Self::Value)> + 'a> {
        Box::new(std::collections::BTreeMap::iter(self))
    }
}
impl<K, V> Map for std::collections::HashMap<K, V>
where
    K: MapKey + Eq + std::hash::Hash,
    V: FieldType,
{
    type Key = K;
    type Value = V;
    type IntoIter = std::collections::hash_map::IntoIter<K, V>;
    fn insert(&mut self, key: Self::Key, value: Self::Value) {
        std::collections::HashMap::insert(self, key, value);
    }
    fn into_iter(self) -> Self::IntoIter {
        std::iter::IntoIterator::into_iter(self)
    }
    fn iter<'a>(&'a self) -> Box<Iterator<Item = (&'a Self::Key, &'a Self::Value)> + 'a> {
        Box::new(std::collections::HashMap::iter(self))
    }
}

pub trait Packable: FieldType {}

pub trait Field: Default {}

pub trait SingularField: Field {}

pub trait Message: Sized + Default {
    type Base: Message;
    fn from_base(base: Self::Base) -> Result<Self>;
    fn into_base(self) -> Self::Base;
    fn encode_message<W: Write>(self, writer: W) -> EncodeMessage<W, Self>
    where
        Self::Base: Encode<W>,
    {
        EncodeMessage::new(writer, self)
    }
    fn decode_message<R: Read>(reader: R) -> DecodeMessage<R, Self>
    where
        Self::Base: Decode<R>,
    {
        DecodeMessage::new(reader)
    }
}

pub trait Encode<W: Write>: Sized {
    type Future: Future<Item = W, Error = Error<W>>;
    fn encode(self, writer: W) -> Self::Future;
    fn encoded_size(&self) -> u64;
}

pub trait Decode<R: Read>: Sized {
    type Future: Future<Item = (R, Self), Error = Error<R>>;
    fn decode(reader: R) -> Self::Future;
    fn decode_into<T>(reader: R) -> DecodeInto<R, Self, T>
    where
        T: From<Self>,
    {
        DecodeInto::new(reader)
    }
    fn decode_try_into<T>(reader: R) -> DecodeTryInto<R, Self, T>
    where
        T: TryFrom<Self>,
    {
        DecodeTryInto::new(reader)
    }
}

pub trait DecodeField<R: Read>: Field {
    type Future: Future<Item = (R, Self), Error = Error<R>>;
    fn is_target(tag: u32) -> bool;
    fn decode_field(
        self,
        reader: R,
        tag: u32,
        wire_type: WireType,
    ) -> std::result::Result<Self::Future, Error<R>>;
}

pub trait TryFrom<F>: Sized {
    fn try_from(f: F) -> Result<Self>;
}
