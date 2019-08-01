
# Changelog

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
