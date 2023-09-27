use regex::Regex;
use std::sync::{Arc, Mutex};
use vibrato::Tokenizer;

pub struct Inserter {
    cols_num: f32,
    pub tokenizer: Arc<Mutex<Option<Tokenizer>>>,
}

impl Inserter {
    pub fn new(cols_num: f32) -> Self {
        Inserter {
            cols_num,
            tokenizer: Arc::new(Mutex::new(None)),
        }
    }

    pub fn is_ready(&mut self) -> bool {
        let tokenizer_clone = self.tokenizer.clone();
        let tokenizer = tokenizer_clone.lock().unwrap();
        tokenizer.is_some()
    }

    pub fn set_tokenizer(&mut self, tokenizer: Tokenizer) {
        let t = Arc::new(Mutex::new(Some(tokenizer)));
        self.tokenizer = t;
    }

    pub fn default() -> Self {
        Self::new(24.0)
    }

    pub fn run(&mut self, src: String) -> String {
        let parts = self.wakachi(src);
        self.render(parts)
    }

    fn wakachi(&mut self, src: String) -> Vec<String> {
        println!("wakachi");
        let tokenizer_clone = self.tokenizer.clone();
        let tokenizer = tokenizer_clone.lock().unwrap();
        let t = tokenizer.as_ref().unwrap();
        let mut worker = t.new_worker();
        let mut text = src.clone();
        let mut _word_counts = vec![0, 0];
        let hinshi_target = vec![
            "名詞",
            "動詞",
            "接頭詞",
            "副詞",
            "感動詞",
            "形容詞",
            "形容動詞",
            "連体詞",
        ];

        let mut results = vec!["".to_string()];
        let mut result = "".to_string();
        let delim_re = "\x1f$1";
        let delim = "\x1f";
        let line_splitter = Regex::new(r"(\\n|\\_l\[0|\\x|\\c|\\[01]|\\p\[\d+\])").unwrap();
        text = line_splitter.replace_all(&text, delim_re).to_string();
        let lines = text
            .split(delim)
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        for line in lines {
            println!("{}", line);
            let sakura_script_re = Regex::new(
                r###"\\_{0,2}[a-zA-Z0-9*!&](\d|\[("([^"]|\\")+?"|([^\]]|\\\])+?)+?\])?"###,
            )
            .unwrap();

            let mut in_brackets = false;
            let mut last_in_brackets = false;
            let mut after_open_bracket = false;
            let mut after_close_bracket = false;
            let mut after_pre_pos = false;
            let mut after_sahen_noun = false;

            let mut sakura_scripts = sakura_script_re.find_iter(&line);
            let ss_splitted = sakura_script_re.split(&line).collect::<Vec<&str>>();
            for pieces in ss_splitted {
                worker.reset_sentence(pieces);
                worker.tokenize();
                for token in worker.token_iter() {
                    let info: Vec<&str> = token.feature().split(',').collect();
                    let pos = info[0];
                    let pos_detail = info[1];
                    let pos_katsuyou = info[4];

                    let mut no_break = hinshi_target.iter().find(|&&p| p == pos) == None
                        || pos_detail.find("接尾") != None
                        || (pos == "動詞" && pos_detail.find("サ変接続") != None)
                        || pos_detail.find("非自立") != None
                        || after_pre_pos
                        || (after_sahen_noun && pos_detail.find("サ変動詞") != None)
                        || after_open_bracket
                        || pos_detail.find("括弧閉") != None
                        || pos_detail.find("ナイ形容詞語幹") != None
                        || pos_katsuyou.find("特殊・ナイ") != None
                        || pos_katsuyou.find("特殊・タ") != None;

                    if after_close_bracket || pos_detail.find("括弧開") != None {
                        no_break = false;
                    }

                    if !no_break {
                        results.push(result);
                        result = "".to_string();
                    }

                    result += token.surface();

                    after_close_bracket = in_brackets != last_in_brackets && !in_brackets;
                    after_pre_pos = pos == "接頭詞";
                    after_sahen_noun = pos_detail.find("サ変接続") != None;
                    after_open_bracket = pos_detail.find("括弧開") != None;
                }
                if let Some(s) = sakura_scripts.nth(0) {
                    println!("sakura_script: {}", s.as_str());
                    result += s.as_str();
                }
            }
            results.push(result);
            result = "".to_string();
        }
        results
    }

    fn render(&mut self, parts: Vec<String>) -> String {
        println!("rendering");
        let re_periods = Regex::new(r"[、。！？]").unwrap();
        let re_change_scope = Regex::new(r"(\\[01][^w]?|\\p\[\d+\])").unwrap();
        let re_not_number = Regex::new(r"[^\d]").unwrap();
        let re_change_line = Regex::new(r"(\\n|\\_l\[0[,0-9em%]+\]|\\x|\\c)").unwrap();
        let mut result = String::new();
        let mut counts = vec![0.0, 0.0];
        let mut i = 0;
        let mut scope: usize = 0;
        loop {
            if i >= parts.len() {
                break;
            }
            let part = &parts[i];
            let c = self.count(part.to_string());

            if re_change_scope.is_match(part) {
                let c = re_change_scope.captures(part).unwrap()[0].to_string();
                scope = re_not_number.replace_all(&c, "").parse::<usize>().unwrap();
            }

            if re_change_line.is_match(part) {
                counts[scope] = 0.0;
            }

            println!(
                "count: {}, c: {}, cols_num: {}",
                counts[scope], c, self.cols_num
            );
            if c > self.cols_num {
                result.push_str(part);
                counts[scope] += c % self.cols_num;
                counts[scope] %= self.cols_num;
                continue;
            }
            if counts[scope] + c > self.cols_num {
                result.push_str("\\n");
                counts[scope] = 0.0;
                println!("continue: {}", counts[scope] + c);
                continue;
            }
            counts[scope] += c;
            println!("ok: {}", i);
            result.push_str(part);

            // 句読点後の文章が1行に収まるなら、一気に出力して次へ
            if re_periods.is_match(part) {
                let mut j = i + 1;
                let mut next_line = String::new();
                while j < parts.len() {
                    let next = &parts[j];
                    if re_change_scope.is_match(next) || re_change_line.is_match(next) {
                        j -= 1;
                        break;
                    }
                    next_line.push_str(next);
                    j += 1;
                }

                let next_word_count = self.count(next_line.clone());
                if next_word_count <= self.cols_num {
                    // 句読点の後に改行を入れるのは、句読点の後の文章が1行に収まらない場合のみ
                    if counts[scope] + next_word_count > self.cols_num {
                        result.push_str("\\n");
                        counts[scope] = 0.0;
                    }
                    result.push_str(&next_line);
                    counts[scope] += next_word_count;
                    i = j;
                }
            }
            i += 1;
        }
        result
    }

    fn count(&self, text: String) -> f32 {
        let sakura_script_re =
            Regex::new(r###"\\_{0,2}[a-zA-Z0-9*!&](\d|\[("([^"]|\\")+?"|([^\]]|\\\])+?)+?\])?"###)
                .unwrap();
        let removed = sakura_script_re.replace_all(&text, "");
        let mut count = 0.0;
        count += removed.chars().filter(|c| c.is_ascii()).count() as f32 * 0.5;
        count += removed.chars().filter(|c| !c.is_ascii()).count() as f32;
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserter() {
        let text = "\
            \\s[1111105]人生に変化は付きもの……けれど、\\s[1111109]停滞はそれ以上。\\n\
            一度立ち止まってしまうと、空気は一瞬で淀んで、身動きがとれなくなってしまうのよ。\
            あなたも経験したこと、あるんじゃないかしら。"
            .to_string();
        let mut ins = Inserter::default();
        println!("{}", ins.run(text));
    }
}
