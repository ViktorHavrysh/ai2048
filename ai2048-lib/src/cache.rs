use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "fnv")] {
        type BuildHasher = fnv::FnvBuildHasher;
    } else if #[cfg(feature = "fxhash")] {
        type BuildHasher = fxhash::FxBuildHasher;
    } else {
        type BuildHasher = std::collections::hash_map::RandomState;
    }
}

cfg_if! {
    if #[cfg(feature = "hashbrown")] {
        pub type Cache<K, V> = hashbrown::HashMap<K, V, BuildHasher>;
    } else if #[cfg(feature = "indexmap")] {
        pub type Cache<K, V> = indexmap::map::IndexMap<K, V, BuildHasher>;
    } else {
        pub type Cache<K, V> = std::collections::HashMap<K, V, BuildHasher>;
    }
}
