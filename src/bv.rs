use num_integer::Integer;
use bitvec::prelude::{BitSlice, Lsb0, BitStore};

type StoreUnit = u16;
type Store = BitSlice<Lsb0, StoreUnit>;

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Hash, Clone)]
pub struct BV<'a> {
  array: &'a Store,
  large_sum: Vec<u32>,
  small_sum: Vec<u16>,
}

impl <'a> BV<'a> {
  const SMALL: usize = StoreUnit::BITS as usize;
  const LARGE: usize = 4 * Self::SMALL * Self::SMALL;
  const S_PER_L: usize = Self::LARGE / Self::SMALL;

  pub fn new(array: &'a Store) -> Self {
    let raw: &[<StoreUnit as BitStore>::Access] = array.as_total_slice();
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
    self.large_sum[i / Self::LARGE] + self.small_sum[i / Self::SMALL] as u32 +
      (i / Self::SMALL * Self::SMALL .. i).filter(|&j| self.array[j]).count() as u32
  }

  pub fn rank0(&self, i: usize) -> u32 {
    i as u32 - self.rank1(i)
  }

  pub fn rank1_with_table(&self, i: usize, table: &Vec<Vec<u8>>) -> u32 {
    let raw = self.array.as_total_slice()[i / Self::S_PER_L].load(core::sync::atomic::Ordering::Relaxed);
    self.large_sum[i / Self::LARGE] + self.small_sum[i / Self::SMALL] as u32 +
      table[raw as usize][i % Self::SMALL] as u32
  }

  pub fn rank0_with_table(&self, i: usize, table: &Vec<Vec<u8>>) -> u32 {
    i as u32 - self.rank1_with_table(i, table)
  }

  pub fn rank_lookup_table() -> Vec<Vec<u8>> {
    let mut table = vec![vec![0; Self::SMALL]; 1 << Self::SMALL];
    for (value, row) in table.iter_mut().enumerate() {
      let mut sum = 0;
      for (pos, count) in row.iter_mut().enumerate() {
        *count = sum;
        if (value >> pos) & 1 == 1 {
          sum += 1;
        }
      }
    }
    table
  }
}

impl std::ops::Index<usize> for BV<'_> {
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
    assert_eq!(BV::new(bits![Lsb0, u16; 0]).large_sum.len(), 1);
  }
  #[test]
  fn new_l_size_large() {
    assert_eq!(BV::new(bits![Lsb0, u16; 0; BV::LARGE]).large_sum.len(), 1);
  }
  #[test]
  fn new_l_size_large_plus_1() {
    assert_eq!(BV::new(bits![Lsb0, u16; 1; BV::LARGE + 1]).large_sum.len(), 2);
  }
  #[test]
  fn new_large_sum() {
    assert_eq!(BV::new(bits![Lsb0, u16; 1; BV::LARGE + 1]).large_sum, vec![0, 1024]);
  }
  #[test]
  fn new_small_sum() {
    let v = (0 .. 50).map(|i| i < 20).collect::<BitVec<Lsb0, u16>>();
    let bv = BV::new(&v);
    assert_eq!(bv.small_sum, vec![0, 16, 20, 20]);
  }
  #[test]
  fn rank() {
    let bv = BV::new(bits![Lsb0, u16; 1; BV::LARGE + 1]);
    assert_eq!(bv.rank1(10), 10);
    assert_eq!(bv.rank1(1000), 1000);
  }
  #[test]
  fn rank_with_table() {
    let bv = BV::new(bits![Lsb0, u16; 1; BV::LARGE + 1]);
    let table = BV::rank_lookup_table();
    assert_eq!(bv.rank1_with_table(10, &table), 10);
    assert_eq!(bv.rank1_with_table(1000, &table), 1000);
  }
}
