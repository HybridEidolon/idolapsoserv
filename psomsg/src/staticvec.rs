//! A static-length vector wrapper whose size is defined by the generic type over the typenum
//! crate. An added benefit to using this is that Vec isn't copy, where array is, so it keeps us
//! from making lots of unnecessary stack space.

use super::prelude::*;

use std::borrow::{Borrow, BorrowMut};
use std::ops::{Deref, DerefMut};
use std::fmt;

// dummy impl

#[derive(Clone)]
/// A static-length vector, to be used in place of native arrays. Borrow this to get a slice view
/// of the vector's contents.
pub struct StaticVec<T: Clone, L: Unsigned + NonZero> {
    vec: Vec<T>,
    pd: PhantomData<L>
}

impl <T: Clone + fmt::Debug, L: Unsigned + NonZero> fmt::Debug for StaticVec<T, L> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "StaticVec[{}] {{ vec: {:?}, pd: PhantomData }}", L::to_usize(), self.vec)
    }
}

impl <T: Clone, L: Unsigned + NonZero> StaticVec<T, L> {
    /// Create a new static vec with a given default value to clone from.
    pub fn with_value(d: &T) -> Self {
        StaticVec {
            vec: vec![d.clone(); L::to_usize()],
            pd: PhantomData
        }
    }
}

impl <T: Clone + Default, L: Unsigned + NonZero> Default for StaticVec<T, L> {
    fn default() -> Self {
        StaticVec {
            vec: vec![T::default(); L::to_usize()],
            pd: PhantomData
        }
    }
}

impl <T: Clone, L: Unsigned + NonZero> Borrow<[T]> for StaticVec<T, L> {
    fn borrow(&self) -> &[T] {
        self.vec.borrow()
    }
}

impl <T: Clone, L: Unsigned + NonZero> BorrowMut<[T]> for StaticVec<T, L> {
    fn borrow_mut(&mut self) -> &mut [T] {
        self.vec.borrow_mut()
    }
}

impl <T: Clone, L: Unsigned + NonZero> Deref for StaticVec<T, L> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        self.vec.borrow()
    }
}

impl <T: Clone, L: Unsigned + NonZero> DerefMut for StaticVec<T, L> {
    fn deref_mut(&mut self) -> &mut [T] {
        self.vec.borrow_mut()
    }
}

impl <T: Clone + Serial, L: Unsigned + NonZero> Serial for StaticVec<T, L> {
    fn serialize(value: &Self, dst: &mut Write) -> io::Result<()> {
        // The size is statically known; we only want to serialize the contents
        for i in &value.vec {
            try!(T::serialize(i, dst));
        }
        Ok(())
    }

    fn deserialize(src: &mut Read) -> io::Result<Self> {
        let mut ret = StaticVec {
            vec: Vec::with_capacity(L::to_usize()),
            pd: PhantomData
        };

        for _ in 0..L::to_usize() {
            ret.vec.push(try!(T::deserialize(src)))
        }
        Ok(ret)
    }

    #[inline]
    fn serial_len(value: &Self) -> usize {
        T::serial_len(&value[0]) * L::to_usize()
    }

    fn serial_flags(_: &Self) -> u32 { 0 }
}
