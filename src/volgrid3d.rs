use crate::{VolAddress, VolBlock, VolGrid2d};

/// A 3D version of [`VolGrid2d`], where multiple "frames" of memory are
/// present.
///
/// ## Generic Parameters
/// * `T` / `R` / `W`: These parameters are applied to the [`VolAddress`] type
///   returned when accessing the block in any way (indexing, iteration, etc).
/// * `WIDTH` / `HEIGHT`: the width and height of a given frame.
/// * `FRAMES`: the number of frames
/// * The total element count is `(WIDTH * HEIGHT) * FRAMES`
///
/// ## Safety
/// * This type stores a base [`VolAddress`] internally, and so you must follow
///   all of those safety rules. Notably, the base address must never be zero.
/// * The address space must legally contain `WIDTH * HEIGHT * FRAMES`
///   contiguous values of the `T` type, starting from the base address.
/// * The memory block must not wrap around past the end of the address space.
pub struct VolGrid3d<
  T,
  R,
  W,
  const WIDTH: usize,
  const HEIGHT: usize,
  const FRAMES: usize,
> {
  pub(crate) base: VolAddress<T, R, W>,
}

impl<T, R, W, const WIDTH: usize, const HEIGHT: usize, const FRAMES: usize>
  VolGrid3d<T, R, W, WIDTH, HEIGHT, FRAMES>
{
  /// A [`VolAddress`] with multi-frame access pattern.
  ///
  /// # Safety
  ///
  /// The given address must be a valid for `WIDTH * HEIGHT * FRAMES` elements.
  #[inline]
  #[must_use]
  pub const unsafe fn new(address: usize) -> Self {
    Self { base: VolAddress::new(address) }
  }

  /// Gets a single frame as a `VolGrid2d`.
  ///
  /// Returns `None` if `z` is out of bounds.
  #[inline]
  #[must_use]
  pub const fn get_frame(
    self, z: usize,
  ) -> Option<VolGrid2d<T, R, W, WIDTH, HEIGHT>> {
    if z < FRAMES {
      // SAFETY:
      // - `y < HEIGHT`
      // - `VolGrid3d::new` safety condition guarantees that all addresses
      //   constructible for `VolGrid2d<T,R,W,WIDTH,HEIGHT>` are valid
      //   `VolAddress`, which is the safety condition of `VolGrid2d::new`.
      Some(unsafe { VolGrid2d { base: self.base.add(z * (WIDTH * HEIGHT)) } })
    } else {
      None
    }
  }
}
