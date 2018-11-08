//! This cache is intended as an alternative to something like `ConditionalWeakTable` from C#.
//!
//! It is a wrapper around `HashMap`.
//!
//! It uses interior mutability so it can be used in a field of an immutable type. It has a single
//! way of retrieving data from it that requires you to provide a way to put it there if it
//! doesn't already exist and returns an `Rc<T>`, while it stores a `Weak<T>` reference
//! inside. When all the copies of the Rc<T> go out of scope, the key is considered not to exist
//! in the cache. However, internally, the Weak<T> reference still exists, so if you want to
//! reclaim the memory taken by that, you need to call the `gc()` method.
//!
//! The type is not thread safe.

use fnv::FnvHashMap;
use std::cell::RefCell;
use std::hash::Hash;
use std::rc::{Rc, Weak};

pub struct Cache<K, V> {
    data: RefCell<CachingHashMap<K, V>>,
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash + Clone,
{
    /// Returns an emtpy `Cache`.
    pub fn new() -> Self {
        Cache {
            data: RefCell::new(FnvHashMap::default()),
        }
    }

    /// Retrieves the cached value by key. If the value doesn't exist, uses the provided
    /// closure to create it, stores in the cache, and then returns the value anyway.
    pub fn get_or_insert_with<F: FnOnce() -> V>(&self, key: K, default: F) -> Rc<V> {
        self.data.borrow_mut().get_or_insert_with(key, default)
    }

    /// Returns the number of non-invalidated values that are stored in the cache.
    pub fn strong_count(&self) -> usize {
        self.data
            .borrow()
            .values()
            .filter(|v| v.upgrade().is_some())
            .count()
    }

    /// Returns the length of the inner `HashMap` together with invalidated, but not cleaned,
    /// references.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.data.borrow().len()
    }

    /// Deletes all the invalidated references.
    pub fn gc(&self) {
        self.data.borrow_mut().gc();
    }
}

type CachingHashMap<K, V> = FnvHashMap<K, Weak<V>>;

trait Gc {
    fn gc(&mut self);
}

impl<K, V> Gc for CachingHashMap<K, V>
where
    K: Eq + Hash + Clone,
{
    fn gc(&mut self) {
        let stale_keys = self.iter()
            .filter_map(|(key, value)| match value.upgrade() {
                Some(_) => None,
                None => Some(key.clone()),
            })
            .collect::<Vec<_>>();

        for key in stale_keys {
            self.remove(&key);
        }
    }
}

trait GetOrInsert<K, V> {
    fn get_or_insert_with<F>(&mut self, key: K, default: F) -> Rc<V>
    where
        F: FnOnce() -> V;
}

impl<K, V> GetOrInsert<K, V> for CachingHashMap<K, V>
where
    K: Eq + Hash,
{
    fn get_or_insert_with<F>(&mut self, key: K, default: F) -> Rc<V>
    where
        F: FnOnce() -> V,
    {
        self.get(&key).and_then(|v| v.upgrade()).unwrap_or_else(|| {
            let value = Rc::new(default());
            self.insert(key, Rc::downgrade(&value));
            value
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Cache, CachingHashMap, Gc};
    use std::rc::Rc;

    #[test]
    fn cachinghashmap_can_gc() {
        let mut hashmap = CachingHashMap::default();
        let rc_kept = Rc::new(1);
        hashmap.insert(1, Rc::downgrade(&rc_kept));

        {
            let rc_destroyed = Rc::new(2);
            hashmap.insert(2, Rc::downgrade(&rc_destroyed));
        }

        let some_count = hashmap.values().filter(|v| v.upgrade().is_some()).count();

        assert_eq!(1, some_count);
        assert_eq!(2, hashmap.len());

        hashmap.gc();
        assert_eq!(1, hashmap.len());
    }

    #[test]
    fn can_get_or_insert() {
        let cache = Cache::new();
        {
            let value = cache.get_or_insert_with(1, || 1);
            assert_eq!(1, *value);
            let value = cache.get_or_insert_with(1, || 2);
            assert_eq!(1, *value);
            assert_eq!(1, cache.strong_count());
            assert_eq!(1, cache.len());
        }
        assert_eq!(0, cache.strong_count());
        assert_eq!(1, cache.len());

        cache.gc();
        assert_eq!(0, cache.len());

        let value = cache.get_or_insert_with(1, || 3);
        assert_eq!(3, *value);
    }
}
