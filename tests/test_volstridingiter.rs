use typenum::consts::{U16, U3};
use voladdress::{VolAddress, VolSeries, VolStridingIter};

#[test]
fn test_size_hint_and_next() {
  let s: VolSeries<i32, U3, U16> = unsafe { VolSeries::new(4) };
  let mut i: VolStridingIter<i32, U16> = s.iter();
  assert_eq!(i.size_hint(), (3, Some(3)));

  assert_eq!(i.next().unwrap(), unsafe { VolAddress::new(0x4) });
  assert_eq!(i.size_hint(), (2, Some(2)));

  assert_eq!(i.next().unwrap(), unsafe { VolAddress::new(0x14) });
  assert_eq!(i.size_hint(), (1, Some(1)));

  assert_eq!(i.next().unwrap(), unsafe { VolAddress::new(0x24) });
  assert_eq!(i.size_hint(), (0, Some(0)));

  assert!(i.next().is_none());
  assert_eq!(i.size_hint(), (0, Some(0)));
}

#[test]
fn test_count() {
  let s: VolSeries<i32, U3, U16> = unsafe { VolSeries::new(4) };
  let i: VolStridingIter<i32, U16> = s.iter();

  assert_eq!(i.count(), 3);
}

#[test]
fn test_last() {
  let s: VolSeries<i32, U3, U16> = unsafe { VolSeries::new(4) };
  let i: VolStridingIter<i32, U16> = s.iter();

  assert_eq!(i.last(), Some(unsafe { VolAddress::new(4 + 3 * 16) }));

  let mut i: VolStridingIter<i32, U16> = s.iter();
  i.next();
  i.next();
  i.next();
  assert_eq!(i.last(), None);
}

#[test]
fn test_nth() {
  let s: VolSeries<i32, U3, U16> = unsafe { VolSeries::new(4) };
  let mut i: VolStridingIter<i32, U16> = s.iter();
  let mut i2: VolStridingIter<i32, U16> = i.clone();

  assert_eq!(i.nth(0), i2.next());
  assert_eq!(i.nth(0), i2.next());
  assert_eq!(i.nth(0), i2.next());

  let mut i: VolStridingIter<i32, U16> = s.iter();
  assert_eq!(i.nth(0), Some(unsafe { VolAddress::new(4) }));

  let mut i: VolStridingIter<i32, U16> = s.iter();
  assert_eq!(i.nth(1), Some(unsafe { VolAddress::new(4 + 16) }));

  let mut i: VolStridingIter<i32, U16> = s.iter();
  assert_eq!(i.nth(2), Some(unsafe { VolAddress::new(4 + 16 * 2) }));

  let mut i: VolStridingIter<i32, U16> = s.iter();
  assert_eq!(i.nth(3), None);
}
