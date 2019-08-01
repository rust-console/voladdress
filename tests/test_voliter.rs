use voladdress::{VolAddress, VolIter};

#[test]
fn test_size_hint_and_next() {
  let a: VolAddress<i32> = unsafe { VolAddress::new(4) };

  unsafe {
    assert_eq!(a.iter_slots(0).size_hint(), (0, Some(0)));
    assert_eq!(a.iter_slots(1).size_hint(), (1, Some(1)));
  }

  let mut i: VolIter<i32> = unsafe { a.iter_slots(2) };
  assert_eq!(i.size_hint(), (2, Some(2)));

  assert_eq!(i.next().unwrap(), unsafe { VolAddress::new(0x4) });
  assert_eq!(i.size_hint(), (1, Some(1)));

  assert_eq!(i.next().unwrap(), unsafe { VolAddress::new(0x8) });
  assert_eq!(i.size_hint(), (0, Some(0)));

  assert!(i.next().is_none());
  assert_eq!(i.size_hint(), (0, Some(0)));
}

#[test]
fn test_count() {
  let a: VolAddress<i32> = unsafe { VolAddress::new(4) };
  let i: VolIter<i32> = unsafe { a.iter_slots(2) };

  assert_eq!(i.count(), 2);
}

#[test]
fn test_last() {
  let a: VolAddress<i32> = unsafe { VolAddress::new(4) };
  let i: VolIter<i32> = unsafe { a.iter_slots(2) };

  assert_eq!(i.last(), Some(unsafe { VolAddress::new(4 + 2 * 4) }));

  let mut i: VolIter<i32> = unsafe { a.iter_slots(2) };
  i.next();
  i.next();
  assert_eq!(i.last(), None);
}

#[test]
fn test_nth() {
  let a: VolAddress<i32> = unsafe { VolAddress::new(4) };
  let mut i: VolIter<i32> = unsafe { a.iter_slots(2) };
  let mut i2: VolIter<i32> = i.clone();

  assert_eq!(i.nth(0), i2.next());
  assert_eq!(i.nth(0), i2.next());
  assert_eq!(i.nth(0), i2.next());

  let mut i: VolIter<i32> = unsafe { a.iter_slots(2) };
  assert_eq!(i.nth(0), Some(unsafe { VolAddress::new(4) }));

  let mut i: VolIter<i32> = unsafe { a.iter_slots(2) };
  assert_eq!(i.nth(1), Some(unsafe { VolAddress::new(8) }));

  let mut i: VolIter<i32> = unsafe { a.iter_slots(2) };
  assert_eq!(i.nth(2), None);
}
