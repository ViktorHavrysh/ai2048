use std::collections::HashMap;
use std::hash::Hash;
use std::rc::{Rc, Weak};
use std::cell::RefCell;

type CachingHashMap<K, V> = HashMap<K, Weak<V>>;

pub struct Cache<K, V> {
    data: RefCell<CachingHashMap<K, V>>,
}

impl<K, V> Cache<K, V>
    where K: Eq + Hash + Clone
{
    pub fn new() -> Cache<K, V> {
        Cache { data: RefCell::new(HashMap::new()) }
    }

    pub fn get_or_insert_with<F: FnOnce() -> V>(&self, key: K, default: F) -> Rc<V> {
        self.data.borrow_mut().get_or_insert_with(key, default)
    }

    pub fn strong_count(&self) -> usize {
        self.data.borrow().values().filter(|v| v.upgrade().is_some()).count()
    }

    pub fn len(&self) -> usize {
        self.data.borrow().len()
    }

    pub fn gc(&self) {
        self.data.borrow_mut().gc();
    }
}

trait Gc {
    fn gc(&mut self);
}

impl<K, V> Gc for CachingHashMap<K, V>
    where K: Eq + Hash + Clone
{
    fn gc(&mut self) {
        let stale_keys: Vec<K> = self.iter()
            .filter_map(|pair| {
                let (key, value) = pair;
                match value.upgrade() {
                    Some(_) => None,
                    None => Some(key.clone()),
                }
            })
            .collect();

        for key in stale_keys {
            self.remove(&key);
        }
    }
}

trait GetOrSet<K, V> {
    fn get_or_insert_with<F: FnOnce() -> V>(&mut self, key: K, default: F) -> Rc<V>;
}

impl<K, V> GetOrSet<K, V> for CachingHashMap<K, V>
    where K: Eq + Hash
{
    fn get_or_insert_with<F: FnOnce() -> V>(&mut self, key: K, default: F) -> Rc<V> {
        match self.get(&key).and_then(|v| v.upgrade()) {
            Some(value) => value,
            None => {
                let value = Rc::new(default());
                self.insert(key, Rc::downgrade(&value));
                value
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Cache, CachingHashMap, Gc};

    use std::rc::Rc;

    #[test]
    fn cachinghashmap_can_gc() {
        let mut hashmap = CachingHashMap::new();
        let rc_kept = Rc::new(1);
        hashmap.insert(1, Rc::downgrade(&rc_kept));

        {
            let rc_destroyed = Rc::new(2);
            hashmap.insert(2, Rc::downgrade(&rc_destroyed));
        }

        assert_eq!(1,
                   hashmap.values().filter(|v| v.upgrade().is_some()).count());
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
        assert_eq!(0, cache.strong_count())        ;
        assert_eq!(1, cache.len());

        cache.gc();
        assert_eq!(0, cache.len());

        let value = cache.get_or_insert_with(1, || 3);
        assert_eq!(3, *value);
    }
}
