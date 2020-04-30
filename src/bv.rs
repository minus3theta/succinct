use num_integer::Integer;
use bitvec::prelude::BitBox;

type Store = Vec<u32>;

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Hash, Clone)]
pub struct BV {
  n: usize,
  array: Store,
  l_sum: Vec<u32>,
  s_sum: Vec<u8>,
}

impl BV {
  const LARGE: usize = 64 * 64;
  const SMALL: usize = 32;

  pub fn new(n: usize, array: Store) -> Self {
    assert_eq!(array.len(), n.div_ceil(&Self::SMALL));
    let mut l_sum = Vec::new();
    let mut s_sum = Vec::new();
    BV {
      n,
      array,
      l_sum,
      s_sum,
    }
  }

  pub fn get(&self, i: usize) -> Option<bool> {
    if i < self.n {
      let block = self.array.get(i / Self::SMALL)?;
      Some((block >> (i % Self::SMALL)) & 1 == 1)
    } else {
      None
    }
  }

  pub fn rank1(&self, i: usize) -> u32 {
    todo!()
  }
}

#[cfg(test)]
mod tests {
  use bitvec::prelude::*;
  use super::*;
  #[test]
  fn get_0() {
    assert_eq!(BV::new(2, vec![2]).get(0), Some(false))
  }
  #[test]
  fn get_1() {
    assert_eq!(BV::new(2, vec![2]).get(1), Some(true))
  }
  #[test]
  fn get_outofbounds() {
    assert_eq!(BV::new(2, vec![2]).get(2), None)
  }
}
