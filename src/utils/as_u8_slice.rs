use core::{ mem::size_of, slice::from_raw_parts_mut };
use super::Pod;

pub trait AsU8Slice {
    fn as_u8_slice    (&    self) -> &    [u8];
    fn as_mut_u8_slice(&mut self) -> &mut [u8];
}

impl< T: Pod > AsU8Slice for T {
    fn as_u8_slice(&self) -> &[u8] {
        let ptr = self as *const T as *mut u8;
        let len = size_of::< T >();
        unsafe { from_raw_parts_mut(ptr, len) }
    }

    fn as_mut_u8_slice(&mut self) -> &mut [u8] {
        let ptr = self as *const T as *mut u8;
        let len = size_of::< T >();
        unsafe { from_raw_parts_mut(ptr, len) }
    }
}

impl< T: Pod > AsU8Slice for [T] {
    fn as_u8_slice(&self) -> &[u8] {
        let ptr = self.as_ptr() as *mut u8;
        let len = size_of::< T >() * self.len();
        unsafe { from_raw_parts_mut(ptr, len) }
    }

    fn as_mut_u8_slice(&mut self) -> &mut [u8] {
        let ptr = self.as_ptr() as *mut u8;
        let len = size_of::< T >() * self.len();
        unsafe { from_raw_parts_mut(ptr, len) }
    }
}
