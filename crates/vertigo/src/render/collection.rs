use std::hash::Hash;

/// Describes how items in a reactive list are identified.
///
/// Implement this on a zero-sized **marker type** (not on the item itself). The
/// marker is the generic parameter threaded through the keyed-collection
/// machinery: [`render_list_memo`](crate::render::render_list_memo),
/// [`render_resource_list_memo`](crate::render::render_resource_list_memo) (via
/// [`keyed_computed_list`](crate::keyed_computed_list)) and
/// [`LazyListCache`](crate::LazyListCache).
///
/// The associated [`Key`](CollectionKey::Key) gives each item a stable identity.
/// This is what enables per-item **memoization** (an item keeps the same inner
/// reactive cell across list updates as long as its key is stable, so only items
/// whose content actually changed are re-rendered) and **deduplication**
/// (items sharing a key within one list are reported and skipped).
///
/// ```rust
/// use vertigo::CollectionKey;
///
/// #[derive(Clone, PartialEq, Eq, Debug)]
/// struct Item { id: u32, name: String }
///
/// struct ItemKey; // marker type
///
/// impl CollectionKey for ItemKey {
///     type Key = u32;
///     type Value = Item;
///     fn get_key(val: &Item) -> u32 { val.id }
/// }
/// ```
pub trait CollectionKey {
    /// Stable identity extracted from an item.
    type Key: Eq + Hash + Clone + std::fmt::Debug + 'static;
    /// The item type stored in the list.
    type Value: Clone + PartialEq + 'static;
    /// Extract the key that identifies `val`.
    fn get_key(val: &Self::Value) -> Self::Key;
}
