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
pub struct VolGrid2d<T, R, W, const WIDTH: usize, const HEIGHT: usize> {
  base: VolAddress<T, R, W>,
}

/// Direct index access methods.
impl<T, R, W, const WIDTH: usize, const HEIGHT: usize>
  VolGrid2d<T, R, W, WIDTH, HEIGHT>
{
  /// A [`VolAddress`] with matrix-style access pattern.
  ///
  /// # Safety
  ///
  /// The given address must be a valid [`VolAddress`] at each position in the
  /// matrix:
  ///
  /// ```text
  /// for all (X, Y) in (0..WIDTH, 0..HEIGHT):
  ///     let accessible = address + mem::size_of::<T>() * (X + WIDTH * Y);
  ///     assert_valid_voladdress(accessible);
  /// ```
  #[inline]
  #[must_use]
  pub const unsafe fn new(address: usize) -> Self {
    Self { base: VolAddress::new(address) }
  }

  /// Create a two-dimensional table from a 1D `VolBlock`.
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
    // SAFETY: block's safety requirement is that all VolAddress accessible within
    // it are safe, Self can only access those addresses, so Self::new requirement
    // is fulfilled.
    Self { base: block.base }
  }

  /// Convert this 2D block into a single 1D [`VolBlock`] spanning the whole
  /// matrix.
  ///
  /// # Panics
  ///
  /// When `B != WIDTH * HEIGHT`.
  /// Note that such a panic should happen at compile time.
  #[inline]
  #[must_use]
  pub const fn into_block<const B: usize>(self) -> VolBlock<T, R, W, B> {
    assert!(B == WIDTH * HEIGHT);
    // SAFETY: block's safety requirement is that all VolAddress accessible within
    // it are safe, all constructors of `VolGrid2d` already guarantees that.
    VolBlock { base: self.base }
  }

  /// Get the [`VolAddress`] at specified matrix location, returns
  /// `None` if out of bound.
  #[inline]
  #[must_use]
  pub const fn get(self, x: usize, y: usize) -> Option<VolAddress<T, R, W>> {
    if x < WIDTH && y < HEIGHT {
      // SAFETY: if x < WIDTH && y < HEIGHT
      Some(unsafe { self.base.add(x + y * WIDTH) })
    } else {
      None
    }
  }

  /// Indexes at `y * HEIGHT + x` the matrix.
  ///
  /// ## Panics
  ///
  /// * If `x >= WIDTH || y >= HEIGHT`.
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
}

/// Row access methods.
impl<T, R, W, const WIDTH: usize, const HEIGHT: usize>
  VolGrid2d<T, R, W, WIDTH, HEIGHT>
{
  /// Get a single row of the matrix as a [`VolBlock`].
  #[inline]
  #[must_use]
  pub const fn get_row(self, y: usize) -> Option<VolBlock<T, R, W, WIDTH>> {
    if y < HEIGHT {
      // SAFETY:
      // - `y < HEIGHT`
      // - `VolGrid2d::new` safety condition guarantees that all addresses
      //   constructible for `VolBlock<T, WIDTH>` are valid `VolAddress`,
      //   which is the safety condition of `VolBlock::new`.
      Some(unsafe { VolBlock { base: self.base.add(y * WIDTH) } })
    } else {
      None
    }
  }
}

/*
# This requires extended const_generic support, which is highly not going
# to happen soon in stable, so we feature-gate.
VolGrid2d_column = []
/// Column access methods.
#[cfg(feature = "VolGrid2d_column")]
impl<T, R, W, const WIDTH: usize, const HEIGHT: usize>
  VolGrid2d<T, R, W, WIDTH, HEIGHT>
{
  /// Get a signle column of the matrix as a [`VolSeries`].
  #[inline]
  #[must_use]
  pub const fn get_column(
    self, x: usize,
  ) -> Option<VolSeries<T, R, W, HEIGHT, { WIDTH * core::mem::size_of::<T>() }>>
  {
    if x < WIDTH {
      // SAFETY:
      // - `x < WIDTH` (hence, will never spill out of the matrix)
      // - `VolGrid2d::new` safety condition guarantees that all addresses
      //   constructible for `VolSeries<T, HEIGHT, …>` are valid `VolAddress`,
      //   which is the safety condition of `VolSeries::new`.
      Some(unsafe { VolSeries { base: self.base.add(x) } })
    } else {
      None
    }
  }
}
*/
