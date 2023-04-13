//! Implementations of the [`Serialize`] and [`Deserialize`] traits
#![cfg(feature = "serde_compat")]

// This module is behind a feature flag: make sure to use `cargo build --all-features` to check that it compiles!
use crate::{PetitMap, PetitSet};
use core::marker::PhantomData;
use serde::{
    de::{SeqAccess, Visitor},
    ser::SerializeSeq,
    Deserialize, Serialize,
};
use std::fmt;

mod petitmap {
    use super::*;

    impl<K: Serialize, V: Serialize, const CAP: usize> Serialize for PetitMap<K, V, CAP> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            // This must be serialized as a sequence, or gaps will be lost
            let mut seq = serializer.serialize_seq(Some(CAP))?;
            for i in 0..CAP {
                seq.serialize_element(&self.storage[i])?;
            }
            seq.end()
        }
    }

    impl<'de, K: Deserialize<'de> + Eq, V: Deserialize<'de>, const CAP: usize> Deserialize<'de>
        for PetitMap<K, V, CAP>
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            // This should be deserialized as a sequence, or gaps will be lost
            deserializer.deserialize_seq(PetitMapVisitor::new())
        }
    }

    #[derive(Debug)]
    struct PetitMapVisitor<K, V, const CAP: usize> {
        marker: PhantomData<fn() -> PetitMap<K, V, CAP>>,
    }

    impl<K, V, const CAP: usize> PetitMapVisitor<K, V, CAP> {
        fn new() -> Self {
            PetitMapVisitor {
                marker: PhantomData,
            }
        }
    }

    impl<'de, K, V, const CAP: usize> Visitor<'de> for PetitMapVisitor<K, V, CAP>
    where
        K: Deserialize<'de> + Eq,
        V: Deserialize<'de>,
    {
        type Value = PetitMap<K, V, CAP>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an array of `Option<T>` values to create a PetitMap.")
        }

        /// Deserialize `PetitMap` from an abstract "sequence" provided by the `Deserializer`.
        fn visit_seq<S>(self, mut access: S) -> Result<Self::Value, S::Error>
        where
            S: SeqAccess<'de>,
        {
            let mut map: PetitMap<K, V, CAP> = PetitMap::default();

            for i in 0..CAP {
                let next_element: Option<Option<(K, V)>> = access.next_element()?;

                // Insert the next element found
                if let Some(element) = next_element {
                    map.storage[i] = element;
                } else {
                    // We have run out of items in the serialized format
                    // before we ran out of capacity.
                    break;
                }
            }

            Ok(map)
        }
    }
}

// The derive macro forces T: Eq bounds on the struct itself, which is undesirable
// So let's write a tighter implementation by hand!
mod petitset {
    use super::*;

    impl<T: Serialize + Clone, const CAP: usize> Serialize for PetitSet<T, CAP> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let mut seq = serializer.serialize_seq(Some(CAP))?;
            for i in 0..CAP {
                let element: Option<&T> = match &self.map.storage[i] {
                    Some((k, _v)) => Some(k),
                    None => None,
                };

                seq.serialize_element(&element)?;
            }
            seq.end()
        }
    }

    impl<'de, T: Deserialize<'de> + Eq + Clone, const CAP: usize> Deserialize<'de>
        for PetitSet<T, CAP>
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(PetitSetVisitor::new())
        }
    }

    #[derive(Debug)]
    struct PetitSetVisitor<T, const CAP: usize> {
        marker: PhantomData<fn() -> PetitSet<T, CAP>>,
    }

    impl<T, const CAP: usize> PetitSetVisitor<T, CAP> {
        fn new() -> Self {
            PetitSetVisitor {
                marker: PhantomData,
            }
        }
    }

    impl<'de, T, const CAP: usize> Visitor<'de> for PetitSetVisitor<T, CAP>
    where
        T: Deserialize<'de> + Eq + Clone,
    {
        type Value = PetitSet<T, CAP>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an array of `Option<T>` values to create a PetitSet.")
        }

        /// Deserialize `PetitSet` from an abstract "sequence" provided by the `Deserializer`.
        fn visit_seq<S>(self, mut access: S) -> Result<Self::Value, S::Error>
        where
            S: SeqAccess<'de>,
        {
            let mut set: PetitSet<T, CAP> = PetitSet::default();

            for i in 0..CAP {
                let next_element: Option<Option<T>> = access.next_element()?;

                // If another element was found in the serialized format
                // process and insert it
                if let Some(element) = next_element {
                    set.map.storage[i] = element.map(|e| (e, ()));
                } else {
                    // We have run out of items in the serialized format
                    // before we ran out of capacity.
                    break;
                }
            }

            Ok(set)
        }
    }
}
