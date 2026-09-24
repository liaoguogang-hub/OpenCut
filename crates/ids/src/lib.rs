//! Opaque identity types for OpenCut editor-core entities.
//!
//! This crate is intentionally dependency-free *within* the workspace: it
//! depends on no other OpenCut crate, only `serde` (an external foundation
//! dependency) for stable serialization. That property is load-bearing — it
//! is what lets `clips` reference a `TimelineId` (for `CompoundClip`) without
//! pulling in the `timelines` crate, which would otherwise create a cycle
//! (`timelines` depends on `clips` for `Track.clips`). See
//! `architecture.mdc`, "Editor core — Compound clips".
//!
//! Ids are newtype wrappers around a `u64` minted by the container that owns
//! the entity (a `Timeline` mints `TrackId`s, a `Project` mints `TimelineId`s
//! and `ClipId`s). Callers never hand-construct an id for a new entity — they
//! receive one back from the creation method. The inner `u64` is `pub` only so
//! a different crate can construct the newtype via `TrackId(n)`; treat the
//! value as opaque everywhere else.

use serde::{Deserialize, Serialize};

/// Derives the standard trait set every id newtype needs: cheap copy, debug,
/// hash/eq for use as map keys and in collections, and serde for persistence.
macro_rules! id_type {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub u64);

        impl $name {
            /// The underlying counter value. Treat as opaque outside the
            /// minting container.
            pub fn raw(self) -> u64 {
                self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }
    };
}

id_type!(ProjectId);
id_type!(TimelineId);
id_type!(TrackId);
id_type!(ClipId);
id_type!(AssetId);
id_type!(BindingId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_transparent_in_serialization() {
        let id = TimelineId(42);
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "42");
        let back: TimelineId = serde_json::from_str(&json).unwrap();
        assert_eq!(back, id);
    }

    #[test]
    fn ids_compare_and_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(ClipId(1));
        set.insert(ClipId(2));
        assert!(set.contains(&ClipId(1)));
        assert_eq!(ClipId(1), ClipId(1));
    }
}
