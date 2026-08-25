//! A fast, non-cryptographic hasher for the framework's own maps.
//!
//! The default `SipHash-1-3` is built to survive adversarial keys, and pays for it on every
//! lookup. Several of vertigo's maps sit on a hot path where that price is paid per row per
//! update - a keyed list rebuilds its index whenever the list changes, and
//! [`DriverDom::insert_before`](crate::driver_module::DriverDom) asks a map a question for
//! every node it attaches.
//!
//! **This is not collision-resistant against chosen keys.** Use it for maps whose keys are
//! internal identifiers or application data, not for anything that hashes untrusted input
//! where a pathological key set would matter. List keys are the borderline case: an SSR
//! render can key a list by a name that came from a request, and the worst outcome there is
//! a render that degrades to quadratic. That is a considered trade, the same one rustc and
//! most UI toolkits make for their internal maps.
//!
//! [`NodeHasher`](crate::reactive) in the reactive graph is the specialised sibling of this:
//! it hashes a single `u64` node id and nothing else, so it can be splitmix64's finaliser
//! and skip the folding below entirely.

use std::{
    collections::HashMap,
    hash::{BuildHasherDefault, Hasher},
};

/// FxHash, as used by rustc: fold each word into the state with a rotate, an xor and one
/// multiply.
///
/// The rotate before the xor is what stops a run of low-entropy words from cancelling out,
/// and the rotate in [`finish`](FastHasher::finish) moves the multiply's well-mixed high
/// bits down into the range `HashMap` uses to pick a bucket. Without it, keys that differ
/// only in their top bits would land in the same bucket.
///
/// Public only because it is the default hasher of
/// [`HashMapMut`](crate::dev::HashMapMut), which `vertigo::dev` re-exports; it is not part
/// of the surface anyone is meant to reach for.
#[derive(Default)]
pub struct FastHasher(u64);

/// Fractional part of the golden ratio, scaled to 64 bits - the constant rustc-hash uses.
/// Any odd multiplier with a well-spread bit pattern works; what matters is that it is odd,
/// so the multiply stays a bijection and no two inputs are forced together.
const SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

impl FastHasher {
    #[inline]
    fn add(&mut self, word: u64) {
        self.0 = (self.0.rotate_left(5) ^ word).wrapping_mul(SEED);
    }
}

impl Hasher for FastHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.0.rotate_left(20)
    }

    /// The fallback, for keys that are not a single primitive - `&str` and `String` reach
    /// the hasher through here. Eight bytes at a time, then whatever is left, padded.
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        let mut rest = bytes;

        while let Some((word, tail)) = rest.split_first_chunk::<8>() {
            self.add(u64::from_le_bytes(*word));
            rest = tail;
        }

        if !rest.is_empty() {
            let mut last = [0u8; 8];
            last[..rest.len()].copy_from_slice(rest);
            self.add(u64::from_le_bytes(last));
        }

        // Length matters: without it `[1]` and `[1, 0]` hash the same, because the padding
        // above cannot tell a zero byte from an absent one.
        self.add(bytes.len() as u64);
    }

    #[inline]
    fn write_u8(&mut self, value: u8) {
        self.add(u64::from(value));
    }

    #[inline]
    fn write_u16(&mut self, value: u16) {
        self.add(u64::from(value));
    }

    #[inline]
    fn write_u32(&mut self, value: u32) {
        self.add(u64::from(value));
    }

    #[inline]
    fn write_u64(&mut self, value: u64) {
        self.add(value);
    }

    #[inline]
    fn write_u128(&mut self, value: u128) {
        self.add(value as u64);
        self.add((value >> 64) as u64);
    }

    #[inline]
    fn write_usize(&mut self, value: usize) {
        self.add(value as u64);
    }
}

pub type FastBuildHasher = BuildHasherDefault<FastHasher>;
pub(crate) type FastMap<K, V> = HashMap<K, V, FastBuildHasher>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::HashSet, hash::Hash};

    fn hash_of(value: impl Hash) -> u64 {
        let mut hasher = FastHasher::default();
        value.hash(&mut hasher);
        hasher.finish()
    }

    /// Consecutive integers are the distribution that actually shows up here - list keys are
    /// usually ids from a counter - so they are what must not collide, and must not land in
    /// a run of neighbouring buckets either.
    #[test]
    fn consecutive_integers_spread() {
        let hashes: HashSet<u64> = (0u32..1000).map(hash_of).collect();
        assert_eq!(hashes.len(), 1000);

        let buckets: HashSet<u64> = (0u32..64).map(|value| hash_of(value) & 0x3f).collect();
        assert!(buckets.len() > 32, "bucket index must vary: {buckets:?}");
    }

    #[test]
    fn similar_strings_spread() {
        let hashes: HashSet<u64> = (0..1000).map(|i| hash_of(format!("row-{i:04}"))).collect();
        assert_eq!(hashes.len(), 1000);
    }

    /// The length fold in `write` is what makes this hold.
    #[test]
    fn trailing_zero_bytes_are_not_padding() {
        assert_ne!(hash_of([1u8].as_slice()), hash_of([1u8, 0].as_slice()));
        assert_ne!(hash_of(""), hash_of("\0"));
    }

    #[test]
    fn the_same_value_hashes_the_same() {
        assert_eq!(hash_of(7u32), hash_of(7u32));
        assert_eq!(hash_of("abc"), hash_of("abc"));
    }

    /// Words are folded, not overwritten: a long key must depend on all of itself.
    #[test]
    fn every_word_of_a_long_key_counts() {
        let base = "0123456789abcdef0123456789abcdef";
        for index in 0..base.len() {
            let mut other = base.to_string();
            other.replace_range(index..=index, "X");
            assert_ne!(hash_of(base), hash_of(other.as_str()), "byte {index}");
        }
    }

    #[test]
    fn map_round_trip() {
        let mut map: FastMap<String, u32> = FastMap::default();
        for i in 0..100u32 {
            map.insert(format!("k{i}"), i);
        }
        for i in 0..100u32 {
            assert_eq!(map.get(&format!("k{i}")), Some(&i));
        }
        assert_eq!(map.get("missing"), None);
    }
}
