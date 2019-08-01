use voladdress::VolAddress;

#[test]
fn test_read_write() {
  let mut x = 5i32;
  let a: VolAddress<i32> = unsafe { VolAddress::new(&mut x as *mut i32 as usize) };
  assert_eq!(a.read(), 5);
  a.write(7);
  assert_eq!(a.read(), 7);
  assert_eq!(x, 7);
}

#[test]
fn test_offset() {
  let a: VolAddress<i32> = unsafe { VolAddress::new(12) };
  let b: VolAddress<i32> = unsafe { VolAddress::new(12 + 4) };
  assert_eq!(unsafe { a.offset(1) }, b);
}

#[test]
fn test_is_aligned() {
  let a: VolAddress<i32> = unsafe { VolAddress::new(12) };
  let b: VolAddress<i32> = unsafe { VolAddress::new(12 + 1) };
  assert!(a.is_aligned());
  assert!(!b.is_aligned());
}

#[test]
fn test_formatting() {
  let a: VolAddress<i32> = unsafe { VolAddress::new(4) };
  assert_eq!(&format!("{:?}", a), "VolAddress(0x4)");
  assert_eq!(&format!("{:p}", a), "0x4");
}
