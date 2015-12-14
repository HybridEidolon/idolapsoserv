//! Structures for PSO.

extern crate byteorder;

pub mod battleparam;

pub use battleparam::BattleParam;

use std::io;
use std::io::{Read, Write};

pub trait Parse: Sized {
    fn read(src: &mut Read) -> io::Result<Self>;
    fn write(&self, dst: &mut Write) -> io::Result<()>;
}
