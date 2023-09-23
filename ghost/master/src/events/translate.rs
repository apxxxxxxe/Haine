use regex::Regex;

pub fn on_translate(text: String) -> String {
    if text.is_empty() {
        return text;
    }

    let mut translated = text.clone();

    let surface_snippet = Regex::new(r"h([0-9]{7})").unwrap();
    translated = surface_snippet.replace_all(&translated, "\\0\\s[$1]").to_string();

    translated = translated.replace("、", "、\\_w[600]");
    translated = translated.replace("。", "。\\_w[1200]");
    translated = translated.replace("！", "！\\_w[1200]");
    translated = translated.replace("？", "？\\_w[1200]");
    translated = translated.replace("…", "…\\_w[600]");

    translated
}
