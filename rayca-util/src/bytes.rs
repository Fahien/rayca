// Copyright © 2020-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

pub trait AsBytes {
    fn as_bytes(&self) -> &[u8];
}

impl<T> AsBytes for Vec<T> {
    fn as_bytes(&self) -> &[u8] {
        self.as_slice().as_bytes()
    }
}

impl<T> AsBytes for [T] {
    fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts::<u8>(self.as_ptr() as _, std::mem::size_of_val(self)) }
    }
}

pub trait IntoBytes {
    fn into_bytes(self) -> Vec<u8>;
}

impl<T> IntoBytes for Vec<T> {
    fn into_bytes(self) -> Vec<u8> {
        Vec::from(self.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[repr(C)]
    #[derive(Debug, PartialEq)]
    struct Foo {
        a: u16,
        b: u16,
    }

    #[test]
    fn test_as_bytes_slice() {
        let arr: [u16; 2] = [0x1234, 0x5678];
        let bytes = arr.as_bytes();
        assert_eq!(bytes.len(), 4);
    }

    #[test]
    fn test_as_bytes_vec() {
        let v = vec![1u8, 2u8, 3u8];
        let bytes = v.as_bytes();
        assert_eq!(bytes, &[1, 2, 3]);
    }

    #[test]
    fn test_into_bytes_vec() {
        let v = vec![10u32, 20u32];
        let bytes = v.into_bytes();
        assert_eq!(bytes.len(), 8);
    }

    #[test]
    fn test_as_bytes_struct() {
        let foo = Foo {
            a: 0xABCD,
            b: 0x1234,
        };
        let bytes = unsafe {
            std::slice::from_raw_parts(
                (&foo as *const Foo) as *const u8,
                std::mem::size_of::<Foo>(),
            )
        };
        assert_eq!(bytes.len(), 4);
    }
}
