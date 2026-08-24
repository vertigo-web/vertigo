use std::{
    collections::{HashMap, HashSet},
    hash::{BuildHasherDefault, Hasher},
};

use super::NodeId;

/// Hasher for the maps keyed by [`NodeId`].
///
/// The default `SipHash` is built to survive adversarial keys. Nothing here is
/// adversarial - a `NodeId` is a counter this graph handed out - and these maps sit on
/// the hot path: one `get` during a wave asks `Dirty` several separate questions about
/// the same id, and a node that reads many parents repeats that per parent. This is
/// splitmix64's finaliser, which spreads a counter over the buckets in a few
/// instructions instead of a keyed permutation.
#[derive(Default)]
pub(super) struct NodeHasher(u64);

impl Hasher for NodeHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write_u64(&mut self, value: u64) {
        let mut z = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        self.0 = z ^ (z >> 31);
    }

    /// `NodeId` is one `u64`, so this never runs for the maps below. It is here because
    /// `Hasher` requires it, and it has to fold rather than overwrite to stay a hasher.
    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.write_u64(self.0 ^ u64::from(*byte));
        }
    }
}

pub(super) type NodeSet = HashSet<NodeId, BuildHasherDefault<NodeHasher>>;
pub(super) type NodeMap<V> = HashMap<NodeId, V, BuildHasherDefault<NodeHasher>>;

#[cfg(test)]
mod tests {
    use super::*;

    fn hash_of(id: NodeId) -> u64 {
        use std::hash::Hash;
        let mut hasher = NodeHasher::default();
        id.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn distinct_ids_hash_apart() {
        // Consecutive ids are the whole input distribution here, so they are what must
        // not collide - or land in a run of neighbouring buckets.
        let hashes: HashSet<u64> = (0..1000).map(|i| hash_of(NodeId(i))).collect();
        assert_eq!(hashes.len(), 1000);

        let low_bits: HashSet<u64> = (0..64).map(|i| hash_of(NodeId(i)) & 0x3f).collect();
        assert!(low_bits.len() > 32, "bucket index must vary: {low_bits:?}");
    }

    #[test]
    fn the_same_id_hashes_the_same() {
        assert_eq!(hash_of(NodeId(7)), hash_of(NodeId(7)));
    }

    #[test]
    fn map_round_trip() {
        let mut map: NodeMap<u32> = NodeMap::default();
        for i in 0..100 {
            map.insert(NodeId(i), i as u32);
        }
        for i in 0..100 {
            assert_eq!(map.get(&NodeId(i)), Some(&(i as u32)));
        }
        assert_eq!(map.get(&NodeId(100)), None);
    }
}
