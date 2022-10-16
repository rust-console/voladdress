# Changelog

## 1.2

* `Safe` and `Unsafe` now also derive `Default`, `Clone`, and `Copy`. This
  doesn't do too much since they're already ZSTs with a public constructor, but
  it doesn't hurt.
* `VolAddress`: Added const fn `as_ptr` and `as_mut_ptr`.
* `VolBlock`: Added const fn `as_usize`, `as_ptr`, `as_mut_ptr`, and non-const
  fn `as_slice_ptr` and `as_slice_mut_ptr`. It turns out that indexing by 0 and
  then calling methods on that index generated address was enough to (sometimes)
  confuse LLVM and prevent a lot of optimizations, so we want to support these
  direct conversions.

## 1.1

* Added `VolAddress::as_volblock` for (unsafely) converting from a `VolAddress`
  to an array into a `VolBlock`. This is totally fine in any case I've ever
  seen, but the general policy of the crate is that any creation of a `VolBlock`
  be an unsafe action, and so this is unsafe for consistency.

* Also adds the `experimental_volmatrix` cargo feature, which adds another
  opt-in type for people to experiment with.

## 1.0.2

* Temporarily adds the `experimental_volregion` feature, allowing a person to
  *experimentally* opt-in to the `VolRegion` type. This type will be part of a
  1.1 release of the crate at some point. This feature is **not** part of the
  crate's SemVer, and it will go away entirely once `VolRegion` becomes a stable
  part of the crate.

## 1.0.0

* Initial stable release.
