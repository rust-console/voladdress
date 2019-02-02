
use voladdress::VolBlock;
use typenum::consts::U256;

const DUMMY: VolBlock<i32, U256> = unsafe { VolBlock::new(4) };

#[test]
fn test_iter() {
  let i = DUMMY.iter();
  let len = DUMMY.len();
  assert_eq!(i.size_hint(), (len, Some(len)));
  assert_eq!(i.count(), len);
}

#[test]
fn test_indexing_styles() {
  let a0 = unsafe { DUMMY.index_unchecked(0) };
  let b0 = DUMMY.index(0);
  assert_eq!(a0, b0);

  let a1 = unsafe { a0.offset(1) };
  let b1 = DUMMY.index(1);
  assert_eq!(a1, b1);

  for i in 0 .. DUMMY.len() {
    assert_eq!(DUMMY.get(i).unwrap(), DUMMY.index(i));
  }
  assert!(DUMMY.get(DUMMY.len()).is_none());
}

#[test]
#[should_panic]
fn test_index_panic() {
  DUMMY.index(DUMMY.len());
}
