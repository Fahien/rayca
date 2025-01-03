// Copyright © 2020-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{
    hash::{Hash, Hasher},
    iter::FromIterator,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use instant::{Duration, Instant};

/// Useful timer to get delta time, and previous time
pub struct Timer {
    prev: Instant,
    curr: Instant,
}

impl Timer {
    pub fn new() -> Self {
        let prev = Instant::now();
        let curr = Instant::now();
        Self { prev, curr }
    }

    /// Returns delta time in seconds
    pub fn get_delta(&mut self) -> Duration {
        self.curr = Instant::now();
        let delta = self.curr - self.prev;
        self.prev = self.curr;
        delta
    }

    /// Returns the time of last `get_delta()`
    pub fn _get_prev(&self) -> Instant {
        self.prev
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

/// A handle is a sort of index into a vector of elements of a specific kind.
/// It is useful when we do not want to keep a reference to an element,
/// while taking advantage of strong typing to avoid using integers.
#[derive(Debug)]
pub struct Handle<T> {
    pub id: usize,
    /// https://stackoverflow.com/a/50201389
    phantom: PhantomData<fn() -> T>,
}

impl<T> Default for Handle<T> {
    fn default() -> Self {
        Self::none()
    }
}

impl<T> From<u32> for Handle<T> {
    fn from(id: u32) -> Self {
        Self::new(id as usize)
    }
}

impl<T> Handle<T> {
    pub const NONE: Self = Self {
        id: std::usize::MAX,
        phantom: PhantomData,
    };

    pub fn new(id: usize) -> Self {
        Self {
            id,
            phantom: PhantomData,
        }
    }

    pub fn none() -> Self {
        Self {
            id: std::usize::MAX,
            phantom: PhantomData,
        }
    }

    pub fn valid(&self) -> bool {
        self.id != std::usize::MAX
    }

    pub fn offset(&mut self, offset: usize) {
        if self.valid() {
            self.id += offset;
        }
    }

    pub fn is_none(&self) -> bool {
        !self.valid()
    }
}

impl<'a, T> Handle<T> {
    pub fn get(&self, pack: &'a Pack<T>) -> Option<&'a T> {
        pack.vec.get(self.id)
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

impl<T> Hash for Handle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// A `Pack` is a powerful structure which contains a vector of contiguous elements
/// and a list of indices to those elements. `Handle`s are used to work with `Pack`s.
#[derive(Default)]
pub struct Pack<T> {
    /// List of contiguous elements
    vec: Vec<T>,
    /// List of indices to elements
    indices: Vec<usize>,
    /// List of positions to free indices
    free: Vec<usize>,
}

impl<T> Pack<T> {
    pub fn new() -> Self {
        Self {
            vec: vec![],
            indices: vec![],
            free: vec![],
        }
    }

    pub fn push(&mut self, elem: T) -> Handle<T> {
        let index = self.vec.len();
        self.vec.push(elem);

        if !self.free.is_empty() {
            let id = self.free.pop().unwrap();
            self.indices[id] = index;
            Handle::new(id)
        } else {
            let id = self.indices.len();
            self.indices.push(index);
            Handle::new(id)
        }
    }

    fn get_vec_index(&self, handle: Handle<T>) -> usize {
        assert!(handle.id < self.indices.len());
        let vec_index = self.indices[handle.id];
        assert!(vec_index < self.vec.len());
        vec_index
    }

    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        if !handle.valid() {
            return None;
        }
        self.vec.get(self.get_vec_index(handle))
    }

    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        if !handle.valid() {
            return None;
        }
        let vec_index = self.get_vec_index(handle);
        self.vec.get_mut(vec_index)
    }

    pub fn remove(&mut self, handle: Handle<T>) {
        let vec_index = self.get_vec_index(handle);
        let last_vec_index = self.vec.len() - 1;
        self.vec.swap(vec_index, last_vec_index);
        self.vec.pop();

        // Update index that was pointing to last element
        // We do not know where it is, therefore let us find it
        for index in &mut self.indices {
            if *index == last_vec_index {
                *index = vec_index;
            }
        }

        // Index of the removed element can be added to free list
        self.free.push(handle.id);
    }

    /// Appends `other` to the current one, returning the offset for updating handles to `other`
    pub fn append(&mut self, other: &mut Pack<T>) -> usize {
        let ret = self.indices.len();

        // Update other indices
        let index_offset = self.vec.len();
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

#[cfg(test)]
mod test {
    use std::{collections::HashMap, thread};

    use super::*;

    #[derive(Debug)]
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
        let handle = Handle::<u32>::none();
        thread::spawn(move || {
            assert!(!handle.valid());
        });
    }
}

#[macro_export]
macro_rules! print_info {
    ( $s:expr, $( $t:tt )* ) => {
        println!("{:>12} {}", owo_colors::OwoColorize::bold(&owo_colors::OwoColorize::blue(&$s)), format!($( $t )*))
    }
}

#[macro_export]
macro_rules! print_success {
    ( $s:expr, $( $t:tt )* ) => {
        println!("{:>12} {}", owo_colors::OwoColorize::bold(&owo_colors::OwoColorize::green(&$s)), format!($( $t )*))
    }
}

#[macro_export]
macro_rules! print_warning {
    ( $s:expr, $( $t:tt )* ) => {
        println!("{:>12} {}", owo_colors::OwoColorize::bold(&owo_colors::OwoColorize::yellow(&$s)), format!($( $t )*))
    }
}

#[macro_export]
macro_rules! fail {
    ( $( $t:tt )* ) => {
        format!("{:>12} {}", owo_colors::OwoColorize::bold(&owo_colors::OwoColorize::red(&"Failed")), format!($( $t )*))
    }
}

#[macro_export]
macro_rules! warn {
    ( $s:expr, $( $t:tt )* ) => {
        format!("{:>12} {}", owo_colors::OwoColorize::bold(&owo_colors::OwoColorize::yellow(&$s)), format!($( $t )*))
    }
}

#[macro_export]
macro_rules! print_warn {
    ( $s:expr, $( $t:tt )* ) => {
        $crate::rlog!("{:>12} {}",
            owo_colors::OwoColorize::bold(
                &owo_colors::OwoColorize::yellow(&$s)), format!($( $t )*))
    }
}

#[macro_export]
macro_rules! panic_fail {
    ( $( $t:tt )* ) => {
        panic!("{:>12} {}", owo_colors::OwoColorize::bold(&owo_colors::OwoColorize::red(&"Failed")), format!($( $t )*))
    }
}
