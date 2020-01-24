
# Changelog

## v0.3.0

* Removed the dependency on the `typenum` crate by adopting the nightly-only
  `const_generics` feature.
* Added the `Zlib` license as an option. It has the same freedom to compile the
  crate into your program without needing to credit this crate (though source
  redistribution requires that you attach the license). The main difference is
  that `Zlib` is a license that's approved for people to use in Google projects,
  while `0BSD` is unfortunately not.

## v0.2.4

* After seeing results in profiling when using `opt-level=s`, added
  `#[inline(always)]` to essentially every function, since they're almost all
  single expression wrapper.

## v0.2.3

* Just fixed a docs typo

## v0.2.2

* Started a Changelog, let's see how long I keep it going.
* Added `to_usize` method.
* Added read only and write only variants.
