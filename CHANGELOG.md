# Changelog

## 1.2

* `Safe` and `Unsafe` now derive `Default`, `Clone`, and `Copy`.

## 1.1

* Added `VolAddress::as_volblock` for (unsafely) converting from a `VolAddress`
  to an array and a `VolBlock`.

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
