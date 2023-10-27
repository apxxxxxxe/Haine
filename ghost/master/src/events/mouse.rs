use crate::events::common::*;
use crate::variables::GlobalVariables;
use once_cell::sync::Lazy;

static DIALOG_SEXIAL_FIRST: Lazy<Vec<String>> =
    Lazy::new(|| vec!["h1111205……会って早々、これ？\nなんというか……h1111204流石ね。".to_string()]);

static DIALOG_SEXIAL_SCOLD: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
      "h1111202……いくら他人の目がないとはいえ、h1111204品性を疑うわ。".to_string(),
      "h1111205これがあなたのやりたいこと？h1111204くだらないのね。".to_string(),
      "h1111205スキンシップにしてはセンスが無いと思うわ。".to_string(),
      "h1111209情熱的という人もいるでしょうし、\\n野蛮で下劣という人もいるでしょうね。\\n\\nh1111204私は後者よ、お猿さん。".to_string(),
    ]
});

static DIALOG_SEXIAL_AKIRE: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        "h1111201さっきからずいぶん必死ね。\\nh1111304ばかみたいな顔してるわよ。".to_string(),
        "h1111304面白い顔。h1111309鏡で見せてあげたいわ。".to_string(),
        "h1111104悪戯がすぎるわよ。".to_string(),
        "h1111103はあ……h1111106何が楽しいんだか。".to_string(),
        "h1111204その熱意は買うけれど。……h1111209虚しくないの？".to_string(),
        "h1111204……………………退屈。".to_string(),
    ]
});

pub fn mouse_dialogs(info: String, vars: &mut GlobalVariables) -> Option<Vec<String>> {
    match info.as_str() {
        "0bustnade" => {
            let zero_bust_nade_threshold = 12;
            let mut zero_bust_nade = Vec::new();
            if !vars.volatility.first_sexial_touch && vars.volatility.ghost_up_time < 30 {
                vars.volatility.first_sexial_touch = true;
                zero_bust_nade.extend(DIALOG_SEXIAL_FIRST.clone());
            } else if vars.volatility.touch_count < zero_bust_nade_threshold / 3 {
                zero_bust_nade.extend(vec![
                "h1111205胸、か。h1111204ずいぶん嬉しそうだけれど、そんなにいいものなのかしら？"
                    .to_string(),
                "h1111209気を引きたいだけなら、もっと賢い方法があると思うわ。".to_string(),
                "h1111204……あなたは、私をそういう対象として見ているの？".to_string(),
                "h1111205気安いのね。あまり好きではないわ。".to_string(),
            ]);
            } else if vars.volatility.touch_count < zero_bust_nade_threshold / 3 * 2 {
                zero_bust_nade.extend(DIALOG_SEXIAL_SCOLD.clone());
            } else if vars.volatility.touch_count < zero_bust_nade_threshold {
                zero_bust_nade.extend(DIALOG_SEXIAL_AKIRE.clone());
            } else if vars.volatility.touch_count == zero_bust_nade_threshold {
                zero_bust_nade.extend(all_combo(&vec![
            vec!["h1111205\\1触れようとした手先が、霧に溶けた。\\n慌てて引っ込めると、手は元通りになった。h1111201許されていると思ったの？\\n".to_string()],
            vec![
                "h1111304残念だけど、それほど気は長くないの。".to_string(),
                "h1111204わきまえなさい。".to_string(),
            ],
        ]));
            } else {
                zero_bust_nade.extend(vec!["h1111204\\1自重しよう……。".to_string()]);
            }
            return Some(zero_bust_nade);
        }
        "0skirtup" => {
            let mut conbo_parts: Vec<Vec<String>> =
                vec![vec!["h2244402……！\\nh1241102\\_w[500]".to_string()]];
            if !vars.volatility.first_sexial_touch && vars.volatility.ghost_up_time < 30 {
                vars.volatility.first_sexial_touch = true;
                conbo_parts.push(DIALOG_SEXIAL_FIRST.clone());
            } else {
                conbo_parts.push(vec![
                    "h1111204……いいもの見たって顔してる。h1111209屈辱だわ。".to_string(),
                    "h1111205……ああ、ひどい人。h1111209泣いてしまいそうだわ。".to_string(),
                    "h1111204……悪餓鬼。".to_string(),
                ]);
            }
            let zero_skirt_up: Vec<String> = all_combo(&conbo_parts);
            return Some(zero_skirt_up);
        }
        "0shoulderdown" => {
            return Some(vec!["抱き寄せる".to_string()]);
        }
        _ => None,
    }
}
