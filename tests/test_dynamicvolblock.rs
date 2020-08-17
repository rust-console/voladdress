use voladdress::DynamicVolBlock;

#[test]
fn test_iter() {
  let dummy: DynamicVolBlock<i32> = unsafe { DynamicVolBlock::new(4, 256) };
  let i = dummy.iter();
  let len = dummy.len();
  assert_eq!(i.size_hint(), (len, Some(len)));
  assert_eq!(i.count(), len);
}

#[test]
fn test_indexing_styles() {
  let dummy: DynamicVolBlock<i32> = unsafe { DynamicVolBlock::new(4, 256) };
  let a0 = unsafe { dummy.index_unchecked(0) };
  let b0 = dummy.index(0);
  assert_eq!(a0, b0);

  let a1 = unsafe { a0.offset(1) };
  let b1 = dummy.index(1);
  assert_eq!(a1, b1);

  for i in 0..dummy.len() {
    assert_eq!(dummy.get(i).unwrap(), dummy.index(i));
  }
  assert!(dummy.get(dummy.len()).is_none());
}

#[test]
#[should_panic]
fn test_index_panic() {
  let dummy: DynamicVolBlock<i32> = unsafe { DynamicVolBlock::new(4, 256) };
  dummy.index(dummy.len());
}
