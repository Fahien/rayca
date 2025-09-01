// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{marker::PhantomData, ops::Range};

#[repr(C)]
pub struct BvhRange<T> {
    pub offset: u32,
    pub count: u32,
    phantom: PhantomData<T>,
}

impl<T> Copy for BvhRange<T> {}
impl<T> Clone for BvhRange<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Default for BvhRange<T> {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

impl<T> BvhRange<T> {
    pub fn new(offset: u32, count: u32) -> Self {
        Self {
            offset,
            count,
            phantom: PhantomData,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn len(&self) -> usize {
        self.count as usize
    }

    pub fn get_start(&self) -> usize {
        self.offset as usize
    }

    pub fn get_end(&self) -> usize {
        (self.offset + self.count) as usize
    }

    pub fn to_range(&self) -> Range<usize> {
        Range {
            start: self.get_start(),
            end: self.get_end(),
        }
    }

    pub fn split_off(&mut self, offset: usize) -> Self {
        let o = offset as u32;
        let right_count = self.count - o;
        self.count = o;
        Self::new(self.offset + o, right_count)
    }
}

impl<T> IntoIterator for BvhRange<T> {
    type Item = usize;
    type IntoIter = Range<usize>;

    fn into_iter(self) -> Self::IntoIter {
        self.offset as usize..(self.offset as usize + self.count as usize)
    }
}

impl<T> IntoIterator for &BvhRange<T> {
    type Item = usize;
    type IntoIter = Range<usize>;

    fn into_iter(self) -> Self::IntoIter {
        self.offset as usize..(self.offset as usize + self.count as usize)
    }
}
