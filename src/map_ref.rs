use crate::iter::*;
use crate::HashMap;
use crossbeam_epoch::Guard;
use std::borrow::Borrow;
use std::fmt::{self, Debug, Formatter};
use std::hash::{BuildHasher, Hash};
use std::ops::{Deref, Index};

/// A reference to a [`HashMap`], constructed with [`HashMap::guarded`] or [`HashMap::with_guard`].
/// The current thread will be pinned for the duration of this reference.
pub struct HashMapRef<'map, K: 'static, V: 'static, S = crate::DefaultHashBuilder> {
    map: &'map HashMap<K, V, S>,
    guard: GuardRef<'map>,
}

enum GuardRef<'g> {
    Owned(Guard),
    Ref(&'g Guard),
}

impl Deref for GuardRef<'_> {
    type Target = Guard;

    #[inline]
    fn deref(&self) -> &Guard {
        match *self {
            GuardRef::Owned(ref guard) | GuardRef::Ref(&ref guard) => guard,
        }
    }
}

impl<K, V, S> HashMap<K, V, S>
where
    K: Sync + Send + Clone + Hash + Eq,
    V: Sync + Send,
    S: BuildHasher,
{
    /// Get a reference to this map with the current thread pinned.
    pub fn guarded(&self) -> HashMapRef<'_, K, V, S> {
        HashMapRef {
            guard: GuardRef::Owned(self.guard()),
            map: &self,
        }
    }

    /// Get a reference to this map with the given guard.
    pub fn with_guard<'g>(&'g self, guard: &'g Guard) -> HashMapRef<'g, K, V, S> {
        HashMapRef {
            map: &self,
            guard: GuardRef::Ref(guard),
        }
    }
}

impl<K, V, S> HashMapRef<'_, K, V, S>
where
    K: Sync + Send + Clone + Hash + Eq,
    V: Sync + Send,
    S: BuildHasher,
{
    /// Tests if `key` is a key in this table.
    /// See also [`HashMap::contains_key`].
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: ?Sized + Hash + Eq,
    {
        self.map.contains_key(key, &self.guard)
    }

    /// Returns the value to which `key` is mapped.
    /// See also [`HashMap::get`].
    pub fn get<'g, Q>(&'g self, key: &Q) -> Option<&'g V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Hash + Eq,
    {
        self.map.get(key, &self.guard)
    }

    /// Returns the key-value pair corresponding to `key`.
    /// See also [`HashMap::get_key_value`].
    pub fn get_key_value<'g, Q>(&'g self, key: &Q) -> Option<(&'g K, &'g V)>
    where
        K: Borrow<Q>,
        Q: ?Sized + Hash + Eq,
    {
        self.map.get_key_value(key, &self.guard)
    }

    /// Maps `key` to `value` in this table.
    /// See also [`HashMap::insert`].
    pub fn insert<'g>(&'g self, key: K, value: V) -> Option<&'g V> {
        self.map.insert(key, value, &self.guard)
    }

    /// If the value for the specified `key` is present, attempts to
    /// compute a new mapping given the key and its current mapped value.
    /// See also [`HashMap::compute_if_present`].
    pub fn compute_if_present<'g, Q, F>(&'g self, key: &Q, remapping_function: F) -> Option<&'g V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Hash + Eq,
        F: FnOnce(&K, &V) -> Option<V>,
    {
        self.map
            .compute_if_present(key, remapping_function, &self.guard)
    }

    /// Tries to reserve capacity for at least additional more elements.
    /// See also [`HashMap::reserve`].
    pub fn reserve(&self, additional: usize) {
        self.map.reserve(additional, &self.guard)
    }

    /// Removes the key (and its corresponding value) from this map.
    /// See also [`HashMap::remove`].
    pub fn remove<'g, Q>(&'g self, key: &Q) -> Option<&'g V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Hash + Eq,
    {
        self.map.remove(key, &self.guard)
    }

    /// Retains only the elements specified by the predicate.
    /// See also [`HashMap::retain`].
    pub fn retain<F>(&self, f: F)
    where
        F: FnMut(&K, &V) -> bool,
    {
        self.map.retain(f, &self.guard);
    }

    /// Retains only the elements specified by the predicate.
    /// See also [`HashMap::retain_force`].
    pub fn retain_force<F>(&self, f: F)
    where
        F: FnMut(&K, &V) -> bool,
    {
        self.map.retain_force(f, &self.guard);
    }

    /// An iterator visiting all key-value pairs in arbitrary order.
    /// The iterator element type is `(&'g K, &'g V)`.
    /// See also [`HashMap::iter`].
    pub fn iter<'g>(&'g self) -> Iter<'g, K, V> {
        self.map.iter(&self.guard)
    }

    /// An iterator visiting all keys in arbitrary order.
    /// The iterator element type is `&'g K`.
    /// See also [`HashMap::keys`].
    pub fn keys<'g>(&'g self) -> Keys<'g, K, V> {
        self.map.keys(&self.guard)
    }

    /// An iterator visiting all values in arbitrary order.
    /// The iterator element type is `&'g V`.
    /// See also [`HashMap::values`].
    pub fn values<'g>(&'g self) -> Values<'g, K, V> {
        self.map.values(&self.guard)
    }

    /// Returns the number of entries in the map.
    /// See also [`HashMap::len`].
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns `true` if the map is empty. Otherwise returns `false`.
    /// See also [`HashMap::is_empty`].
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

impl<'g, K, V, S> IntoIterator for &'g HashMapRef<'_, K, V, S>
where
    K: Sync + Send + Clone + Hash + Eq,
    V: Sync + Send,
    S: BuildHasher,
{
    type IntoIter = Iter<'g, K, V>;
    type Item = (&'g K, &'g V);

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter(&self.guard)
    }
}

impl<K, V, S> Debug for HashMapRef<'_, K, V, S>
where
    K: Sync + Send + Clone + Hash + Eq + Debug,
    V: Sync + Send + Debug,
    S: BuildHasher,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self).finish()
    }
}

impl<K, V, S> Clone for HashMapRef<'_, K, V, S>
where
    K: Sync + Send + Clone + Hash + Eq,
    V: Sync + Send,
    S: BuildHasher,
{
    fn clone(&self) -> Self {
        self.map.guarded()
    }
}

impl<K, V, S> PartialEq for HashMapRef<'_, K, V, S>
where
    K: Sync + Send + Clone + Hash + Eq,
    V: Sync + Send + PartialEq,
    S: BuildHasher,
{
    fn eq(&self, other: &Self) -> bool {
        self.map.guarded_eq(&other.map, &self.guard, &other.guard)
    }
}

impl<K, V, S> PartialEq<HashMap<K, V, S>> for HashMapRef<'_, K, V, S>
where
    K: Sync + Send + Clone + Hash + Eq,
    V: Sync + Send + PartialEq,
    S: BuildHasher,
{
    fn eq(&self, other: &HashMap<K, V, S>) -> bool {
        self.map.guarded_eq(&other, &self.guard, &other.guard())
    }
}

impl<K, V, S> PartialEq<HashMapRef<'_, K, V, S>> for HashMap<K, V, S>
where
    K: Sync + Send + Clone + Hash + Eq,
    V: Sync + Send + PartialEq,
    S: BuildHasher,
{
    fn eq(&self, other: &HashMapRef<'_, K, V, S>) -> bool {
        self.guarded_eq(&other.map, &self.guard(), &other.guard)
    }
}

impl<K, V, S> Eq for HashMapRef<'_, K, V, S>
where
    K: Sync + Send + Clone + Hash + Eq,
    V: Sync + Send + Eq,
    S: BuildHasher,
{
}

impl<K, Q, V, S> Index<&'_ Q> for HashMapRef<'_, K, V, S>
where
    K: Sync + Send + Clone + Hash + Eq + Borrow<Q>,
    Q: ?Sized + Hash + Eq,
    V: Sync + Send,
    S: BuildHasher,
{
    type Output = V;

    fn index(&self, key: &Q) -> &V {
        self.get(key).expect("no entry found for key")
    }
}
