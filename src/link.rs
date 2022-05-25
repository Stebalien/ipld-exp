use cid::Cid;
use serde::{de::DeserializeOwned, ser::Error, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    cell::Cell,
    lazy::OnceCell,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{CidShape, MagicStore, StaticStore};

/// An IPLD link that loads data via the specified type-level [`Store`], caches the value, and
/// writes it back on [`Link::save`].
///
/// ```
/// use auto_ipld::{Link, BadStore as Store, MagicStore};
/// use serde::{Deserialize, de::DeserializeOwned};
///
/// #[derive(Deserialize)]
/// pub struct Node<T, Store> {
///     pub value: T,
///     pub next: Option<Link<Box<Node<T>>, Store>>,
/// }
///
/// impl<T, Store> Node<T, Store> where Store: MagicStore {
///     pub fn find(&self, mut cond: impl FnMut(&T) -> bool) -> Option<&T>
///     where
///         T: DeserializeOwned,
///     {
///         if cond(&self.value) {
///             return Some(&self.value)
///         } else {
///             // obviously a horrible idea, bit it's a demo!
///             self.next.as_deref().and_then(|next| next.find(cond))
///         }
///     }
/// }
///```
#[derive(Clone)]
pub struct Link<T, Store> {
    value: OnceCell<T>,
    state: Cell<LinkState>,
    _marker: PhantomData<fn(Store)>,
}

#[derive(Copy, Clone)]
enum LinkState {
    Unmodified(Cid),
    Modified(Option<CidShape>),
}

impl LinkState {
    fn unwrap_unmodified(self) -> Cid {
        match self {
            LinkState::Unmodified(k) => k,
            _ => panic!("expected link to be unmodified"),
        }
    }
}

impl<T, Store> Serialize for Link<T, Store>
where
    T: Serialize,
    Store: StaticStore,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let k = self.save().map_err(S::Error::custom)?;
        Serialize::serialize(&k, serializer)
    }
}

impl<'de, T, Store> Deserialize<'de> for Link<T, Store>
where
    T: Deserialize<'de>,
    Store: StaticStore,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::new(Cid::deserialize(deserializer)?))
    }
}

impl<T, Store> From<T> for Link<T, Store>
where
    Store: StaticStore,
{
    fn from(c: T) -> Self {
        Self::from_value(c, None)
    }
}

impl<T, Store> Deref for Link<T, Store>
where
    T: DeserializeOwned,
    Store: MagicStore,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        Store::unwrap(self.read())
    }
}

impl<T, Store> DerefMut for Link<T, Store>
where
    T: DeserializeOwned + Serialize,
    Store: MagicStore,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        Store::unwrap(self.edit())
    }
}

impl<T, Store> Link<T, Store> {
    /// Construct a new `Link` from a `Cid`.
    #[must_use]
    pub const fn new(k: Cid) -> Self {
        Self {
            state: Cell::new(LinkState::Unmodified(k)),
            value: OnceCell::new(),
            _marker: PhantomData,
        }
    }

    /// Construct a new `Link` from a value (with an optional link-shape hint).
    #[must_use]
    pub fn from_value(v: T, shape: Option<CidShape>) -> Self {
        Self {
            state: Cell::new(LinkState::Modified(shape)),
            value: OnceCell::from(v),
            _marker: PhantomData,
        }
    }

    /// Read the linked object. This will automatically load and decode the underlying data if
    /// it isn't cached.
    pub fn read(&self) -> Result<&T, Store::Error>
    where
        T: DeserializeOwned,
        Store: StaticStore,
    {
        self.value
            .get_or_try_init(|| Store::load(&self.state.get().unwrap_unmodified()))
    }

    /// Edit the linked object. Like [`Link::read`], this will automatically load and decode the
    /// object. Additionally, it will mark it as "modified" ensuring: the modified value will be
    /// persisted when this object is next serialized or `Link::save` is called.
    pub fn edit(&mut self) -> Result<&mut T, Store::Error>
    where
        T: DeserializeOwned + Serialize,
        Store: StaticStore,
    {
        if let LinkState::Unmodified(k) = self.state.get() {
            if self.value.get().is_none() {
                self.value = OnceCell::from(Store::load::<T>(&k)?);
            }
            self.state = Cell::new(LinkState::Modified(Some(CidShape::from(&k))));
        }
        Ok(self.value.get_mut().expect("expected value"))
    }

    /// Write-back the value if modified, and return the CID. Links are automatically "saved" when
    /// serialized, so you only need to call this to store the root object.
    pub fn save(&self) -> Result<Cid, Store::Error>
    where
        T: Serialize,
        Store: StaticStore,
    {
        let shape = match self.state.get() {
            LinkState::Unmodified(k) => return Ok(k),
            LinkState::Modified(shape) => shape,
        };

        let k = Store::store(
            self.value.get().expect("modified link has no value"),
            shape.as_ref(),
        )?;
        self.state.set(LinkState::Unmodified(k));
        Ok(k)
    }

    /// Write-back the value if modified, return the CID, and drop any cached values.
    pub fn free(&mut self) -> Result<Cid, Store::Error>
    where
        T: Serialize,
        Store: StaticStore,
    {
        let k = self.save()?;
        self.value = OnceCell::new();
        Ok(k)
    }
}

#[cfg(test)]
mod test {
    use std::marker::PhantomData;

    use cid::Cid;
    use serde::{Deserialize, Serialize};

    use crate::{Link, MagicStore, StaticStore};

    // TODO Having the store here is _really_ annoying. We might just want to remove it entirely.

    #[derive(Deserialize, Serialize)]
    #[serde(bound = "")] // ugh. The generic parameter really needs to go.
    struct DataObject<Store: StaticStore> {
        field1: String,
        field2: String,
        _marker: PhantomData<fn(Store)>, // future proof. ew.
    }
    #[derive(Deserialize, Serialize)]
    #[serde(bound = "")]
    struct State<Store: StaticStore> {
        name: String,
        data1: Link<DataObject<Store>, Store>,
        data2: Link<DataObject<Store>, Store>,
    }

    // Having to specify `MagicStore` here kind of defeats the point of abstracting over different
    // store types.
    impl<Store: MagicStore> State<Store> {
        pub fn set_data1_field1(&mut self, field1: String) {
            // Lazily loads `data1`, modifies, it, and marks it as dirty (because we mutably
            // dereference it).
            self.data1.field1 = field1
        }

        pub fn save(&self) -> Cid {
            // Saves the object, returning a CID. This will internally save any _modified_ objects.
            Store::unwrap(Store::store(self, None))
        }
    }
}
