use super::*;

/// A volatile memory "series".
///
/// This is intended to model when a portion of memory is a series of evenly
/// spaced values that are *not* directly contiguous.
///
/// ## Generic Parameters
/// * `T` / `R` / `W`: These parameters are applied to the [`VolAddress`] type
///   returned when accessing the series in any way (indexing, iteration, etc).
/// * `C`: the count of elements in the series.
/// * `S`: the stride **in bytes** between series elements.
///
/// ## Safety
/// * This type stores a [`VolAddress`] internally, and so you must follow all
///   of those safety rules. Notably, the base address must never be zero.
/// * The address space must legally contain `C` values of the `T` type, spaced
///   every `S` bytes, starting from the base address.
/// * The memory series must not wrap around the end of the address space.
#[derive(Hash)]
#[repr(transparent)]
pub struct VolSeries<T, R, W, const C: usize, const S: usize> {
  base: VolAddress<T, R, W>,
  target: PhantomData<T>,
  read_status: PhantomData<R>,
  write_status: PhantomData<W>,
}

impl<T, R, W, const C: usize, const S: usize> VolSeries<T, R, W, C, S> {
  /// Constructs the value.
  ///
  /// ## Safety
  /// * As per the type docs.
  #[inline]
  #[must_use]
  pub const unsafe fn new(base: usize) -> Self {
    Self {
      base: VolAddress::new(base),
      target: PhantomData,
      read_status: PhantomData,
      write_status: PhantomData,
    }
  }

  /// Indexes to the `i`th position of the memory series.
  ///
  /// ## Panics
  /// * If the index is out of bounds this will panic.
  #[inline]
  #[must_use]
  pub const fn index(self, i: usize) -> VolAddress<T, R, W> {
    if i < C {
      unsafe { self.base.cast::<[u8; S]>().add(i).cast::<T>() }
    } else {
      // Note(Lokathor): We force a const panic by indexing out of bounds.
      #[allow(unconditional_panic)]
      unsafe {
        VolAddress::new([usize::MAX][1])
      }
    }
  }

  /// Gets the address of the `i`th position, if it's in bounds.
  #[inline]
  #[must_use]
  pub const fn get(self, i: usize) -> Option<VolAddress<T, R, W>> {
    if i < C {
      Some(unsafe { self.base.cast::<[u8; S]>().add(i).cast::<T>() })
    } else {
      None
    }
  }

  /// Creates an iterator over the addresses of the memory series.
  #[inline]
  #[must_use]
  pub const fn iter(self) -> VolSeriesIter<T, R, W, S> {
    VolSeriesIter { base: self.base, count: C }
  }
}

impl<T, R, W, const C: usize, const S: usize> Clone
  for VolSeries<T, R, W, C, S>
{
  #[inline]
  #[must_use]
  fn clone(&self) -> Self {
    *self
  }
}
impl<T, R, W, const C: usize, const S: usize> Copy
  for VolSeries<T, R, W, C, S>
{
}

impl<T, R, W, const C: usize, const S: usize> core::cmp::PartialEq
  for VolSeries<T, R, W, C, S>
{
  fn eq(&self, other: &Self) -> bool {
    core::cmp::PartialEq::eq(&self.base, &other.base)
  }
}
impl<T, R, W, const C: usize, const S: usize> core::cmp::Eq
  for VolSeries<T, R, W, C, S>
{
}

impl<T, R, W, const C: usize, const S: usize> core::fmt::Debug
  for VolSeries<T, R, W, C, S>
{
  #[cold]
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(f, "VolSeries<{elem_ty}, r{readability}, w{writeability}, c{count}, s{stride:#X}>({address:#X})",
      elem_ty = core::any::type_name::<T>(),
      readability=core::any::type_name::<R>(),
      writeability=core::any::type_name::<W>(),
      count=C,
      stride=S,
      address=self.base.address.get())
  }
}

/// An iterator over a volatile series.
///
/// You will generally not construct types of this value yourself. Instead, you
/// obtain them via the [`VolSeries::iter`](VolSeries::iter) method.
#[repr(C)]
pub struct VolSeriesIter<T, R, W, const S: usize> {
  pub(crate) base: VolAddress<T, R, W>,
  pub(crate) count: usize,
}

impl<T, R, W, const S: usize> Clone for VolSeriesIter<T, R, W, S> {
  #[inline]
  #[must_use]
  fn clone(&self) -> Self {
    Self { base: self.base, count: self.count }
  }
}

impl<T, R, W, const S: usize> core::iter::Iterator
  for VolSeriesIter<T, R, W, S>
{
  type Item = VolAddress<T, R, W>;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    if self.count > 0 {
      let out = Some(self.base);
      self.count -= 1;
      self.base = unsafe { self.base.cast::<[u8; S]>().add(1).cast::<T>() };
      out
    } else {
      None
    }
  }

  #[inline]
  #[must_use]
  fn size_hint(&self) -> (usize, Option<usize>) {
    (self.count, Some(self.count))
  }

  #[inline]
  #[must_use]
  fn count(self) -> usize {
    self.count
  }

  #[inline]
  #[must_use]
  fn last(self) -> Option<Self::Item> {
    if self.count > 0 {
      Some(unsafe {
        self.base.cast::<[u8; S]>().add(self.count - 1).cast::<T>()
      })
    } else {
      None
    }
  }

  #[inline]
  fn nth(&mut self, n: usize) -> Option<Self::Item> {
    if n < self.count {
      self.count -= n;
      self.base = unsafe { self.base.cast::<[u8; S]>().add(1 + n).cast::<T>() };
      Some(self.base)
    } else {
      self.count = 0;
      None
    }
  }
}

impl<T, R, W, const S: usize> core::iter::DoubleEndedIterator
  for VolSeriesIter<T, R, W, S>
{
  #[inline]
  fn next_back(&mut self) -> Option<Self::Item> {
    if self.count > 0 {
      let out = Some(unsafe {
        self.base.cast::<[u8; S]>().add(self.count - 1).cast::<T>()
      });
      self.count -= 1;
      out
    } else {
      None
    }
  }

  #[inline]
  fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
    if n < self.count {
      self.count -= n;
      Some(unsafe { self.base.cast::<[u8; S]>().add(1 + n).cast::<T>() })
    } else {
      self.count = 0;
      None
    }
  }
}

#[test]
#[allow(bad_style)]
fn test_impl_Iterator_for_VolSeriesIter() {
  let i: VolSeriesIter<u16, (), (), 0x100> = VolSeriesIter {
    base: unsafe { VolAddress::new(core::mem::align_of::<u16>()) },
    count: 4,
  };

  let mut i_c = i.clone().map(|a| a.as_usize());
  assert_eq!(i_c.next(), Some(0x002));
  assert_eq!(i_c.next(), Some(0x102));
  assert_eq!(i_c.next(), Some(0x202));
  assert_eq!(i_c.next(), Some(0x302));
  assert_eq!(i_c.next(), None);
  assert_eq!(i_c.next(), None);

  let i_c = i.clone();
  assert_eq!(i_c.size_hint(), (4, Some(4)));

  let i_c = i.clone();
  assert_eq!(i_c.count(), 4);

  let i_c = i.clone().map(|a| a.as_usize());
  assert_eq!(i_c.last(), Some(0x302));

  let mut i_c = i.clone().map(|a| a.as_usize());
  assert_eq!(i_c.nth(0), Some(0x002));
  assert_eq!(i_c.nth(0), Some(0x102));
  assert_eq!(i_c.nth(0), Some(0x202));
  assert_eq!(i_c.nth(0), Some(0x302));
  assert_eq!(i_c.nth(0), None);
  assert_eq!(i_c.nth(0), None);

  let mut i_c = i.clone().map(|a| a.as_usize());
  assert_eq!(i_c.nth(1), Some(0x102));
  assert_eq!(i_c.nth(1), Some(0x302));
  assert_eq!(i_c.nth(1), None);
  assert_eq!(i_c.nth(1), None);

  let mut i_c = i.clone().map(|a| a.as_usize());
  assert_eq!(i_c.nth(2), Some(0x202));
  assert_eq!(i_c.nth(2), None);
  assert_eq!(i_c.nth(2), None);

  let mut i_c = i.clone().map(|a| a.as_usize());
  assert_eq!(i_c.nth(3), Some(0x302));
  assert_eq!(i_c.nth(3), None);
  assert_eq!(i_c.nth(3), None);

  let mut i_c = i.clone().map(|a| a.as_usize());
  assert_eq!(i_c.nth(4), None);
  assert_eq!(i_c.nth(4), None);
}

#[test]
#[allow(bad_style)]
fn test_impl_DoubleEndedIterator_for_VolSeriesIter() {
  let i: VolSeriesIter<u16, (), (), 0x100> = VolSeriesIter {
    base: unsafe { VolAddress::new(core::mem::align_of::<u16>()) },
    count: 4,
  };

  let mut i_c = i.clone().map(|a| a.as_usize());
  assert_eq!(i_c.next_back(), Some(0x302));
  assert_eq!(i_c.next_back(), Some(0x202));
  assert_eq!(i_c.next_back(), Some(0x102));
  assert_eq!(i_c.next_back(), Some(0x002));
  assert_eq!(i_c.next_back(), None);
  assert_eq!(i_c.next_back(), None);

  let mut i_c = i.clone().map(|a| a.as_usize());
  assert_eq!(i_c.nth_back(0), Some(0x302));
  assert_eq!(i_c.nth_back(0), Some(0x202));
  assert_eq!(i_c.nth_back(0), Some(0x102));
  assert_eq!(i_c.nth_back(0), Some(0x002));
  assert_eq!(i_c.nth_back(0), None);
  assert_eq!(i_c.nth_back(0), None);

  let mut i_c = i.clone().map(|a| a.as_usize());
  assert_eq!(i_c.nth_back(1), Some(0x202));
  assert_eq!(i_c.nth_back(1), Some(0x002));
  assert_eq!(i_c.nth_back(1), None);
  assert_eq!(i_c.nth_back(1), None);

  let mut i_c = i.clone().map(|a| a.as_usize());
  assert_eq!(i_c.nth_back(2), Some(0x102));
  assert_eq!(i_c.nth_back(2), None);
  assert_eq!(i_c.nth_back(2), None);

  let mut i_c = i.clone().map(|a| a.as_usize());
  assert_eq!(i_c.nth_back(3), Some(0x002));
  assert_eq!(i_c.nth_back(3), None);
  assert_eq!(i_c.nth_back(3), None);

  let mut i_c = i.clone().map(|a| a.as_usize());
  assert_eq!(i_c.nth_back(4), None);
  assert_eq!(i_c.nth_back(4), None);
}
