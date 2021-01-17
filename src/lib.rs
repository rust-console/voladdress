#![no_std]
#![deny(missing_docs)]

//! A crate for working with volatile addresses / memory mapped IO / MMIO.

// TODO: crate docs that explain how to model weird stuff.

use core::{
  marker::PhantomData,
  num::NonZeroUsize,
  ptr::{read_volatile, write_volatile},
};

mod plain_voladdress;
pub use plain_voladdress::*;

mod volblock;
pub use volblock::*;

mod volseries;
pub use volseries::*;

/// Lets you put "No" into a generic type parameter.
pub struct No;

/// Lets you put "Yes" into a generic type parameter.
pub struct Yes;

/// Lets you put "Unsafe" into a generic type parameter.
pub struct Unsafe;
