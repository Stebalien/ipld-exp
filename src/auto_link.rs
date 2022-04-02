use std::{
    cell::Cell,
    lazy::OnceCell,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use cid::Cid;
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};

use crate::{MagicStore, MaybeLink, StaticStore};

/// A type that will be inlined if small enough, but is a link otherwise.
///
/// NOTE: The maximum inline size isn't enforced on decode.
pub struct AutoLink<T, Store, const S: usize = 256> {
    value: OnceCell<T>,
    state: Cell<InlineState>,
    _marker: PhantomData<fn(Store)>,
}

#[derive(Copy, Clone)]
enum InlineState {
    Modified,
    Inlined,
    Link(Cid),
}

impl InlineState {
    fn unwrap_ref(self) -> Cid {
        match self {
            InlineState::Link(c) => c,
            _ => panic!("expected an external reference"),
        }
    }
}

impl<T, Store, const S: usize> Serialize for AutoLink<T, Store, S>
where
    T: Serialize,
    Store: StaticStore,
{
    fn serialize<SS>(&self, serializer: SS) -> Result<SS::Ok, SS::Error>
    where
        SS: Serializer,
    {
        self.save()
            .map_err(<SS::Error as serde::ser::Error>::custom)?
            .serialize(serializer)
    }
}

impl<'de, T, Store, const S: usize> Deserialize<'de> for AutoLink<T, Store, S>
where
    T: Deserialize<'de>,
    Store: StaticStore,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match MaybeLink::deserialize(deserializer)? {
            MaybeLink::Value(value) => AutoLink::from_value(value),
            MaybeLink::Link(k) => AutoLink::from_cid(k),
        })
    }
}

impl<T, Store, const S: usize> From<T> for AutoLink<T, Store, S> {
    fn from(v: T) -> Self {
        Self::from_value(v)
    }
}

impl<T, Store, const S: usize> AutoLink<T, Store, S> {
    #[must_use]
    pub const fn from_cid(k: Cid) -> Self {
        Self {
            state: Cell::new(InlineState::Link(k)),
            value: OnceCell::new(),
            _marker: PhantomData,
        }
    }

    /// Construct a new `AutoLink` from a value.
    #[must_use]
    pub fn from_value(v: T) -> Self {
        Self {
            state: Cell::new(InlineState::Modified),
            value: OnceCell::from(v),
            _marker: PhantomData,
        }
    }
    /// Read the object.
    pub fn read(&self) -> Result<&T, Store::Error>
    where
        T: DeserializeOwned,
        Store: StaticStore,
    {
        self.value
            .get_or_try_init(|| Store::load(&self.state.get().unwrap_ref()))
    }

    /// Edit the object.
    pub fn edit(&mut self) -> Result<&mut T, Store::Error>
    where
        T: DeserializeOwned,
        Store: StaticStore,
    {
        if let InlineState::Link(k) = self.state.get() {
            if self.value.get().is_none() {
                self.value = OnceCell::from(Store::load::<T>(&k)?);
            }
            self.state = Cell::new(InlineState::Modified);
        }
        Ok(self.value.get_mut().expect("expected value"))
    }

    /// Write-back the value if modified, and return a [`MaybeLink`] that's either the object (if
    /// small enough) or a link to it (if too large).
    pub fn save(&self) -> Result<MaybeLink<&T>, Store::Error>
    where
        T: Serialize,
        Store: StaticStore,
    {
        match self.state.get() {
            InlineState::Modified => (),
            InlineState::Link(k) => return Ok(MaybeLink::Link(k)),
            InlineState::Inlined => {
                return Ok(MaybeLink::Value(
                    self.value.get().expect("modified link has no value"),
                ))
            }
        }

        let encoded = Store::encode(self.value.get().expect("modified link has no value"))?;
        if encoded.len() <= S {
            // We're going to throw away the value here: serde doesn't give us a way to handle
            // pre-serialized values generically.
            //
            // However, we only throw away _small_ values, so it's not terrible.
            self.state.set(InlineState::Inlined);
            Ok(MaybeLink::Value(
                self.value.get().expect("modified link has no value"),
            ))
        } else {
            let k = Store::store_bytes(&encoded, None)?;
            self.state.set(InlineState::Link(k));
            Ok(MaybeLink::Link(k))
        }
    }
}

impl<T, Store> Deref for AutoLink<T, Store>
where
    T: DeserializeOwned,
    Store: MagicStore,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        Store::unwrap(self.read())
    }
}

impl<T, Store> DerefMut for AutoLink<T, Store>
where
    T: DeserializeOwned + Serialize,
    Store: MagicStore,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        Store::unwrap(self.edit())
    }
}
