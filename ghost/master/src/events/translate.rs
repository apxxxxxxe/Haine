use crate::variables::get_global_vars;
use regex::Regex;

pub fn on_translate(text: String) -> String {
  if text.is_empty() {
    return text;
  }

  let mut translated = text.clone();

  translated = text_only_translater(translated);

  let vars = get_global_vars();
  if vars.volatility.inserter.is_ready() {
    vars.volatility.inserter.run(translated)
  } else {
    translated
  }
}

// 参考：http://emily.shillest.net/ayaya/?cmd=read&page=Tips%2FOnTranslate%E3%81%AE%E4%BD%BF%E3%81%84%E6%96%B9&word=OnTranslate
fn text_only_translater(text: String) -> String {
  let re_tags = Regex::new(r"\\(\\|q\[.*?\]\[.*?\]|[!&8cfijmpqsn]\[.*?\]|[-*+1014567bcehntuvxz]|_[ablmsuvw]\[.*?\]|__(t|[qw]\[.*?\])|_[!?+nqsV]|[sipw][0-9])").unwrap();
  let tags = re_tags.find_iter(&text);
  let splitted = re_tags.split(&text).collect::<Vec<&str>>();
  let mut result = String::new();

  for (i, tag) in tags.enumerate() {
    result.push_str(translate_part(splitted[i].to_string()).as_str());
    result.push_str(&tag.as_str());
  }
  result.push_str(translate_part(splitted[splitted.len() - 1].to_string()).as_str());

  translate_whole(result)
}

fn translate_part(text: String) -> String {
  let surface_snippet = Regex::new(r"h([0-9]{7})").unwrap();

  let surface_replaced = surface_snippet.replace_all(&text, "\\0\\s[$1]").to_string();

  let replaces = vec![
    ("、", "、\\_w[600]", "φ", ""),
    ("。", "。\\_w[1200]", "φ", "」』"),
    ("！", "！\\_w[1200]", "φ", "」』"),
    ("？", "？\\_w[1200]", "φ", "」』"),
    ("…", "…\\_w[600]", "φ", ""),
    ("」", "」\\_w[600]", "φ", ""),
    ("』", "』\\_w[600]", "φ", ""),
    ("\\n\\n", "\\n\\n\\_w[700]", "φ", ""),
  ];
  let wait_replaced = replace_with_check(surface_replaced, replaces);
  let phi_replaced = wait_replaced.replace("φ", "");

  let vars = get_global_vars();
  let vars_replaced = phi_replaced.replace("{user_name}", &vars.user_name.clone().unwrap());

  vars_replaced
}

fn replace_with_check(text: String, replaces: Vec<(&str, &str, &str, &str)>) -> String {
  let mut translated = String::new();
  let text_chars_vec = text.char_indices().collect::<Vec<(usize, char)>>();
  debug!("iter: {:?}", text_chars_vec);
  let mut i = 0;
  loop {
    let (j, c) = match text_chars_vec.get(i) {
      Some((j, c)) => {
        debug!("{}: {}: {}", i, j, c);
        (j, c)
      }
      None => break,
    };
    if let Some(p) = replaces
      .iter()
      .position(|&(old, _, _, _)| text[*j..].starts_with(old))
    {
      let (old, new, prefix, suffix) = replaces[p];
      let mut has_suffix = false;
      if let Some(next) = text.chars().nth(i + old.chars().count()) {
        debug!("next: {}", next);
        if suffix.contains(next) {
          has_suffix = true;
        }
      }
      let mut has_prefix = false;
      if let Some(prev) = text.chars().nth(i - 1) {
        if prefix.contains(prev) {
          has_prefix = true;
        }
      }
      if !has_prefix && !has_suffix {
        translated.push_str(new);
      } else {
        translated.push_str(old);
      }
      i += old.chars().count();
    } else {
      translated.push(*c);
      i += 1;
    }
  }
  translated
}

fn translate_whole(text: String) -> String {
  let last_wait = Regex::new(r"\\_w\[([0-9]+)\]$").unwrap();
  let mut translated = text.clone();

  translated = last_wait.replace(&translated, "").to_string();

  translated
}

#[test]
fn test_translate() {
  let text = "こんにちは、\\n{user_name}さん。\\nお元気ですか。\\1ええ、私は元気です。\\nあなたはどうですか、ゴースト。".to_string();
  let translated = text_only_translater(text);
  println!("{}", translated);
}
