use std::marker::PhantomData;

use cid::Cid;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone)]
pub struct CidShape {
    pub codec: u64,
    pub mh_code: u64,
}

impl From<&Cid> for CidShape {
    fn from(k: &Cid) -> Self {
        CidShape {
            codec: k.codec(),
            mh_code: k.hash().code(),
        }
    }
}

pub trait StaticStore {
    type Error: std::error::Error;

    fn store_bytes(value: &[u8], shape: Option<&CidShape>) -> Result<Cid, Self::Error>;
    fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, Self::Error>;

    fn load_bytes(k: &Cid) -> Result<Vec<u8>, Self::Error>;

    /// Decode an object.
    fn decode<'a, T: Deserialize<'a>>(bytes: &[u8]) -> Result<T, Self::Error>;

    /// Load an object.
    fn load<'a, T: Deserialize<'a>>(key: &Cid) -> Result<T, Self::Error> {
        Self::decode(&Self::load_bytes(key)?)
    }
    /// Store an object. The `shape` is a hint.
    fn store<T: Serialize>(value: &T, shape: Option<&CidShape>) -> Result<Cid, Self::Error> {
        Self::store_bytes(&Self::encode(value)?, shape)
    }
}

pub trait MagicStore: StaticStore {
    fn unwrap<T>(r: Result<T, Self::Error>) -> T {
        r.unwrap()
    }
}

pub struct Magic<S>(PhantomData<S>);

impl<S> StaticStore for Magic<S>
where
    S: StaticStore,
{
    type Error = S::Error;

    fn store_bytes(value: &[u8], shape: Option<&CidShape>) -> Result<Cid, Self::Error> {
        S::store_bytes(value, shape)
    }

    fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, Self::Error> {
        S::encode(value)
    }

    fn load_bytes(k: &Cid) -> Result<Vec<u8>, Self::Error> {
        S::load_bytes(k)
    }

    fn decode<'a, T: Deserialize<'a>>(bytes: &[u8]) -> Result<T, Self::Error> {
        S::decode(bytes)
    }

    fn load<'a, T: Deserialize<'a>>(key: &Cid) -> Result<T, Self::Error> {
        S::load(key)
    }

    fn store<T: Serialize>(value: &T, shape: Option<&CidShape>) -> Result<Cid, Self::Error> {
        S::store(value, shape)
    }
}

impl<S> MagicStore for Magic<S> where S: StaticStore {}
