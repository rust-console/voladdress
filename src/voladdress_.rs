use super::*;

/// A volatile address.
///
/// This type stores a memory address and provides ergonomic volatile access to
/// said memory address.
///
/// Note that this type has several methods for accessing the data at the
/// address specified, and a particular instance of this type can use them
/// unsafely, use them safely, or not use them at all based on the generic
/// values of `R` and `W` (explained below).
/// * `read`
/// * `write`
/// * `apply` (reads, runs a function, then writes)
///
/// ## Generic Parameters
///
/// * `T`: The type of the value stored at the address.
///   * The target type type must impl `Copy` for reading and writing to be
///     allowed.
/// * `R`: If the address is readable.
///   * If `R=Safe` then you can safely read from the address.
///   * If `R=Unsafe` then you can unsafely read from the address.
///   * Otherwise you cannot read from the address.
/// * `W`: If the address is writable.
///   * If `W=Safe` then you can safely write to the address.
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
///   **not** a good fit for the `VolAddress` abstraction.
///
/// ## Safety
/// This type's safety follows the "unsafe creation, then safe use" strategy.
///
/// * **Validity Invariant**: The address of a `VolAddress` must always be
///   non-zero, or you will instantly trigger UB.
/// * **Safety Invariant**: The address of a `VolAddress` must be an aligned and
///   legal address for a `T` type value (with correct `R` and `W` permissions)
///   within the device's memory space, otherwise the `read` and `write` methods
///   will trigger UB when called.
/// * **Synchronization Invariant**: Volatile access has **no** cross-thread
///   synchronization behavior within the LLVM memory model. The results of
///   *all* volatile access is target-dependent, including cross-thread access.
///   Volatile access has no automatic synchronization of its own, and so if
///   your target requires some sort of synchronization for volatile accesses of
///   the address in question you must provide the appropriate synchronization
///   in some way external to this type.
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VolAddress<T, R, W> {
  pub(crate) address: NonZeroUsize,
  target: PhantomData<T>,
  read_status: PhantomData<R>,
  write_status: PhantomData<W>,
}

impl<T, R, W> Clone for VolAddress<T, R, W> {
  #[inline]
  #[must_use]
  fn clone(&self) -> Self {
    *self
  }
}
impl<T, R, W> Copy for VolAddress<T, R, W> {}

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

  /// Changes the permissions of the address to the new read and write
  /// permissions specified.
  ///
  /// ## Safety
  /// * As per the type docs
  #[inline]
  #[must_use]
  pub const unsafe fn change_permissions<NewRead, NewWrite>(
    self,
  ) -> VolAddress<T, NewRead, NewWrite> {
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

  /// Converts the `VolAddress` into const pointer form.
  ///
  /// This should usually only be used when you need to call a foreign function
  /// that expects a pointer.
  #[inline]
  #[must_use]
  pub const fn as_ptr(self) -> *const T {
    self.address.get() as *const T
  }

  /// Converts the `VolAddress` into mut pointer form.
  ///
  /// This should usually only be used when you need to call a foreign function
  /// that expects a pointer.
  #[inline]
  #[must_use]
  pub const fn as_mut_ptr(self) -> *mut T {
    self.address.get() as *mut T
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

impl<T, R, W, const C: usize> VolAddress<[T; C], R, W> {
  /// Converts an address for an array to a block for each element of the array.
  ///
  /// ## Safety
  /// * As per the `VolBlock` construction rules.
  /// * It is *highly likely* that on any device this is safe, but because of
  ///   possible strangeness with volatile side effects this is marked as an
  ///   `unsafe` method.
  #[inline]
  #[must_use]
  pub const unsafe fn as_volblock(self) -> VolBlock<T, R, W, C> {
    VolBlock { base: self.cast::<T>() }
  }
}

impl<T, W> VolAddress<T, Safe, W>
where
  T: Copy,
{
  /// Volatile reads the current value of `A`.
  #[inline]
  pub fn read(self) -> T {
    // Safety: The declarer of the value gave this a `Safe` read typing, thus
    // they've asserted that this is a safe to read address.
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
    // Safety: The declarer of the value gave this a `Safe` write typing, thus
    // they've asserted that this is a safe to write address.
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
  /// * The safety rules of writing this address depend on the device. Consult
  ///   your hardware manual.
  #[inline]
  pub unsafe fn write(self, t: T) {
    write_volatile(self.address.get() as *mut T, t)
  }
}

impl<T> VolAddress<T, Safe, Safe>
where
  T: Copy,
{
  /// Reads the address, applies the operation, and writes back the new value.
  #[inline]
  pub fn apply<F: FnOnce(&mut T)>(self, op: F) {
    let mut temp = self.read();
    op(&mut temp);
    self.write(temp);
  }
}
impl<T> VolAddress<T, Unsafe, Safe>
where
  T: Copy,
{
  /// Reads the address, applies the operation, and writes back the new value.
  ///
  /// ## Safety
  /// * The safety rules of reading/writing this address depend on the device.
  ///   Consult your hardware manual.
  #[inline]
  pub unsafe fn apply<F: FnOnce(&mut T)>(self, op: F) {
    let mut temp = self.read();
    op(&mut temp);
    self.write(temp);
  }
}
impl<T> VolAddress<T, Safe, Unsafe>
where
  T: Copy,
{
  /// Reads the address, applies the operation, and writes back the new value.
  ///
  /// ## Safety
  /// * The safety rules of reading/writing this address depend on the device.
  ///   Consult your hardware manual.
  #[inline]
  pub unsafe fn apply<F: FnOnce(&mut T)>(self, op: F) {
    let mut temp = self.read();
    op(&mut temp);
    self.write(temp);
  }
}
impl<T> VolAddress<T, Unsafe, Unsafe>
where
  T: Copy,
{
  /// Reads the address, applies the operation, and writes back the new value.
  ///
  /// ## Safety
  /// * The safety rules of reading/writing this address depend on the device.
  ///   Consult your hardware manual.
  #[inline]
  pub unsafe fn apply<F: FnOnce(&mut T)>(self, op: F) {
    let mut temp = self.read();
    op(&mut temp);
    self.write(temp);
  }
}

impl<T, R, W> core::fmt::Debug for VolAddress<T, R, W> {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(
      f,
      "VolAddress<{elem_ty}, r{readability}, w{writeability}>(0x{address:#X})",
      elem_ty = core::any::type_name::<T>(),
      readability = core::any::type_name::<R>(),
      writeability = core::any::type_name::<W>(),
      address = self.address.get()
    )
  }
}

impl<T, R, W> core::fmt::Pointer for VolAddress<T, R, W> {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(f, "0x{address:#X}", address = self.address.get())
  }
}
