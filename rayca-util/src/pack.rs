// Copyright © 2020-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{
    hash::{Hash, Hasher},
    iter::FromIterator,
    marker::PhantomData,
    ops::{Add, AddAssign, Deref, DerefMut},
};

use serde::*;

/// A handle is a sort of index into a vector of elements of a specific kind.
/// It is useful when we do not want to keep a reference to an element,
/// while taking advantage of strong typing to avoid using integers.
///
/// # Safety and Limitations
/// - Handles are not globally unique and may be reused after removal; do not rely on uniqueness over time.
/// - Using a handle after its element has been removed, or after extending a pack, is invalid and may cause panics or incorrect results.
/// - Handles are only valid for the specific `Pack` instance they were created from.
#[repr(C)]
#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Handle<T> {
    pub id: u32,
    /// https://stackoverflow.com/a/50201389
    phantom: PhantomData<T>,
}

unsafe impl<T> Send for Handle<T> {}
unsafe impl<T> Sync for Handle<T> {}

impl<T> Default for Handle<T> {
    fn default() -> Self {
        Self::NONE
    }
}

impl<T> From<usize> for Handle<T> {
    fn from(id: usize) -> Self {
        Self::new(id as u32)
    }
}

impl<T> From<u32> for Handle<T> {
    fn from(id: u32) -> Self {
        Self::new(id)
    }
}

impl<T> From<i32> for Handle<T> {
    fn from(id: i32) -> Self {
        Self::new(id as u32)
    }
}

impl<T> Handle<T> {
    pub const NONE: Self = Self {
        id: u32::MAX,
        phantom: PhantomData,
    };

    pub fn new(id: u32) -> Self {
        Self {
            id,
            phantom: PhantomData,
        }
    }

    /// Modifies the handle by adding an offset to its id.
    pub fn offset(&mut self, offset: u32) {
        if self.is_valid() {
            self.id += offset;
        }
    }

    pub fn is_valid(&self) -> bool {
        self.id != u32::MAX
    }

    pub fn as_index(&self) -> usize {
        self.id as usize
    }

    pub fn is_some(&self) -> bool {
        self.id != u32::MAX
    }

    pub fn is_none(&self) -> bool {
        self.id == u32::MAX
    }
}

impl<'a, T> Handle<T> {
    pub fn get(&self, pack: &'a Pack<T>) -> Option<&'a T> {
        pack.get(*self)
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Handle<T> {}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for Handle<T> {}

impl<T> PartialEq<u32> for Handle<T> {
    fn eq(&self, other: &u32) -> bool {
        self.id.eq(other)
    }
}

impl<T> Hash for Handle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T> Add<u32> for Handle<T> {
    type Output = Self;

    fn add(self, rhs: u32) -> Self::Output {
        Self::new(self.id + rhs)
    }
}

impl<T> AddAssign<u32> for Handle<T> {
    fn add_assign(&mut self, rhs: u32) {
        self.id += rhs;
    }
}

/// A `Pack` is a powerful structure which contains a vector of contiguous elements
/// and a list of indices to those elements. `Handle`s are used to work with `Pack`s.
#[derive(Clone, Default, Serialize)]
#[serde(transparent)]
pub struct Pack<T> {
    /// List of contiguous elements
    vec: Vec<T>,
    /// List of indices to elements
    #[serde(skip)]
    indices: Vec<u32>,
    /// List of positions to free indices
    #[serde(skip)]
    free: Vec<u32>,
}

impl<T> Pack<T> {
    pub fn new() -> Self {
        Self {
            vec: vec![],
            indices: vec![],
            free: vec![],
        }
    }

    /// Returns a vector of all handles in the pack.
    pub fn get_handles(&self) -> Vec<Handle<T>> {
        (0..self.indices.len() as u32).map(Handle::new).collect()
    }

    pub fn as_slice(&self) -> &[T] {
        &self.vec
    }

    pub fn get_indices(&self) -> &Vec<u32> {
        &self.indices
    }

    pub fn push(&mut self, elem: T) -> Handle<T> {
        let index = self.vec.len();
        self.vec.push(elem);

        if !self.free.is_empty() {
            let id = self.free.pop().unwrap();
            self.indices[id as usize] = index as u32;
            Handle::new(id)
        } else {
            let id = self.indices.len();
            self.indices.push(index as u32);
            Handle::new(id as u32)
        }
    }

    fn get_vec_index(&self, handle: Handle<T>) -> Option<usize> {
        // Handle id is an index into the vector of indices
        // The vector of indices contains the actual index of the element
        let vec_index = self.indices.get(handle.id as usize)?;
        Some(*vec_index as usize)
    }

    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        if !handle.is_valid() {
            return None;
        }
        self.vec.get(self.get_vec_index(handle)?)
    }

    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        if !handle.is_valid() {
            return None;
        }
        let vec_index = self.get_vec_index(handle)?;
        self.vec.get_mut(vec_index)
    }

    pub fn remove(&mut self, handle: Handle<T>) {
        let Some(vec_index) = self.get_vec_index(handle) else {
            return; // Handle is invalid or element already removed
        };
        let last_vec_index = self.vec.len() - 1;
        self.vec.swap(vec_index, last_vec_index);
        self.vec.pop();

        // Update index that was pointing to last element
        // We do not know where it is, therefore let us find it
        for index in &mut self.indices {
            if *index == last_vec_index as u32 {
                *index = vec_index as u32;
            }
        }

        // Index of the removed element can be added to free list
        self.free.push(handle.id);
    }

    /// Appends `other` to `self` leaving `other` empty.
    /// Handles to `other` will be invalid unless they are updated with the returned offset.
    pub fn append(&mut self, other: &mut Pack<T>) -> u32 {
        let ret = self.indices.len() as u32;

        // Update other indices
        let index_offset = self.vec.len() as u32;
        for index in &mut other.indices {
            *index += index_offset;
        }
        for free_index in &mut other.free {
            *free_index += index_offset;
        }

        // Append everything
        self.vec.append(&mut other.vec);
        self.indices.append(&mut other.indices);
        self.free.append(&mut other.free);

        ret
    }
}

impl<T> From<Vec<T>> for Pack<T> {
    fn from(vec: Vec<T>) -> Self {
        let mut ret = Self::new();

        for elem in vec {
            ret.push(elem);
        }

        ret
    }
}

impl<T> FromIterator<T> for Pack<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut ret = Self::new();

        for elem in iter {
            ret.push(elem);
        }

        ret
    }
}

impl<T> IntoIterator for Pack<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter()
    }
}

impl<T> Deref for Pack<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

impl<T> DerefMut for Pack<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Pack<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(PackVisitor::<T>::default())
    }
}

struct PackVisitor<T>(std::marker::PhantomData<T>);

impl<T> Default for PackVisitor<T> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<'de, T> de::Visitor<'de> for PackVisitor<T>
where
    T: Deserialize<'de>,
{
    type Value = Pack<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a sequence of elements")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: de::SeqAccess<'de>,
    {
        let mut pack = Pack::new();

        while let Some(elem) = seq.next_element()? {
            pack.push(elem);
        }

        Ok(pack)
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, thread};

    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    struct Thing {
        val: u32,
    }

    impl Thing {
        fn new(val: u32) -> Self {
            Thing { val }
        }
    }

    #[test]
    fn compare() {
        let a = Handle::<Thing>::new(0);
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn contain() {
        let mut map = HashMap::<Handle<Thing>, Thing>::new();
        let h = Handle::new(0);
        map.insert(h, Thing::new(1));
        assert!(map.contains_key(&h));
    }

    #[test]
    fn simple() {
        let mut pack = Pack::new();
        let thing = pack.push(Thing { val: 2 });
        assert_eq!(thing.get(&pack).unwrap().val, 2);
        assert_eq!(pack.get(thing).unwrap().val, 2);
    }

    #[test]
    fn multiple() {
        let mut pack = Pack::new();
        let mut handles = vec![];

        for i in 0..4 {
            let handle = pack.push(Thing { val: i });
            handles.push(handle);
        }

        for i in 0..4u32 {
            assert_eq!(handles[i as usize].get(&pack).unwrap().val, i);
            assert_eq!(pack.get(handles[i as usize]).unwrap().val, i);
        }
    }

    #[test]
    fn add_remove_add() {
        let mut pack = Pack::new();
        let handle = pack.push(Thing { val: 0 });
        assert_eq!(handle.id, 0);

        pack.remove(handle);
        assert_eq!(pack.len(), 0);

        let handle = pack.push(Thing { val: 1 });
        assert_eq!(handle.id, 0);
        assert_eq!(pack.get(handle).unwrap().val, 1);
    }

    trait Handy {
        fn handy(&self) -> bool;
    }

    impl Handy for Thing {
        fn handy(&self) -> bool {
            self.val == 1
        }
    }

    #[test]
    fn use_traits() {
        let mut pack = Pack::<Box<dyn Handy>>::new();
        let handle = pack.push(Box::new(Thing::new(1)));
        assert_eq!(handle.id, 0);
        assert!(pack.get(handle).unwrap().handy());
    }

    #[test]
    fn send_handle() {
        let handle = Handle::<u32>::NONE;
        thread::spawn(move || {
            assert!(!handle.is_valid());
        });
    }

    #[test]
    fn handle_after_removal_returns_none() {
        let mut pack = Pack::new();
        let handle = pack.push(Thing { val: 42 });
        pack.remove(handle);
        assert!(pack.get(handle).is_none());
    }

    #[test]
    fn handles_are_reused() {
        let mut pack = Pack::new();
        let h1 = pack.push(Thing { val: 1 });
        pack.remove(h1);
        let h2 = pack.push(Thing { val: 2 });
        // h2 should reuse the id of h1
        assert_eq!(h1.id, h2.id);
        assert_eq!(pack.get(h2).unwrap().val, 2);
    }

    #[test]
    fn remove_is_on_cost() {
        // This test is just to document that remove is O(n), not to measure it.
        let mut pack = Pack::new();
        for i in 0..100 {
            pack.push(Thing { val: i });
        }
        // Remove an element and ensure pack is still valid
        let handle = Handle::new(50);
        pack.remove(handle);
        assert_eq!(pack.len(), 99);
    }

    #[test]
    fn serde_pack() {
        let mut pack = Pack::new();
        pack.push(Thing { val: 1 });
        pack.push(Thing { val: 2 });

        let serialized = serde_json::to_string(&pack).unwrap();
        let deserialized: Pack<Thing> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.len(), 2);
        assert_eq!(deserialized.get(Handle::new(0)).unwrap().val, 1);
        assert_eq!(deserialized.get(Handle::new(1)).unwrap().val, 2);
    }

    #[test]
    fn serde_handle() {
        let handle = Handle::<Thing>::new(42);
        let serialized = serde_json::to_string(&handle).unwrap();
        assert_eq!(serialized, "42");
        let deserialized: Handle<Thing> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.id, 42);
        assert!(deserialized.is_valid());
    }
}
