use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// `ViewId` is a unique identifier for a view. We're using a u64 and hashing
/// under the assumption there won't be collisions. The underlying u64 is a function
/// of the path down the view tree.
#[derive(Copy, Clone, Default, Eq, PartialEq, Hash, Debug)]
pub struct ViewId {
    pub id: u64,
}

impl ViewId {
    pub fn is_default(self) -> bool {
        self == ViewId::default()
    }
}

pub type IdPath = Vec<u64>;

pub fn hh<H: Hash>(index: &H) -> u64 {
    let mut hasher = DefaultHasher::new();
    index.hash(&mut hasher);
    hasher.finish()
}
