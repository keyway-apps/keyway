use crate::HashSet;
use core::fmt;
use std::{borrow::Borrow, collections::hash_set, hash::Hash};

#[derive(Clone)]
pub struct PinnedSet<T> {
    set: HashSet<T>,
    pinned: Option<T>,
}

impl<T: Hash + Eq> PinnedSet<T> {
    pub fn new() -> Self {
        Self {
            set: HashSet::default(),
            pinned: None,
        }
    }

    pub fn pinned(&self) -> Option<&T> {
        self.pinned.as_ref()
    }

    pub fn take_pinned(&mut self) -> Option<T> {
        self.pinned.take()
    }

    pub fn is_pinned<Q>(&self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.pinned.as_ref().map_or(false, |p| p.borrow() == value)
    }

    pub fn set_pinned(&mut self, value: Option<T>) {
        let same = match (&value, &self.pinned) {
            (Some(new), Some(old)) => new == old,
            (None, None) => true,
            _ => false,
        };
        if same {
            return;
        }

        if let Some(ref v) = value {
            self.set.remove(v);
        }
        if let Some(old) = std::mem::replace(&mut self.pinned, value) {
            self.set.insert(old);
        }
    }

    pub fn insert(&mut self, value: T) -> bool {
        if self.is_pinned(&value) {
            return false;
        }
        self.set.insert(value)
    }

    pub fn remove<Q>(&mut self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if self.is_pinned(value) {
            self.pinned = None;
            return true;
        }
        self.set.remove(value)
    }

    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.is_pinned(value) || self.set.contains(value)
    }

    pub fn get<Q>(&self, value: &Q) -> Option<&T>
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(p) = &self.pinned {
            if p.borrow() == value {
                return Some(p);
            }
        }
        self.set.get(value)
    }

    pub fn len(&self) -> usize {
        self.set.len() + self.pinned.is_some() as usize
    }

    pub fn is_empty(&self) -> bool {
        self.pinned.is_none() && self.set.is_empty()
    }

    pub fn clear(&mut self) {
        self.set.clear();
        self.pinned = None;
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        if matches!(&self.pinned, Some(p) if !f(p)) {
            self.pinned = None;
        }
        self.set.retain(f);
    }

    /// 迭代所有元素 (pinned 在前, 其余按 HashSet 顺序)
    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.pinned.iter().chain(self.set.iter())
    }
}

impl<T: Hash + Eq> Default for PinnedSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Hash + Eq> FromIterator<T> for PinnedSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut s = Self::new();
        s.extend(iter);
        s
    }
}

impl<T: Hash + Eq> Extend<T> for PinnedSet<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for v in iter {
            self.insert(v);
        }
    }
}

impl<T: Hash + Eq> IntoIterator for PinnedSet<T> {
    type Item = T;
    type IntoIter = std::iter::Chain<std::option::IntoIter<T>, hash_set::IntoIter<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.pinned.into_iter().chain(self.set.into_iter())
    }
}

impl<'a, T: Hash + Eq> IntoIterator for &'a PinnedSet<T> {
    type Item = &'a T;
    type IntoIter = std::iter::Chain<std::option::Iter<'a, T>, hash_set::Iter<'a, T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.pinned.iter().chain(self.set.iter())
    }
}

impl<T: Hash + Eq> PartialEq for PinnedSet<T> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().all(|x| other.contains(x))
    }
}

impl<T: Hash + Eq> Eq for PinnedSet<T> {}

impl<T: Hash + Eq + fmt::Debug> fmt::Debug for PinnedSet<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PinnedSet")
            .field("pinned", &self.pinned)
            .field("set", &self.set)
            .finish()
    }
}
