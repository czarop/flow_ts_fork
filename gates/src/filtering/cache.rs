use std::sync::Arc;

/// Cache key for filtered event indices
///
/// This key uniquely identifies a cached filter result based on:
/// - The file being filtered
/// - The gate being applied
/// - The parent gate chain (for hierarchical filtering)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FilterCacheKey {
    /// File GUID
    pub file_guid: Arc<str>,
    /// Gate ID
    pub gate_id: Arc<str>,
    /// Parent gate chain (for hierarchical filtering)
    /// Stored as a sorted, deduplicated list for consistent hashing
    pub parent_chain: Vec<Arc<str>>,
}

impl FilterCacheKey {
    /// Create a new cache key
    pub fn new(
        file_guid: impl Into<Arc<str>>,
        gate_id: impl Into<Arc<str>>,
        parent_chain: Vec<impl Into<Arc<str>>>,
    ) -> Self {
        let mut chain: Vec<Arc<str>> = parent_chain.into_iter().map(|s| s.into()).collect();
        chain.sort();
        chain.dedup();

        Self {
            file_guid: file_guid.into(),
            gate_id: gate_id.into(),
            parent_chain: chain,
        }
    }

    /// Create a simple key without parent chain
    pub fn simple(file_guid: impl Into<Arc<str>>, gate_id: impl Into<Arc<str>>) -> Self {
        Self {
            file_guid: file_guid.into(),
            gate_id: gate_id.into(),
            parent_chain: Vec::new(),
        }
    }
}

/// Trait for caching filtered event indices
///
/// This trait allows the filtering system to work with any cache implementation.
/// The application crate should implement this trait for its FilterCache type.
pub trait FilterCache: Send + Sync {
    /// Get cached filtered indices for a key
    ///
    /// Returns `Some(Arc<Vec<usize>>)` if the value is cached, `None` otherwise
    fn get(&self, key: &FilterCacheKey) -> Option<Arc<Vec<usize>>>;

    /// Insert filtered indices into the cache
    ///
    /// # Arguments
    /// * `key` - Cache key
    /// * `value` - Filtered event indices to cache
    fn insert(&self, key: FilterCacheKey, value: Arc<Vec<usize>>);
}
