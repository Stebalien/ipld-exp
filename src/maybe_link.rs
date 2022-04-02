use std::marker::PhantomData;

use cid::{serde::BytesToCidVisitor, Cid};
use serde::{
    de::{
        value::{MapAccessDeserializer, SeqAccessDeserializer},
        IntoDeserializer, Visitor,
    },
    forward_to_deserialize_any, Deserialize, Deserializer, Serialize,
};

/// An type to represent IPLD values that can either be link, or any other value.
pub enum MaybeLink<T> {
    Value(T),
    Link(Cid),
}

impl<T> Serialize for MaybeLink<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            MaybeLink::Value(v) => Serialize::serialize(v, serializer),
            MaybeLink::Link(k) => Serialize::serialize(k, serializer),
        }
    }
}

impl<'de, T> Deserialize<'de> for MaybeLink<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(MaybeLinkVisitor(PhantomData))
    }
}

struct NoneDeserializer<E>(PhantomData<fn() -> E>);
impl<'de, E> Deserializer<'de> for NoneDeserializer<E>
where
    E: serde::de::Error,
{
    type Error = E;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple tuple_struct
        map struct enum identifier ignored_any option
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_none()
    }
}

struct SomeDeserializer<D>(D);
impl<'de, D> Deserializer<'de> for SomeDeserializer<D>
where
    D: Deserializer<'de>,
{
    type Error = D::Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple tuple_struct
        map struct enum identifier ignored_any option
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self.0)
    }
}

struct MaybeLinkVisitor<T>(PhantomData<fn() -> T>);

fn visit_value<'de, V, E, T>(
    v: V,
) -> Result<MaybeLink<T>, <V::Deserializer as Deserializer<'de>>::Error>
where
    T: Deserialize<'de>,
    V: IntoDeserializer<'de, E>,
    E: serde::de::Error,
{
    Deserialize::deserialize(v.into_deserializer()).map(MaybeLink::Value)
}

impl<'de, T> Visitor<'de> for MaybeLinkVisitor<T>
where
    T: Deserialize<'de>,
{
    type Value = MaybeLink<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "failed to decode into a 'maybe link'")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_value(v)
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_value(v)
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_value(v)
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_value(v)
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_value(v)
    }

    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_value(v)
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_value(v)
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_value(v)
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_value(v)
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_value(v)
    }

    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_value(v)
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_value(v)
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_value(v)
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_value(v)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_value(v)
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_value(v)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_value(v)
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_value(v)
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_value(v)
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        visit_value(v)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        T::deserialize(NoneDeserializer(PhantomData)).map(MaybeLink::Value)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // TODO: is this right? This will just recurse on "some", which is likely the best we can
        // do.
        deserializer.deserialize_any(self)
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // TODO: is this safe?
        deserializer
            .deserialize_bytes(BytesToCidVisitor)
            .map(MaybeLink::Link)
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        Deserialize::deserialize(SeqAccessDeserializer::new(seq)).map(MaybeLink::Value)
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        Deserialize::deserialize(MapAccessDeserializer::new(map)).map(MaybeLink::Value)
    }
}
