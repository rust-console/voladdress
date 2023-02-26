use crate::{VolAddress, VolBlock};

/// A 2D version of [`VolBlock`], with a const generic `WIDTH` and `HEIGHT`.
///
/// This is intended for "video-like" memory that is better to logically access
/// with an `x` and `y` position rather than a single `i` index. It's just an
/// alternative way to manage a `VolBlock`.
///
/// ## Generic Parameters
/// * `T` / `R` / `W`: These parameters are applied to the [`VolAddress`] type
///   returned when accessing the block in any way (indexing, iteration, etc).
/// * `WIDTH` / `HEIGHT`: the matrix width and height, the total element count
///   is `WIDTH * HEIGHT`.
///
/// ## Safety
/// * This type stores a base [`VolAddress`] internally, and so you must follow
///   all of those safety rules. Notably, the base address must never be zero.
/// * The address space must legally contain `WIDTH * HEIGHT` contiguous values
///   of the `T` type, starting from the base address.
/// * The memory block must not wrap around past the end of the address space.
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VolGrid2d<T, R, W, const WIDTH: usize, const HEIGHT: usize> {
  pub(crate) base: VolAddress<T, R, W>,
}

impl<T, R, W, const WIDTH: usize, const HEIGHT: usize> Clone
  for VolGrid2d<T, R, W, WIDTH, HEIGHT>
{
  #[inline]
  #[must_use]
  fn clone(&self) -> Self {
    *self
  }
}
impl<T, R, W, const WIDTH: usize, const HEIGHT: usize> Copy
  for VolGrid2d<T, R, W, WIDTH, HEIGHT>
{
}

impl<T, R, W, const WIDTH: usize, const HEIGHT: usize>
  VolGrid2d<T, R, W, WIDTH, HEIGHT>
{
  /// Converts the address into a `VolGrid2d`
  ///
  /// # Safety
  ///
  /// The given address must be a valid [`VolAddress`] at each position in the
  /// grid, as if you were making a `VolBlock<T,R,W,{WIDTH * HEIGHT}>`.
  #[inline]
  #[must_use]
  pub const unsafe fn new(address: usize) -> Self {
    Self { base: VolAddress::new(address) }
  }

  /// Creates a `VolGrid2d` from an appropriately sized `VolBlock`.
  ///
  /// # Panics
  ///
  /// When `B != WIDTH * HEIGHT`.
  /// Note that such a panic should happen at compile time.
  #[inline]
  #[must_use]
  pub const fn from_block<const B: usize>(block: VolBlock<T, R, W, B>) -> Self {
    // TODO: one day in the distant future, when full const_generic is
    // implemented in rust, someone may be interested in coming down from their
    // flying car, replace the `B` parameter by `{ WIDTH * HEIGHT }` and remove
    // the assert! (same with into_block)
    assert!(B == WIDTH * HEIGHT);
    // SAFETY: block's safety requirement is that all VolAddress accessible
    // within it are safe, Self can only access those addresses, so
    // Self::new requirement is fulfilled.
    Self { base: block.base }
  }

  /// Turn a `VolGrid2d` into its `VolBlock` equivalent.
  ///
  /// # Panics
  ///
  /// When `B != WIDTH * HEIGHT`.
  /// Note that such a panic should happen at compile time.
  #[inline]
  #[must_use]
  pub const fn into_block<const B: usize>(self) -> VolBlock<T, R, W, B> {
    assert!(B == WIDTH * HEIGHT);
    // SAFETY: block's safety requirement is that all VolAddress accessible
    // within it are safe, all constructors of `VolGrid2d` already
    // guarantees that.
    VolBlock { base: self.base }
  }

  /// Gets the address of the `(x,y)` given.
  ///
  /// Returns `None` if either coordinate it out of bounds.
  #[inline]
  #[must_use]
  pub const fn get(self, x: usize, y: usize) -> Option<VolAddress<T, R, W>> {
    if x < WIDTH && y < HEIGHT {
      // SAFETY: if condition
      Some(unsafe { self.base.add(x + y * WIDTH) })
    } else {
      None
    }
  }

  /// Indexes the address of the `(x,y)` given.
  ///
  /// ## Panics
  ///
  /// * If either coordinate it out of bounds this will panic.
  #[inline]
  #[must_use]
  #[track_caller]
  pub const fn index(self, x: usize, y: usize) -> VolAddress<T, R, W> {
    match self.get(x, y) {
      Some(address) => address,
      None => {
        // Note(Lokathor): We force a const panic by indexing out of bounds.
        #[allow(unconditional_panic)]
        unsafe {
          VolAddress::new([usize::MAX][1])
        }
      }
    }
  }

  /// Get a single row of the grid as a [`VolBlock`].
  #[inline]
  #[must_use]
  pub const fn get_row(self, y: usize) -> Option<VolBlock<T, R, W, WIDTH>> {
    if y < HEIGHT {
      // SAFETY:
      // - `y < HEIGHT`
      // - `VolGrid2d::new` safety condition guarantees that all addresses
      //   constructible for `VolBlock<T, WIDTH>` are valid `VolAddress`, which
      //   is the safety condition of `VolBlock::new`.
      Some(unsafe { VolBlock { base: self.base.add(y * WIDTH) } })
    } else {
      None
    }
  }

  /// Converts the `VolGrid2d` the `usize` for the start of the grid.
  #[inline]
  #[must_use]
  pub const fn as_usize(self) -> usize {
    self.base.address.get()
  }
}
