/// FastMap is a type alias for a concurrent map implementation, it allows for optional use of dashmap and concurrent access in multi-threaded environments, falling back to a standard HashMap when dashmap is not enabled or in single-threaded environments.
/// `fastmap` is gated at the crate level with `fastmap` so it can be used alone
#[cfg(all(feature = "dashmap", not(feature = "wasm-single-thread")))]
pub type FastMap<K, V> = dashmap::DashMap<K, V>;
#[cfg(all(feature = "dashmap", not(feature = "wasm-single-thread")))]
pub type FastSet<K> = dashmap::DashSet<K>;

// Fallback to HashMap for single-threaded
#[cfg(all(feature = "dashmap", feature = "wasm-single-thread"))]
pub type FastMap<K, V> = std::collections::HashMap<K, V>;
#[cfg(all(feature = "dashmap", feature = "wasm-single-thread"))]
pub type FastSet<K> = std::collections::HashSet<K>;

// Fallback to HashMap when dashmap feature is not enabled (should never be the case, but just in case...)
#[cfg(not(feature = "dashmap"))]
pub type FastMap<K, V> = std::collections::HashMap<K, V>;
#[cfg(not(feature = "dashmap"))]
pub type FastSet<K> = std::collections::HashSet<K>;
