use super::*;

/// A dynamically sized span of volatile memory.
///
/// If you think of [VolBlock] as being similar to an array, this type is more
/// similar to a slice.
///
/// The primary utility of this type is just that it bundles a pointer and
/// length together, which allows you to have safe dynamic bounds checking. Just
/// like with `VolBlock`, It does **not** have a lifetime or participate in
/// borrow checking, and it does **not** enforce exclusive access.
///
/// A `VolRegion` assumes that elements of the region are directly one after the
/// other (again, like how `VolBlock` works). If you need dynamic bounds
/// checking on a spaced out series of values that would be some other type,
/// which doesn't currently exist in the library. (Open a PR maybe?)
///
/// ## Generic Parameters
/// * `T` / `R` / `W`: These parameters are applied to the [`VolAddress`] type
///   returned when accessing the region in any way (indexing, iteration, etc).
///
/// ## Safety
/// * This type stores a base [`VolAddress`] internally, and so you must follow
///   all of those safety rules. Notably, the base address must never be zero.
/// * The region must legally contain `len` contiguous values of the `T` type,
///   starting from the base address.
/// * The region must not wrap around past the end of the address space.
#[repr(C)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VolRegion<T, R, W> {
  pub(crate) addr: VolAddress<T, R, W>,
  pub(crate) len: usize,
}
impl<T, R, W> Clone for VolRegion<T, R, W> {
  #[inline]
  #[must_use]
  fn clone(&self) -> Self {
    *self
  }
}
impl<T, R, W> Copy for VolRegion<T, R, W> {}
impl<T, R, W> core::fmt::Debug for VolRegion<T, R, W> {
  #[cold]
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(f, "VolRegion<{elem_ty}, r{readability}, w{writeability}>({address:#X}, len: {len})",
      elem_ty = core::any::type_name::<T>(),
      readability=core::any::type_name::<R>(),
      writeability=core::any::type_name::<W>(),
      address=self.addr.as_usize(),
      len=self.len,
    )
  }
}
impl<T, R, W, const C: usize> From<VolBlock<T, R, W, C>>
  for VolRegion<T, R, W>
{
  #[inline]
  #[must_use]
  fn from(block: VolBlock<T, R, W, C>) -> Self {
    Self { addr: block.base, len: C }
  }
}

impl<T, R, W> VolRegion<T, R, W> {
  /// Constructs a region from raw parts.
  ///
  /// ## Safety
  /// * As per the type docs.
  #[inline]
  #[must_use]
  pub const unsafe fn from_raw_parts(
    addr: VolAddress<T, R, W>, len: usize,
  ) -> Self {
    Self { addr, len }
  }

  /// Gets the length (in elements) of the region.
  #[inline]
  #[must_use]
  pub const fn len(self) -> usize {
    self.len
  }

  /// Converts the `VolBlock` the `usize` for the start of the block.
  #[inline]
  #[must_use]
  pub const fn as_usize(self) -> usize {
    self.addr.address.get()
  }

  /// Converts the `VolBlock` into an individual const pointer.
  ///
  /// This should usually only be used when you need to call a foreign function
  /// that expects a pointer.
  #[inline]
  #[must_use]
  pub const fn as_ptr(self) -> *const T {
    self.addr.address.get() as *const T
  }

  /// Converts the `VolBlock` into an individual mut pointer.
  ///
  /// This should usually only be used when you need to call a foreign function
  /// that expects a pointer.
  #[inline]
  #[must_use]
  pub const fn as_mut_ptr(self) -> *mut T {
    self.addr.address.get() as *mut T
  }

  /// Converts the `VolBlock` into a const slice pointer.
  ///
  /// This should usually only be used when you need to call a foreign function
  /// that expects a pointer.
  #[inline]
  #[must_use]
  // TODO(2022-10-15): const fn this at some point in the future (1.64 minimum)
  pub fn as_slice_ptr(self) -> *const [T] {
    core::ptr::slice_from_raw_parts(
      self.addr.address.get() as *const T,
      self.len,
    )
  }

  /// Converts the `VolBlock` into an individual mut pointer.
  ///
  /// This should usually only be used when you need to call a foreign function
  /// that expects a pointer.
  #[inline]
  #[must_use]
  // TODO(2022-10-15): const fn this at some point in the future (unstable)
  pub fn as_slice_mut_ptr(self) -> *mut [T] {
    core::ptr::slice_from_raw_parts_mut(
      self.addr.address.get() as *mut T,
      self.len,
    )
  }

  /// Index into the region.
  ///
  /// ## Panics
  /// * If the index requested is out of bounds this will panic.
  #[inline]
  #[must_use]
  #[track_caller]
  pub const fn index(self, i: usize) -> VolAddress<T, R, W> {
    if i < self.len {
      unsafe { self.addr.add(i) }
    } else {
      // Note(Lokathor): We force a const panic by indexing out of bounds.
      #[allow(unconditional_panic)]
      unsafe {
        VolAddress::new([usize::MAX][1])
      }
    }
  }

  /// Gets `Some(addr)` if in bounds, or `None` if out of bounds.
  #[inline]
  #[must_use]
  pub const fn get(self, i: usize) -> Option<VolAddress<T, R, W>> {
    if i < self.len {
      Some(unsafe { self.addr.add(i) })
    } else {
      None
    }
  }

  /// Gets a sub-slice of this region as a new region.
  ///
  /// ## Panics
  /// * If either specified end of the range is out of bounds this will panic.
  #[inline]
  #[must_use]
  #[track_caller]
  pub fn sub_slice<RB: core::ops::RangeBounds<usize>>(self, r: RB) -> Self {
    // TODO: some day make this a const fn, once start_bound and end_bound are
    // made into const fn, but that requires const trait impls.
    use core::ops::Bound;
    let start_inclusive: usize = match r.start_bound() {
      Bound::Included(i) => *i,
      Bound::Excluded(x) => x + 1,
      Bound::Unbounded => 0,
    };
    assert!(start_inclusive < self.len);
    let end_exclusive: usize = match r.end_bound() {
      Bound::Included(i) => i + 1,
      Bound::Excluded(x) => *x,
      Bound::Unbounded => self.len,
    };
    assert!(end_exclusive <= self.len);
    let len = end_exclusive.saturating_sub(start_inclusive);
    Self { addr: unsafe { self.addr.add(start_inclusive) }, len }
  }

  /// Gives an iterator over this region.
  #[inline]
  #[must_use]
  pub const fn iter(self) -> VolBlockIter<T, R, W> {
    VolBlockIter { base: self.addr, count: self.len }
  }

  /// Same as `region.sub_slice(range).iter()`
  #[inline]
  #[must_use]
  #[track_caller]
  pub fn iter_range<RB: core::ops::RangeBounds<usize>>(
    self, r: RB,
  ) -> VolBlockIter<T, R, W> {
    self.sub_slice(r).iter()
  }
}

impl<T, W> VolRegion<T, Safe, W>
where
  T: Copy,
{
  /// Volatile reads each element into the provided buffer.
  ///
  /// ## Panics
  /// * If the buffer's length is not *exactly* this region's length.
  #[inline]
  pub fn read_to_slice(self, buffer: &mut [T]) {
    assert_eq!(self.len, buffer.len());
    self.iter().zip(buffer.iter_mut()).for_each(|(va, s)| *s = va.read())
  }
}
impl<T, W> VolRegion<T, Unsafe, W>
where
  T: Copy,
{
  /// Volatile reads each element into the provided buffer.
  ///
  /// ## Panics
  /// * If the buffer's length is not *exactly* this region's length.
  ///
  /// ## Safety
  /// * The safety rules of reading this address depend on the device. Consult
  ///   your hardware manual.
  #[inline]
  pub unsafe fn read_to_slice(self, buffer: &mut [T]) {
    assert_eq!(self.len, buffer.len());
    self.iter().zip(buffer.iter_mut()).for_each(|(va, s)| *s = va.read())
  }
}

impl<T, R> VolRegion<T, R, Safe>
where
  T: Copy,
{
  /// Volatile all slice elements into this region.
  ///
  /// ## Panics
  /// * If the buffer's length is not *exactly* this region's length.
  #[inline]
  pub fn write_from_slice(self, buffer: &[T]) {
    assert_eq!(self.len, buffer.len());
    self.iter().zip(buffer.iter()).for_each(|(va, s)| va.write(*s))
  }
}
impl<T, R> VolRegion<T, R, Unsafe>
where
  T: Copy,
{
  /// Volatile all slice elements into this region.
  ///
  /// ## Panics
  /// * If the buffer's length is not *exactly* this region's length.
  ///
  /// ## Safety
  /// * The safety rules of writing this address depend on the device. Consult
  ///   your hardware manual.
  #[inline]
  pub unsafe fn write_from_slice(self, buffer: &[T]) {
    assert_eq!(self.len, buffer.len());
    self.iter().zip(buffer.iter()).for_each(|(va, s)| va.write(*s))
  }
}

#[test]
fn test_volregion_sub_slice() {
  let region: VolRegion<u8, Unsafe, Unsafe> =
    unsafe { VolRegion::from_raw_parts(VolAddress::new(1), 10) };
  assert_eq!(region.len, 10);

  let sub_region = region.sub_slice(..);
  assert_eq!(sub_region.len, 10);

  let sub_region = region.sub_slice(2..);
  assert_eq!(sub_region.len, 10 - 2);

  let sub_region = region.sub_slice(..3);
  assert_eq!(sub_region.len, 3);

  let sub_region = region.sub_slice(4..6);
  assert_eq!(sub_region.len, 2);
}
