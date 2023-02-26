# Changelog

## 1.3.0

* New: `VolGrid2d<T,R,W, WIDTH, HEIGHT>` works like video memory (accessed with `(x,y)`)
* New `VolGrid2dStrided<T,R,W, WIDTH, HEIGHT, FRAMES, BYTE_STRIDE>` has many
  "frames" of `VolGrid2d`, each offset by the given stride in bytes. The stride
  can be larger, equal to, or smaller than the number of bytes per frame.
* New: `VolRegion<T,R,W>` is a 1d span with a dynamic size, like a slice.
* Removed: the "experimental" cargo features were removed from Cargo.toml.
  If you had opted-in to using them you will have to adjust your `[dependencies]` entry.

## 1.2.3

* Fixed up unclear documentation.

## 1.2.2

* **Soundness:** Previous versions of the iterators in
  this crate (since 0.4) had a math error in the `nth`
  method, causing them to potentially go out of bounds.

## 1.2

* The `Safe` and `Unsafe` types now also derive `Default`, `Clone`, and `Copy`.
  This doesn't do too much since they're already ZSTs with a public constructor,
  but it doesn't hurt.
* `VolAddress`: Added const fn `as_ptr` and `as_mut_ptr`.
* `VolBlock`: Added const fn `as_usize`, `as_ptr`, `as_mut_ptr`, and non-const
  fn `as_slice_ptr` and `as_slice_mut_ptr`.
* It turns out that getting the pointer to a `VolBlock` by indexing to the 0th
  element and then turning that into a usize and then turning that into a
  pointer was enough layers to confuse LLVM. Specifically, all volatile accesses
  have an "aligned and non-null" debug check, which wasn't getting optimized out
  of debug builds with `build-std`, even with `opt-level=3`. Providing these
  more direct conversion methods does seem to help LLVM eliminate that non-null
  check more often.
* Added `core::fmt::Pointer` impls. While `Debug` formats the address along with
  extra metadata, `Pointer` just formats the address.

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
