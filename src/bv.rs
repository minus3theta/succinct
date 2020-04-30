use num_integer::Integer;
use bitvec::prelude::{BitBox, Lsb0, BitStore};

type Store = BitBox<Lsb0, u32>;

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Hash, Clone)]
pub struct BV {
  array: Store,
  large_sum: Vec<u32>,
  small_sum: Vec<u16>,
}

impl BV {
  const LARGE: usize = 64 * 64;
  const SMALL: usize = 32;
  const S_PER_L: usize = Self::LARGE / Self::SMALL;

  pub fn new(array: Store) -> Self {
    let raw: &[<u32 as BitStore>::Access] = array.as_total_slice();
    let mut large_sum = vec![0; raw.len().div_ceil(&Self::S_PER_L)];
    let mut small_sum = vec![0; raw.len()];
    let mut current_l_sum = 0;
    let mut current_s_sum = 0;
    for (i, x) in raw.iter().enumerate() {
      let x = x.load(core::sync::atomic::Ordering::Relaxed);
      let ones = x.count_ones();
      if i % Self::S_PER_L == 0 {
        large_sum[i / Self::S_PER_L] = current_l_sum;
        current_s_sum = 0;
      }
      small_sum[i] = current_s_sum;
      current_s_sum += ones as u16;
      current_l_sum += ones;
    }
    BV {
      array,
      large_sum,
      small_sum,
    }
  }

  pub fn get(&self, i: usize) -> Option<&bool> {
    self.array.get(i)
  }

  pub fn rank1(&self, i: usize) -> u32 {
    todo!()
  }
}

impl std::ops::Index<usize> for BV {
  type Output = bool;
  fn index(&self, i: usize) -> &Self::Output {
    &self.array[i]
  }
}

#[cfg(test)]
mod tests {
  use bitvec::prelude::*;
  use super::*;
  #[test]
  fn new_l_size_1() {
    assert_eq!(BV::new(bitbox![Lsb0, u32; 0]).large_sum.len(), 1);
  }
  #[test]
  fn new_l_size_large() {
    assert_eq!(BV::new(bitbox![Lsb0, u32; 0; BV::LARGE]).large_sum.len(), 1);
  }
  #[test]
  fn new_l_size_large_plus_1() {
    assert_eq!(BV::new(bitbox![Lsb0, u32; 1; BV::LARGE + 1]).large_sum.len(), 2);
  }
  #[test]
  fn new_large_sum() {
    assert_eq!(BV::new(bitbox![Lsb0, u32; 1; BV::LARGE + 1]).large_sum, vec![0, 4096]);
  }
  #[test]
  fn new_small_sum() {
    let bv = BV::new((0 .. 50).map(|i| i < 20).collect::<BitVec<Lsb0, u32>>().into());
    assert_eq!(bv.small_sum, vec![0, 20]);
  }
}
