use rand::distributions::{Distribution, WeightedIndex};
use rand::Rng;
use std::collections::HashMap;

pub(crate) trait RouletteCell {
  fn key(&self) -> &str; // トークの識別子: 全体において一意である必要がある
}

impl RouletteCell for String {
  fn key(&self) -> &str {
    self
  }
}

impl RouletteCell for &str {
  fn key(&self) -> &str {
    self
  }
}

// 重みの合計が u64 を溢れないためのシフト量上限
const MAX_BIAS_EXP: u32 = 40;

// 未選択回数 count を重みに変換する。0 は直前に選ばれたことを意味し、抽選から除外される
fn calc_bias(count: u32) -> u64 {
  if count == 0 {
    0
  } else {
    1u64 << count.min(MAX_BIAS_EXP)
  }
}

pub(crate) struct TalkBias(HashMap<String, u32>);

impl TalkBias {
  pub fn new() -> TalkBias {
    TalkBias(HashMap::new())
  }

  pub fn reset(&mut self, digest: &str) {
    self.0.insert(digest.to_string(), 0);
  }

  pub fn get(&self, digest: &str) -> u32 {
    *self.0.get(digest).unwrap_or(&1)
  }

  pub fn increment(&mut self, digest: &str) {
    self.0.insert(digest.to_string(), self.get(digest) + 1);
  }

  pub fn roulette(&mut self, cells: &[impl RouletteCell], is_consume: bool) -> Option<usize> {
    if cells.is_empty() {
      return None;
    }
    let mut rng = rand::thread_rng();

    let weights: Vec<u64> = cells.iter().map(|s| calc_bias(self.get(s.key()))).collect();

    let selected_index = match WeightedIndex::new(&weights) {
      Ok(dist) => dist.sample(&mut rng),
      // 全重みゼロ(1件リストの直後など)は一様抽選
      Err(_) => rng.gen_range(0..cells.len()),
    };

    if is_consume {
      for (i, cell) in cells.iter().enumerate() {
        if i == selected_index {
          // 選ばれたトークの重みを0に
          self.reset(cell.key());
        } else {
          // 全体の1/2が消費されるまで、それまでのトークが再び選ばれる可能性は生まれない
          self.increment(cell.key());
        }
      }
    }

    Some(selected_index)
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::events::talk::Talk;

  #[test]
  fn test_talk_bias() {
    let mut bias = TalkBias::new();
    for key in ["a", "b", "c", "d", "e", "f", "g", "h"] {
      bias.reset(key);
    }

    let talks: Vec<Talk> = ["a", "b", "c", "d", "e", "f", "g", "h"]
      .iter()
      .map(|s| Talk::new(None, s.to_string(), s.to_string(), None))
      .collect();

    let mut indexes: Vec<usize> = vec![];
    let mut select_count: Vec<i32> = vec![0; talks.len()];

    for _ in 0..100 {
      let selected_index = bias.roulette(&talks, true).unwrap();
      if let Some(last) = indexes.last() {
        if last == &selected_index {
          println!("duplication: {}", selected_index);
        }
      };
      let biases: Vec<u64> = talks.iter().map(|s| calc_bias(bias.get(s.key()))).collect();
      println!("biases: {:?}", biases);
      indexes.push(selected_index);
      select_count[selected_index] += 1;
    }

    let mut duplication = 0;
    for i in 0..indexes.len() - 1 {
      if indexes[i] == indexes[i + 1] {
        duplication += 1;
      }
    }
    assert_eq!(duplication, 0);

    // 全トークがほぼ等しい回数選ばれること (100回 / 8件 ≈ 12.5回)
    assert!(
      select_count.iter().all(|&c| c >= 5),
      "select_count is unbalanced: {:?}",
      select_count
    );

    println!("indexes: {:?}", indexes);
    println!("select_count: {:?}", select_count);
  }
}
