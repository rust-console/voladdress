use crate::{VolAddress, VolGrid2d};

/// Models having many "frames" of [`VolGrid2d`] within a chunk of memory.
///
/// Each frame may or may not overlap, according to the stride specified.
/// * If the byte stride per frame is less than the byte size of a frame, the
///   frames will have some amount of overlap.
/// * If the stride bytes equals the frame bytes, then each frame will directly
///   follow the previous one.
/// * If the stride bytes exceeds the frame bytes, then there will be some
///   amount of gap between frames.
///
/// ## Generic Parameters
/// * `T` / `R` / `W`: These parameters are applied to the [`VolAddress`] type
///   returned when accessing the block in any way (indexing, iteration, etc).
/// * `WIDTH` / `HEIGHT`: the width and height of a given frame.
/// * `FRAMES`: the number of frames.
/// * `BYTE_STRIDE`: The number of bytes between the start of each frame.
///
/// ## Safety
/// * This type stores a base [`VolAddress`] internally, and so you must follow
///   all of those safety rules. Notably, the base address must never be zero.
/// * The address space must legally contain `WIDTH * HEIGHT * FRAMES`
///   contiguous values of the `T` type, starting from the base address.
/// * The memory block must not wrap around past the end of the address space.
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VolGrid2dStrided<
  T,
  R,
  W,
  const WIDTH: usize,
  const HEIGHT: usize,
  const FRAMES: usize,
  const BYTE_STRIDE: usize,
> {
  pub(crate) base: VolAddress<T, R, W>,
}

impl<
    T,
    R,
    W,
    const WIDTH: usize,
    const HEIGHT: usize,
    const FRAMES: usize,
    const BYTE_STRIDE: usize,
  > Clone for VolGrid2dStrided<T, R, W, WIDTH, HEIGHT, FRAMES, BYTE_STRIDE>
{
  #[inline]
  #[must_use]
  fn clone(&self) -> Self {
    *self
  }
}
impl<
    T,
    R,
    W,
    const WIDTH: usize,
    const HEIGHT: usize,
    const FRAMES: usize,
    const BYTE_STRIDE: usize,
  > Copy for VolGrid2dStrided<T, R, W, WIDTH, HEIGHT, FRAMES, BYTE_STRIDE>
{
}

impl<
    T,
    R,
    W,
    const WIDTH: usize,
    const HEIGHT: usize,
    const FRAMES: usize,
    const BYTE_STRIDE: usize,
  > VolGrid2dStrided<T, R, W, WIDTH, HEIGHT, FRAMES, BYTE_STRIDE>
{
  /// A [`VolAddress`] with multi-frame access pattern.
  ///
  /// # Safety
  ///
  /// The given address must be a valid for `WIDTH * HEIGHT` elements per frame,
  /// at frame indexes `0..FRAMES`, with all non-zero frame indexes being offset
  /// by `BYTE_STRIDE` bytes from the previous frame.
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
      // - `z` is in bounds of `FRAMES`.
      // - `VolGrid3d::new` safety condition guarantees that all `VolGrid2d`
      //   values we could construct for `0..FRAMES` are valid.
      Some(unsafe {
        VolGrid2d {
          base: self.base.cast::<u8>().add(z * BYTE_STRIDE).cast::<T>(),
        }
      })
    } else {
      None
    }
  }
}

#[test]
fn test_vol_grid_2d_strided() {
  let small: VolGrid2dStrided<u8, (), (), 10, 10, 6, 0x100> =
    unsafe { VolGrid2dStrided::new(0x1000) };
  assert_eq!(small.get_frame(0).unwrap().as_usize(), 0x1000);
  assert_eq!(small.get_frame(1).unwrap().as_usize(), 0x1100);
  assert_eq!(small.get_frame(2).unwrap().as_usize(), 0x1200);
  assert_eq!(small.get_frame(3).unwrap().as_usize(), 0x1300);
  assert_eq!(small.get_frame(4).unwrap().as_usize(), 0x1400);
  assert_eq!(small.get_frame(5).unwrap().as_usize(), 0x1500);
  assert!(small.get_frame(6).is_none());
}
