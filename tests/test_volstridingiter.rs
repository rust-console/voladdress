
use voladdress::{VolAddress, VolSeries, VolStridingIter};
use typenum::consts::{U16, U3};

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
