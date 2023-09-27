use crate::autolinefeed::Inserter;
use crate::variables::GlobalVariables;
use regex::Regex;

pub fn on_translate(text: String, vars: &mut GlobalVariables, inserter: &mut Inserter) -> String {
    if text.is_empty() {
        return text;
    }

    let mut translated = text.clone();

    translated = text_only_translater(translated, vars);

    if inserter.is_ready() {
        translated = inserter.run(translated);
    }
    translated
}

// 参考：http://emily.shillest.net/ayaya/?cmd=read&page=Tips%2FOnTranslate%E3%81%AE%E4%BD%BF%E3%81%84%E6%96%B9&word=OnTranslate
fn text_only_translater(text: String, vars: &mut GlobalVariables) -> String {
    let re_tags = Regex::new(r"\\(\\|q\[.*?\]\[.*?\]|[!&8cfijmpqsn]\[.*?\]|[-*+1014567bcehntuvxz]|_[ablmsuvw]\[.*?\]|__(t|[qw]\[.*?\])|_[!?+nqsV]|[sipw][0-9])").unwrap();
    let tags = re_tags.find_iter(&text);
    let splitted = re_tags.split(&text).collect::<Vec<&str>>();
    let mut result = String::new();

    for (i, tag) in tags.enumerate() {
        result.push_str(translate(splitted[i].to_string(), vars).as_str());
        result.push_str(&tag.as_str());
    }
    result.push_str(translate(splitted[splitted.len() - 1].to_string(), vars).as_str());

    result
}

fn translate(text: String, vars: &mut GlobalVariables) -> String {
    let surface_snippet = Regex::new(r"h([0-9]{7})").unwrap();

    let mut translated = text.clone();

    translated = surface_snippet
        .replace_all(&translated, "\\0\\s[$1]")
        .to_string();

    translated = translated.replace("、", "、\\_w[600]");
    translated = translated.replace("。", "。\\_w[1200]");
    translated = translated.replace("！", "！\\_w[1200]");
    translated = translated.replace("？", "？\\_w[1200]");
    translated = translated.replace("…", "…\\_w[600]");

    translated = translated.replace("{user_name}", &vars.user_name.clone().unwrap());

    translated
}
