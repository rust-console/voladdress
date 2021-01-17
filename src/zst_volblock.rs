use super::*;

/// Like a [`VolBlock`], but "stores" the address as a const generic.
///
/// Because of the very limited nature of Rust's current const generics, you
/// can't really do any *dynamic* address calculation with this type. It's only
/// really suitable for volatile blocks that have a const location known at
/// compile time.
///
/// Still, if that does fit your use case, this is kinda neat to have available.
#[derive(Hash)]
pub struct ZstVolBlock<T, R, W, const C: usize, const A: usize> {
  base: ZstVolAddress<T, R, W, A>,
  target: PhantomData<T>,
  read_status: PhantomData<R>,
  write_status: PhantomData<W>,
}

impl<T, R, W, const C: usize, const A: usize> ZstVolBlock<T, R, W, C, A> {
  /// Constructs the value.
  ///
  /// ## Safety
  /// * As per the type docs.
  #[inline]
  #[must_use]
  pub const unsafe fn new() -> Self {
    Self {
      base: ZstVolAddress::new(),
      target: PhantomData,
      read_status: PhantomData,
      write_status: PhantomData,
    }
  }

  /// Indexes to the `i`th position of the memory block.
  ///
  /// ## Panics
  /// * If the index is out of bounds this will panic.
  #[inline]
  #[must_use]
  pub const fn index(self, i: usize) -> VolAddress<T, R, W> {
    if i < C {
      unsafe { VolAddress::new(A).add(i) }
    } else {
      // Note(Lokathor): We force a const panic by indexing out of bounds.
      #[allow(unconditional_panic)]
      unsafe {
        VolAddress::new([usize::MAX][1])
      }
    }
  }

  /// Gets the address of the `i`th position if it's in bounds.
  #[inline]
  #[must_use]
  pub const fn get(self, i: usize) -> Option<VolAddress<T, R, W>> {
    if i < C {
      Some(unsafe { VolAddress::new(A).add(i) })
    } else {
      None
    }
  }

  /// Creates an iterator over the addresses of the memory block.
  #[inline]
  #[must_use]
  pub const fn iter(self) -> VolBlockIter<T, R, W> {
    VolBlockIter { base: unsafe { VolAddress::new(A) }, count: C }
  }
}

impl<T, R, W, const C: usize, const A: usize> Clone
  for ZstVolBlock<T, R, W, C, A>
{
  #[inline]
  #[must_use]
  fn clone(&self) -> Self {
    *self
  }
}
impl<T, R, W, const C: usize, const A: usize> Copy
  for ZstVolBlock<T, R, W, C, A>
{
}

impl<T, R, W, const C: usize, const LEFT: usize, const RIGHT: usize>
  core::cmp::PartialEq<ZstVolBlock<T, R, W, C, RIGHT>>
  for ZstVolBlock<T, R, W, C, LEFT>
{
  fn eq(&self, _: &ZstVolBlock<T, R, W, C, RIGHT>) -> bool {
    core::cmp::PartialEq::eq(&LEFT, &RIGHT)
  }
}
impl<T, R, W, const C: usize, const A: usize> core::cmp::Eq
  for ZstVolBlock<T, R, W, C, A>
{
}

impl<T, R, W, const C: usize, const A: usize> core::fmt::Debug
  for ZstVolBlock<T, R, W, C, A>
{
  #[cold]
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(f, "VolBlock<{elem_ty}, r{readability}, w{writeability}, c{count}, @{address:#X}>",
      elem_ty = core::any::type_name::<T>(),
      readability=core::any::type_name::<R>(),
      writeability=core::any::type_name::<W>(),
      count=C,
      address=A)
  }
}
