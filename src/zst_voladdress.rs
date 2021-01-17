use super::*;

/// Like a [`VolAddress`], but "stores" the address as a const generic.
///
/// Because of the very limited nature of Rust's current const generics, you
/// can't really do any *dynamic* address calculation with this type. It's only
/// really suitable for volatile addresses that have a const location known at
/// compile time.
///
/// Still, if that does fit your use case, this is kinda neat to have available.
#[derive(Hash)]
pub struct ZstVolAddress<T, R, W, const A: usize> {
  target: PhantomData<T>,
  read_status: PhantomData<R>,
  write_status: PhantomData<W>,
}
impl<T, R, W, const A: usize> ZstVolAddress<T, R, W, A> {
  /// Constructs the value.
  ///
  /// ## Safety
  /// * This has the same safety rules as [VolAddress::new](VolAddress::new),
  ///   but instead of needing to pass in a legal `usize` value, you must have a
  ///   legal `A` type parameter.
  #[inline]
  #[must_use]
  pub const unsafe fn new() -> Self {
    Self {
      target: PhantomData,
      read_status: PhantomData,
      write_status: PhantomData,
    }
  }

  /// Changes this `ZstVolAddress` into a plain old [`VolAddress`].
  ///
  /// Naming is hard.
  #[inline]
  #[must_use]
  pub const fn to_plain() -> VolAddress<T, R, W> {
    unsafe { VolAddress::new(A) }
  }
}

impl<T, W, const A: usize> ZstVolAddress<T, Yes, W, A>
where
  T: Copy,
{
  /// Volatile reads the current value of `A`.
  #[inline]
  pub fn read(self) -> T {
    unsafe { read_volatile(A as *const T) }
  }
}
impl<T, W, const A: usize> ZstVolAddress<T, Unsafe, W, A>
where
  T: Copy,
{
  /// Volatile reads the current value of `A`.
  #[inline]
  pub unsafe fn read(self) -> T {
    read_volatile(A as *const T)
  }
}

impl<T, R, const A: usize> ZstVolAddress<T, R, Yes, A>
where
  T: Copy,
{
  /// Volatile writes a new value to `A`.
  #[inline]
  pub fn write(self, t: T) {
    unsafe { write_volatile(A as *mut T, t) }
  }
}
impl<T, R, const A: usize> ZstVolAddress<T, R, Unsafe, A>
where
  T: Copy,
{
  /// Volatile writes a new value to `A`.
  #[inline]
  pub unsafe fn write(self, t: T) {
    write_volatile(A as *mut T, t)
  }
}

impl<T, R, W, const A: usize> Clone for ZstVolAddress<T, R, W, A> {
  #[inline]
  #[must_use]
  fn clone(&self) -> Self {
    *self
  }
}
impl<T, R, W, const A: usize> Copy for ZstVolAddress<T, R, W, A> {}

impl<T, R, W, const LEFT: usize, const RIGHT: usize>
  core::cmp::PartialEq<ZstVolAddress<T, R, W, RIGHT>>
  for ZstVolAddress<T, R, W, LEFT>
{
  fn eq(&self, _: &ZstVolAddress<T, R, W, RIGHT>) -> bool {
    core::cmp::PartialEq::eq(&LEFT, &RIGHT)
  }
}
impl<T, R, W, const A: usize> core::cmp::Eq for ZstVolAddress<T, R, W, A> {}

impl<T, R, W, const LEFT: usize, const RIGHT: usize>
  core::cmp::PartialOrd<ZstVolAddress<T, R, W, RIGHT>>
  for ZstVolAddress<T, R, W, LEFT>
{
  fn partial_cmp(
    &self, _: &ZstVolAddress<T, R, W, RIGHT>,
  ) -> Option<core::cmp::Ordering> {
    core::cmp::PartialOrd::partial_cmp(&LEFT, &RIGHT)
  }
}
impl<T, R, W, const A: usize> core::cmp::Ord for ZstVolAddress<T, R, W, A> {
  fn cmp(&self, _: &Self) -> core::cmp::Ordering {
    core::cmp::Ordering::Equal
  }
}

impl<T, R, W, const A: usize> core::fmt::Debug for ZstVolAddress<T, R, W, A> {
  #[cold]
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(
      f,
      "ZstVolAddress<{elem_ty}, r{readability}, w{writeability}, @{address:#X}>",
      elem_ty = core::any::type_name::<T>(),
      readability = core::any::type_name::<R>(),
      writeability = core::any::type_name::<W>(),
      address = A
    )
  }
}
