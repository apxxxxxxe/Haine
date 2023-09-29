use md5::Digest;
use rand::Rng;
use std::collections::HashMap;

pub struct TalkBias(HashMap<Digest, i32>);

impl TalkBias {
    pub fn new() -> TalkBias {
        TalkBias(HashMap::new())
    }

    pub fn add(&mut self, digest: Digest, bias: i32) {
        self.0.insert(digest, bias);
    }

    pub fn get(&self, digest: &Digest) -> i32 {
        *self.0.get(digest).unwrap_or(&1)
    }

    fn increment(&mut self, digest: &Digest) {
        let mut bias = self.get(digest);
        bias <<= 1;
        if bias > 128 {
            bias = 128;
        }
        self.add(*digest, bias);
    }

    pub fn roulette(&mut self, talks: &Vec<String>) -> usize {
        let bias_vec: Vec<i32> = talks.iter().map(|s| self.get(&md5(s))).collect();

        let cumulative_sum: Vec<i32> = bias_vec
            .iter()
            .scan(0, |acc, &x| {
                *acc += x;
                Some(*acc)
            })
            .collect();
        println!("cumulative_sum: {:?}", cumulative_sum);

        let mut rng = rand::thread_rng();
        let r = rng.gen_range(0..*cumulative_sum.last().unwrap_or(&0));

        // binsearch
        let selected_index = cumulative_sum.binary_search(&r).unwrap_or_else(|i| i);

        // increment bias without selection
        for i in 0..talks.len() {
            if i == selected_index {
                self.add(md5(&talks[i]), 1);
            } else {
                self.increment(&md5(&talks[i]));
            }
        }

        selected_index
    }
}

fn md5(s: &str) -> Digest {
    md5::compute(s.as_bytes())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_talk_bias() {
        let mut bias = TalkBias::new();
        bias.add(md5("a"), 1);
        bias.add(md5("b"), 1);
        bias.add(md5("c"), 1);
        bias.add(md5("d"), 1);
        bias.add(md5("e"), 1);
        bias.add(md5("f"), 1);
        bias.add(md5("g"), 1);
        bias.add(md5("h"), 1);

        let talks: Vec<String> = vec!["a", "b", "c", "d", "e", "f", "g", "h"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        let mut indexes: Vec<usize> = vec![];
        let mut select_count: Vec<i32> = vec![0; talks.len()];

        for _ in 0..100 {
            println!("bias: {:?}", bias.0);
            let selected_index = bias.roulette(&talks);
            indexes.push(selected_index);
            select_count[selected_index] += 1;
            assert!(selected_index < talks.len());
        }

        println!("indexes: {:?}", indexes);
        println!("select_count: {:?}", select_count);
    }
}
