use crate::entities::Entity;
use crate::search::{BitapSearcher, NgramIndexer, Searchable};
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

pub struct Id<T: Entity + ?Sized> {
    value: String,
    _marker: PhantomData<T>,
}

impl<T: Entity + ?Sized> Id<T> {
    /// Creates a new [`Id`] with the given string value.
    pub fn new(value: String) -> Self {
        Id {
            value,
            _marker: PhantomData,
        }
    }

    /// Generates a new [`Id`] with a random Ulid.
    #[cfg(feature = "ulid")]
    pub fn new_ulid(&self) -> Self {
        Self::new(ulid::Ulid::new().to_string())
    }

    /// Returns the string value of the ID.
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl<T: Entity + ?Sized> Debug for Id<T> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let Id { value, _marker } = self;

        f.debug_struct("Id")
            .field("value", &value)
            .field("_marker", &_marker)
            .finish()
    }
}

impl<T: Entity + ?Sized> Clone for Id<T> {
    fn clone(&self) -> Self {
        Id {
            value: self.value.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T: Entity + ?Sized> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Entity + ?Sized> Eq for Id<T> {}

impl<T: Entity + ?Sized> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Entity + ?Sized> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl<T: Entity + ?Sized> Hash for Id<T> {
    fn hash<H: Hasher>(&self, ra_expand_state: &mut H) {
        let Id { value, _marker } = self;

        value.hash(ra_expand_state);
        _marker.hash(ra_expand_state);
    }
}

impl<T: Entity + ?Sized> Searchable for Id<T> {
    fn index(&self, indexer: &mut NgramIndexer) {
        indexer.index(self.value());
    }

    fn get_score(&self, searcher: &BitapSearcher) -> Option<f32> {
        searcher.get_score(self.value())
    }
}

impl<T: Entity + ?Sized> Display for Id<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

#[cfg(feature = "bincode")]
impl<T: Entity + ?Sized> bincode::Encode for Id<T> {
    fn encode<__E: bincode::enc::Encoder>(
        &self,
        encoder: &mut __E,
    ) -> Result<(), bincode::error::EncodeError> {
        bincode::Encode::encode(&self.value, encoder)?;

        Ok(())
    }
}

#[cfg(feature = "bincode")]
impl<T: Entity + ?Sized, __Context> bincode::Decode<__Context> for Id<T> {
    fn decode<__D: bincode::de::Decoder<Context = __Context>>(
        decoder: &mut __D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Ok(Self {
            value: bincode::Decode::decode(decoder)?,
            _marker: PhantomData,
        })
    }
}

#[cfg(feature = "bincode")]
impl<'__de, T: Entity + ?Sized, __Context> bincode::BorrowDecode<'__de, __Context> for Id<T> {
    fn borrow_decode<__D: bincode::de::BorrowDecoder<'__de, Context = __Context>>(
        decoder: &mut __D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Ok(Self {
            value: bincode::BorrowDecode::<'_, __Context>::borrow_decode(decoder)?,
            _marker: PhantomData,
        })
    }
}

#[cfg(feature = "serde")]
impl<T: Entity + ?Sized> serde::Serialize for Id<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.value)
    }
}

#[cfg(feature = "serde")]
struct IdVisitor<T: Entity + ?Sized> {
    marker: PhantomData<T>,
}

#[cfg(feature = "serde")]
impl<'de, T: Entity + ?Sized> serde::de::Visitor<'de> for IdVisitor<T> {
    type Value = Id<T>;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a string representing an ID")
    }

    #[inline]
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Id {
            value: v.to_string(),
            _marker: PhantomData,
        })
    }
}

#[cfg(feature = "serde")]
impl<'de, T: Entity + ?Sized> serde::Deserialize<'de> for Id<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(IdVisitor {
            marker: PhantomData::<T>,
        })
    }
}
