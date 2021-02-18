use super::*;

/// A volatile address.
///
/// This type stores a memory address and provides ergonomic volatile access to
/// said memory address.
///
/// ## Generic Parameters
///
/// * `T`: The type of the value stored at the address.
///   * The target type type must impl `Copy` for reading and writing to be
///     allowed.
/// * `R`: If the address is readable.
///   * If `R=Yes` then you can safely read from the address.
///   * If `R=Unsafe` then you can unsafely read from the address.
///   * Otherwise you cannot read from the address.
/// * `W`: If the address is writable.
///   * If `W=Yes` then you can safely write to the address.
///   * If `W=Unsafe` then you can unsafely write to the address.
///   * Otherwise you cannot write to the address.
///
/// The `VolAddress` type is intended to represent a single value of a `T` type
/// that is the size of a single machine register (or less).
/// * If there's an array of contiguous `T` values you want to model, consider
///   using [`VolBlock`] instead.
/// * If there's a series of strided `T` values you want to model, consider
///   using [`VolSeries`] instead.
/// * If the `T` type is larger than a single machine register it's probably
///   **not** a good fit for the `VolAddress` abstraction. See the notes at the
///   crate root for an alternative.
///
/// ## Safety
/// This type's safety follows the "unsafe creation, then safe use" strategy.
///
/// * Validity Invariant: The address of a `VolAddress` must always be non-zero,
///   or you will instantly trigger UB.
/// * Safety Invariant: The address of a `VolAddress` must be an aligned and
///   legal address for a `T` type value within the device's memory space,
///   otherwise the `read` and `write` methods will trigger UB when called.
#[repr(transparent)]
pub struct VolAddress<T, R, W> {
  pub(crate) address: NonZeroUsize,
  target: PhantomData<T>,
  read_status: PhantomData<R>,
  write_status: PhantomData<W>,
}

impl<T, R, W> VolAddress<T, R, W> {
  /// Constructs the value.
  ///
  /// ## Safety
  /// * As per the type docs.
  #[inline]
  #[must_use]
  pub const unsafe fn new(address: usize) -> Self {
    Self {
      address: NonZeroUsize::new_unchecked(address),
      target: PhantomData,
      read_status: PhantomData,
      write_status: PhantomData,
    }
  }

  /// Changes the target type from `T` to `Z`.
  ///
  /// ## Safety
  /// * As per the type docs
  #[inline]
  #[must_use]
  pub const unsafe fn cast<Z>(self) -> VolAddress<Z, R, W> {
    VolAddress {
      address: self.address,
      target: PhantomData,
      read_status: PhantomData,
      write_status: PhantomData,
    }
  }

  /// Converts the `VolAddress` back into a normal `usize` value.
  #[inline]
  #[must_use]
  pub const fn as_usize(self) -> usize {
    self.address.get()
  }

  /// Advances the pointer by the given number of positions (`usize`).
  ///
  /// Shorthand for `addr.offset(count as isize)`
  ///
  /// This is intended to basically work like [`<*mut
  /// T>::wrapping_add`](https://doc.rust-lang.org/std/primitive.pointer.html#method.wrapping_add-1).
  ///
  /// ## Safety
  /// * As per the type docs
  #[inline]
  #[must_use]
  pub const unsafe fn add(self, count: usize) -> Self {
    self.offset(count as isize)
  }

  /// Reverses the pointer by the given number of positions (`usize`).
  ///
  /// Shorthand for `addr.offset((count as isize).wrapping_neg())`
  ///
  /// This is intended to basically work like [`<*mut
  /// T>::wrapping_sub`](https://doc.rust-lang.org/std/primitive.pointer.html#method.wrapping_sub-1).
  ///
  /// ## Safety
  /// * As per the type docs
  #[inline]
  #[must_use]
  pub const unsafe fn sub(self, count: usize) -> Self {
    self.offset((count as isize).wrapping_neg())
  }

  /// Offsets the address by the given number of positions (`isize`).
  ///
  /// This is intended to basically work like [`<*mut
  /// T>::wrapping_offset`](https://doc.rust-lang.org/std/primitive.pointer.html#method.wrapping_offset-1).
  ///
  /// ## Safety
  /// * As per the type docs
  #[inline]
  #[must_use]
  pub const unsafe fn offset(self, count: isize) -> Self {
    let total_delta = core::mem::size_of::<T>().wrapping_mul(count as usize);
    VolAddress {
      address: NonZeroUsize::new_unchecked(
        self.address.get().wrapping_add(total_delta),
      ),
      target: PhantomData,
      read_status: PhantomData,
      write_status: PhantomData,
    }
  }
}
impl<T, W> VolAddress<T, Safe, W>
where
  T: Copy,
{
  /// Volatile reads the current value of `A`.
  #[inline]
  #[must_use]
  pub fn read(self) -> T {
    unsafe { read_volatile(self.address.get() as *const T) }
  }
}
impl<T, W> VolAddress<T, Unsafe, W>
where
  T: Copy,
{
  /// Volatile reads the current value of `A`.
  ///
  /// ## Safety
  /// * The safety rules of reading this address depend on the device. Consult
  ///   your hardware manual.
  #[inline]
  #[must_use]
  pub unsafe fn read(self) -> T {
    read_volatile(self.address.get() as *const T)
  }
}
impl<T, R> VolAddress<T, R, Safe>
where
  T: Copy,
{
  /// Volatile writes a new value to `A`.
  #[inline]
  pub fn write(self, t: T) {
    unsafe { write_volatile(self.address.get() as *mut T, t) }
  }
}
impl<T, R> VolAddress<T, R, Unsafe>
where
  T: Copy,
{
  /// Volatile writes a new value to `A`.
  ///
  /// ## Safety
  /// * The safety rules of reading this address depend on the device. Consult
  ///   your hardware manual.
  #[inline]
  pub unsafe fn write(self, t: T) {
    write_volatile(self.address.get() as *mut T, t)
  }
}

impl<T, R, W> Clone for VolAddress<T, R, W> {
  #[inline]
  #[must_use]
  fn clone(&self) -> Self {
    *self
  }
}
impl<T, R, W> Copy for VolAddress<T, R, W> {}

impl<T, R, W> core::cmp::PartialEq for VolAddress<T, R, W> {
  #[inline]
  #[must_use]
  fn eq(&self, other: &Self) -> bool {
    core::cmp::PartialEq::eq(&self.address.get(), &other.address.get())
  }
}
impl<T, R, W> core::cmp::Eq for VolAddress<T, R, W> {}

impl<T, R, W> core::cmp::PartialOrd for VolAddress<T, R, W> {
  #[inline]
  #[must_use]
  fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
    core::cmp::PartialOrd::partial_cmp(
      &self.address.get(),
      &other.address.get(),
    )
  }
}
impl<T, R, W> core::cmp::Ord for VolAddress<T, R, W> {
  #[inline]
  #[must_use]
  fn cmp(&self, other: &Self) -> core::cmp::Ordering {
    core::cmp::Ord::cmp(&self.address.get(), &other.address.get())
  }
}

impl<T, R, W> core::fmt::Debug for VolAddress<T, R, W> {
  #[cold]
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(
      f,
      "VolAddress<{elem_ty}, r{readability}, w{writeability}>({address:#X})",
      elem_ty = core::any::type_name::<T>(),
      readability = core::any::type_name::<R>(),
      writeability = core::any::type_name::<W>(),
      address = self.address.get()
    )
  }
}
