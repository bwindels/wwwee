mod buffer;
pub mod pool;

use std::cell::RefMut;
pub type Buffer<'a> = self::buffer::Buffer<RefMut<'a, [u8]>>;
