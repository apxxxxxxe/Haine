use rand::Rng;
use std::collections::HashMap;

pub struct TalkBias(HashMap<String, i32>);

impl TalkBias {
  pub fn new() -> TalkBias {
    TalkBias(HashMap::new())
  }

  pub fn add(&mut self, digest: String, bias: i32) {
    self.0.insert(digest, bias);
  }

  pub fn reset(&mut self, digest: String) {
    self.0.insert(digest, 0);
  }

  pub fn get(&self, digest: &String) -> i32 {
    *self.0.get(digest).unwrap_or(&1)
  }

  fn increment(&mut self, digest: String) {
    let max = 2 ^ 32 - 1;
    let mut bias = self.get(&digest);
    if bias == 0 {
      bias = 1;
    } else {
      bias <<= 1;
      if bias > max {
        bias = max;
      }
    }
    self.add(digest, bias);
  }

  pub fn roulette(&mut self, talks: &Vec<String>, is_consume: bool) -> usize {
    let bias_vec: Vec<i32> = talks.iter().map(|s| self.get(s)).collect();

    let cumulative_sum: Vec<i32> = bias_vec
      .iter()
      .scan(0, |acc, &x| {
        *acc += x;
        Some(*acc)
      })
      .collect();

    let selected_index: usize;
    let mut rng = rand::thread_rng();
    let sum = *cumulative_sum.last().unwrap();
    let first_non_zero = cumulative_sum.iter().find(|&&x| x != 0).unwrap_or(&-1);
    if sum == 0 || *first_non_zero == -1 || first_non_zero == &sum {
      debug!("talkslen: {}", talks.len());
      selected_index = rng.gen_range(0..talks.len());
    } else {
      // binsearch
      let r = rng.gen_range(*first_non_zero..sum);
      selected_index = binsearch_min(&cumulative_sum, r);
    }

    // increment bias without selection
    if is_consume {
      for i in 0..talks.len() {
        if i == selected_index {
          // 選ばれたトークの重みを0に
          self.reset(talks[i].clone());
        } else if bias_vec.iter().filter(|b| b == &&0).count() > talks.len() / 2 {
          // 全体の1/2が消費されるまで、それまでのトークが再び選ばれる可能性は生まれない
          self.increment(talks[i].clone());
        }
      }
    }

    selected_index
  }
}

// 二分探索をするが、同値がある場合は最小のインデックスを返す
fn binsearch_min(v: &Vec<i32>, r: i32) -> usize {
  let mut left = 0;
  let mut right = v.len() - 1;
  let mut mid = (left + right) / 2;
  while left < right {
    if v[mid] < r {
      left = mid + 1;
    } else {
      right = mid;
    }
    mid = (left + right) / 2;
  }
  mid
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_binsearch() {
    let v = vec![3, 3, 5, 10];
    assert_eq!(binsearch_min(&v, 3), 0);
  }

  #[test]
  fn test_talk_bias() {
    let mut bias = TalkBias::new();
    bias.reset("a".to_string());
    bias.reset("b".to_string());
    bias.reset("c".to_string());
    bias.reset("d".to_string());
    bias.reset("e".to_string());
    bias.reset("f".to_string());
    bias.reset("g".to_string());
    bias.reset("h".to_string());

    let talks: Vec<String> = vec!["a", "b", "c", "d", "e", "f", "g", "h"]
      .iter()
      .map(|s| s.to_string())
      .collect();

    let mut indexes: Vec<usize> = vec![];
    let mut select_count: Vec<i32> = vec![0; talks.len()];

    for _ in 0..10000 {
      let selected_index = bias.roulette(&talks, true);
      if let Some(last) = indexes.last() {
        if last == &selected_index {
          println!("duplication: {}", selected_index);
          let biases: Vec<i32> = talks.iter().map(|s| bias.get(&s)).collect();
          println!("biases: {:?}", biases);
        }
      };
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

    println!("indexes: {:?}", indexes);
    println!("select_count: {:?}", select_count);
  }
}
