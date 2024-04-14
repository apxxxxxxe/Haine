use crate::events::common::*;
use crate::events::menu::on_menu_exec;
use crate::variables::{get_global_vars, EventFlag, GlobalVariables, TouchInfo};
use once_cell::sync::Lazy;
use shiorust::message::{Parser, Request, Response};

pub fn new_mouse_response(info: String) -> Response {
  let vars = get_global_vars();
  let dummy_req = Request::parse(DUMMY_REQUEST).unwrap();
  if info != *vars.volatility.last_touch_info() {
    vars.volatility.set_touch_count(0);
  }
  vars.volatility.set_last_touch_info(info.clone());
  vars
    .volatility
    .set_touch_count(vars.volatility.touch_count() + 1);

  match info.as_str() {
    "0doubleclick" => {
      return on_menu_exec(&dummy_req);
    }
    "0headdoubleclick" => {
      if !get_global_vars()
        .flags()
        .check(&EventFlag::FirstHitTalkDone)
      {
        return on_menu_exec(&dummy_req);
      }
    }
    _ => (),
  }

  match mouse_dialogs(info, vars) {
    Some(dialogs) => new_response_with_value(
      dialogs[choose_one(&dialogs, true).unwrap()].clone(),
      TranslateOption::with_shadow_completion(),
    ),
    None => new_response_nocontent(),
  }
}

static DIALOG_SEXIAL_FIRST: Lazy<Vec<String>> =
  Lazy::new(|| vec!["h1111205……会って早々、これ？\nなんというか……h1111204流石ね。".to_string()]);

static DIALOG_SEXIAL_SCOLD: Lazy<Vec<String>> = Lazy::new(|| {
  vec![
      "h1111202……いくら他人の目がないとはいえ、h1111204品性を疑うわ。".to_string(),
      "h1111205これがあなたのやりたいこと？h1111204くだらないのね。".to_string(),
      "h1111205スキンシップにしてはセンスが無いと思うわ。".to_string(),
      "h1111210情熱的という人もいるでしょうし、\\n野蛮で下劣という人もいるでしょうね。\\n\\nh1111204私は後者よ、お猿さん。".to_string(),
    ]
});

static DIALOG_SEXIAL_AKIRE: Lazy<Vec<String>> = Lazy::new(|| {
  vec![
    "h1111201さっきからずいぶん必死ね。\\nh1111304ばかみたいな顔してるわよ。".to_string(),
    "h1111304面白い顔。h1111310鏡で見せてあげたいわ。".to_string(),
    "h1111104悪戯がすぎるわよ。".to_string(),
    "h1111103はあ……h1111106何が楽しいんだか。".to_string(),
    "h1111204その熱意は買うけれど。……h1111210虚しくないの？".to_string(),
    "h1111204…………退屈。".to_string(),
  ]
});

fn first_and_other(
  touch_info: &mut TouchInfo,
  first: Vec<String>,
  other: Vec<String>,
) -> Vec<String> {
  let result = if touch_info.is_reset() { first } else { other };
  touch_info.add();
  result
}

pub fn mouse_dialogs(info: String, vars: &mut GlobalVariables) -> Option<Vec<String>> {
  match info.as_str() {
    "0headdoubleclick" => Some(head_hit(vars)),
    "0handnade" => Some(first_and_other(
      &mut vars.volatility.touch_info_mut().hand,
      vec![],
      vec![
        "\
        h1111205\\1触れた手の感触はゼリーを掴むような頼りなさだった。\
        ……手が冷えるわよ。h1111204興味があるのは分かるけど、ほどほどにね。\
        "
        .to_string(),
        "\
        h1111205あなたが何を伝えたいのかは、なんとなく分かるけれど。\\n\
        ……それは不毛というものよ。\
        "
        .to_string(),
        "\
        h1111205\\1彼女の指は長い。
        h1111210……うん。\\n\
        "
        .to_string(),
      ],
    )),
    "0bustnade" => Some(bust_touch(vars)),
    "0bustdoubleclick" => Some(bust_touch(vars)),
    "0skirtup" => {
      let mut conbo_parts: Vec<Vec<String>> =
        vec![vec!["h2244402……！\\nh1241102\\_w[500]".to_string()]];
      if !vars.volatility.first_sexial_touch() && vars.volatility.ghost_up_time() < 30 {
        vars.volatility.set_first_sexial_touch(true);
        conbo_parts.push(DIALOG_SEXIAL_FIRST.clone());
      } else {
        conbo_parts.push(vec![
          "h1111204……いいもの見たって顔してる。h1111210屈辱だわ。".to_string(),
          "h1111205……ああ、ひどい人。h1111210泣いてしまいそうだわ。".to_string(),
          "h1111304……悪餓鬼。".to_string(),
        ]);
      }
      let zero_skirt_up: Vec<String> = all_combo(&conbo_parts);
      Some(zero_skirt_up)
    }
    "0shoulderdown" => Some(first_and_other(
      &mut vars.volatility.touch_info_mut().shoulder,
      vec!["\
      h1141601φ！\\_w[250]h1000000\\_w[1200]\\n\
      ……h1111206あまりスキンシップは好きじゃないのだけど。\\n\
      "
      .to_string()],
      vec![
        "\
      h1111101\\1抱き寄せようとすると、腕は彼女をすり抜けた。\
      h1111101……h1111204私はあなたのものじゃないのよ。\\n\
      "
        .to_string(),
        "\
      h1111205\\1背の高い彼女の肩に手をかけると、柔らかい髪が指に触れた。\
      h1111204……それで？h1111210あなたは私をどうしたいのかしら。\
      "
        .to_string(),
      ],
    )),
    _ => None,
  }
}

fn bust_touch(vars: &mut GlobalVariables) -> Vec<String> {
  let zero_bust_touch_threshold = 12;
  let mut zero_bust_touch = Vec::new();
  if !vars.volatility.first_sexial_touch()
    && vars.volatility.ghost_up_time() < 30
    && vars.flags().check(&EventFlag::FirstClose)
  {
    vars.volatility.set_first_sexial_touch(true);
    zero_bust_touch.extend(DIALOG_SEXIAL_FIRST.clone());
  } else if vars.volatility.touch_count() < zero_bust_touch_threshold / 3 {
    zero_bust_touch.extend(vec![
      "h1111205……ずいぶん嬉しそうだけれど、h1111204そんなにいいものなのかしら？".to_string(),
      "h1111210気を引きたいだけなら、もっと賢い方法があると思うわ。".to_string(),
      "h1111204……あなたは、私をそういう対象として見ているの？".to_string(),
      "h1111205気安いのね。あまり好きではないわ。".to_string(),
      "h1111304媚びた反応を期待してるの？\\nh1112204この身体にそれを求められても、ね。".to_string(),
    ]);
  } else if vars.volatility.touch_count() < zero_bust_touch_threshold / 3 * 2 {
    zero_bust_touch.extend(DIALOG_SEXIAL_SCOLD.clone());
  } else if vars.volatility.touch_count() < zero_bust_touch_threshold {
    zero_bust_touch.extend(DIALOG_SEXIAL_AKIRE.clone());
  } else if vars.volatility.touch_count() == zero_bust_touch_threshold {
    zero_bust_touch.push(
      "\
      h1111205\\1触れようとした手先が、霧に溶けた。\\n\
      慌てて引っ込めると、手は元通りになった。\
      h1111201許されていると思ったの？\\n\
      h1111304残念だけど、それほど気は長くないの。\\n\
      h1111310わきまえなさい。"
        .to_string(),
    );
  } else {
    zero_bust_touch.push("h1111204\\1自重しよう……。".to_string());
  }
  zero_bust_touch
}

pub fn on_head_hit(_req: &Request) -> Response {
  to_aroused();
  get_global_vars()
    .flags_mut()
    .done(EventFlag::FirstHitTalkDone);
  let m = "\\t\\*\
    h1111201あら、話す気に……h1121414っ！？\\n\
    \\1半ば衝動的に、彼女を突き飛ばした。\\n\
    h1112101\\1それは嫌悪からだ。\\n\
    私を受け入れるようなそぶりを見せながら、\\n\
    同時に私を助けないと嘯く傲慢さ。\\n\
    そして何よりも、理性的な物言いをしておきながら一欠片も倫理の匂いを感じさせない態度に、毒虫にも似た嫌悪感を感じたのだ。\\n\
    \\0……。\\1叩かれたハイネは呆けたように私を見つめ…………h1222804\\1笑った。\\n\
    \\0……殴られるなんて、ずいぶん久しぶりだわ。\\n\
    h1222310ウフ、あなたのせいで思い出しちゃった。\\n\
    痛いってこういうものだったわね。\\n\
    h1222506アハハ、素敵だわ、とても。\\n\
    h1222511ねえ、もっとやってみて。\\n\
    嫌悪したから叩いたのでしょう？あなた。\\n\
    双方に得があるのよ、遠慮はいらないわ。\\n\
    さあ。h1322813さあ！\\n\
    "
    .to_string();
  new_response_with_value(m, TranslateOption::simple_translate())
}

pub fn head_hit(vars: &mut GlobalVariables) -> Vec<String> {
  let is_aroused = vars.volatility.aroused();
  to_aroused();
  if !is_aroused {
    vec!["h1121414痛っ……\\nh1311204あら、その気になってくれた？".to_string()]
  } else {
    all_combo(&vec![
      ["h1221410ぐっ……", "h1221411痛っ……", "h1221714づっ……"]
        .iter()
        .map(|s| s.to_string())
        .collect(),
      vec!["\\n".to_string()],
      [
        "h1311204すてき。h1311207もっとして。",
        "h1111206ああ、星が舞っているわ。h1311215痛くて苦しくて、死んでしまいそう。",
        "h1111204ひどい。h1111210ひどいわ。\\nh1311513癖になってしまったらどうするの？",
      ]
      .iter()
      .map(|s| s.to_string())
      .collect(),
    ])
  }
}

const DUMMY_REQUEST: &str = "GET SHIORI/3.0\r\n\
Charset: UTF-8\r\n\
Sender: SSP\r\n\
SenderType: internal,raise\r\n\
SecurityLevel: local\r\n\
Status: choosing,balloon(0=0)\r\n\
ID: OnFirstBoot\r\n\
BaseID: OnBoot\r\n\
Reference0: 1\r\n\r\n";
