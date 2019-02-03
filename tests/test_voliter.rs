
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
