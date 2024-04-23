use crate::events::aitalk::FIRST_RANDOMTALKS;
use crate::events::common::*;
use crate::events::menu::on_menu_exec;
use crate::variables::{get_global_vars, EventFlag, GlobalVariables, TouchInfo};
use once_cell::sync::Lazy;
use shiorust::message::{Parser, Request, Response};

#[macro_export]
macro_rules! get_touch_info {
  ($info:expr) => {
    get_global_vars()
      .volatility
      .touch_info_mut()
      .entry($info.to_string())
      .or_insert($crate::variables::TouchInfo::new())
  };
}

pub fn new_mouse_response(info: String) -> Response {
  let vars = get_global_vars();
  let last_touch_info = vars.volatility.last_touch_info();

  // 同一に扱う
  let i = if info == "0bustdoubleclick" {
    "0bustnade".to_string()
  } else {
    info
  };

  if i != last_touch_info.as_str() {
    if let Some(touch_info) = vars
      .volatility
      .touch_info_mut()
      .get_mut(last_touch_info.as_str())
    {
      touch_info.reset();
    }
    vars.volatility.set_last_touch_info(i.clone());
  }
  if i.as_str() == "0doubleclick"
    || i.as_str() == "0headdoubleclick"
      && !get_global_vars()
        .flags()
        .check(&EventFlag::FirstRandomTalkDone(
          FIRST_RANDOMTALKS.len() as u32 - 1,
        ))
  {
    let dummy_req = Request::parse(DUMMY_REQUEST).unwrap();
    return on_menu_exec(&dummy_req);
  }

  let response = match mouse_dialogs(i.clone(), vars) {
    Some(dialogs) => new_response_with_value(
      format!(
        "{}{}",
        REMOVE_BALLOON_NUM,
        dialogs[choose_one(&dialogs, true).unwrap()].clone()
      ),
      TranslateOption::with_shadow_completion(),
    ),
    None => new_response_nocontent(),
  };

  // 一括で回数を増やす
  vars
    .volatility
    .touch_info_mut()
    .entry(i)
    .or_insert(TouchInfo::new())
    .add();

  response
}

static DIALOG_SEXIAL_WHILE_HITTING: Lazy<Vec<String>> = Lazy::new(|| {
  vec!["h1221204ねえ、焦らさないで。\\nもっと叩いて。\\n死を、感じさせて。".to_string()]
});

static DIALOG_TOUCH_WHILE_HITTING: Lazy<Vec<String>> =
  Lazy::new(|| vec!["h1211104優しくしないで。退屈になるじゃない。".to_string()]);

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

pub fn mouse_dialogs(info: String, vars: &mut GlobalVariables) -> Option<Vec<String>> {
  let touch_count = get_touch_info!(info.as_str()).count();
  match info.as_str() {
    "0headdoubleclick" => Some(head_hit_dialog(touch_count, vars)),
    "0handnade" => Some(zero_hand_nade(touch_count, vars)),
    "0bustnade" => Some(zero_bust_touch(touch_count, vars)),
    "0skirtup" => Some(zero_skirt_up(touch_count, vars)),
    "0shoulderdown" => Some(zero_shoulder_down(touch_count, vars)),
    _ => None,
  }
}

fn zero_hand_nade(count: u32, vars: &mut GlobalVariables) -> Vec<String> {
  if vars.volatility.aroused() {
    DIALOG_TOUCH_WHILE_HITTING.clone()
  } else {
    let dialogs = vec![vec![
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
    ]];
    phased_talks(count, dialogs).0
  }
}

fn zero_skirt_up(_count: u32, vars: &mut GlobalVariables) -> Vec<String> {
  if vars.volatility.aroused() {
    DIALOG_SEXIAL_WHILE_HITTING.clone()
  } else {
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
    all_combo(&conbo_parts)
  }
}

fn zero_shoulder_down(count: u32, _vars: &mut GlobalVariables) -> Vec<String> {
  let dialogs = vec![
    vec![
      "\
      h1141601φ！\\_w[250]h1000000\\_w[1200]\\n\
      ……h1111206あまりスキンシップは好きじゃないのだけど。\\n\
      "
      .to_string(),
      "hogefuga".to_string(),
      "minute".to_string(),
    ],
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
  ];
  phased_talks(count, dialogs).0
}

fn zero_bust_touch(count: u32, vars: &mut GlobalVariables) -> Vec<String> {
  if vars.volatility.aroused() {
    DIALOG_TOUCH_WHILE_HITTING.clone()
  } else {
    let zero_bust_touch_threshold = 12;
    let mut zero_bust_touch = Vec::new();
    if !vars.volatility.first_sexial_touch()
      && vars.volatility.ghost_up_time() < 30
      && vars.flags().check(&EventFlag::FirstClose)
    {
      vars.volatility.set_first_sexial_touch(true);
      zero_bust_touch.extend(DIALOG_SEXIAL_FIRST.clone());
    } else if count < zero_bust_touch_threshold / 3 {
      zero_bust_touch.extend(vec![
        "h1111205……ずいぶん嬉しそうだけれど、h1111204そんなにいいものなのかしら？".to_string(),
        "h1111210気を引きたいだけなら、もっと賢い方法があると思うわ。".to_string(),
        "h1111204……あなたは、私をそういう対象として見ているの？".to_string(),
        "h1111205気安いのね。あまり好きではないわ。".to_string(),
        "h1111304媚びた反応を期待してるの？\\nh1112204この身体にそれを求められても、ね。"
          .to_string(),
      ]);
    } else if count < zero_bust_touch_threshold / 3 * 2 {
      zero_bust_touch.extend(DIALOG_SEXIAL_SCOLD.clone());
    } else if count < zero_bust_touch_threshold {
      zero_bust_touch.extend(DIALOG_SEXIAL_AKIRE.clone());
    } else if count == zero_bust_touch_threshold {
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
}

pub fn on_head_hit_cancel(_req: &Request) -> Response {
  let m = "\\1……踏みとどまった。".to_string();
  new_response_with_value(m, TranslateOption::simple_translate())
}

pub fn on_head_hit(_req: &Request) -> Response {
  to_aroused();
  let m = "\\t\\*\
    h1111201あら、話す気に……h1121414っ！？\\n\
    \\1半ば衝動的に、彼女を突き飛ばした。\\n\
    h1112401\\1それは嫌悪からだ。\\n\
    私を受け入れるようなそぶりを見せながら、\\n\
    同時に私を助けないと嘯く傲慢さ。\\n\
    そして何よりも、理性的な物言いをしておきながら一欠片も倫理の匂いを感じさせない態度に、毒虫にも似た嫌悪感を感じたのだ。\\n\
    \\0……。\\1立ち上がったハイネは呆けたように私を見つめ…………h1222804\\1笑った。\\n\
    \\0……殴られるなんて、ずいぶん久しぶりだわ。\\n\
    h1222310ウフ、あなたのせいで思い出しちゃった。\\n\
    痛いってこういうものだったわね。\\n\
    h1222506アハハ、素敵だわ、とても。\\n\
    h1222204ねえ、もっとやってみて。\\n\
    嫌悪したから突き飛ばしたのでしょう？あなた。\\n\
    双方に得があるのよ、遠慮はいらないわ。\\n\
    さあ。h1322813さあ！\\n\
    "
    .to_string();
  new_response_with_value(m, TranslateOption::simple_translate())
}

pub fn head_hit_dialog(count: u32, vars: &mut GlobalVariables) -> Vec<String> {
  let is_aroused = vars.volatility.aroused();
  to_aroused();
  if !vars.flags().check(&EventFlag::FirstHitTalkStart) {
    vars.flags_mut().done(EventFlag::FirstHitTalkStart);
    get_touch_info!("0headdoubleclick").reset(); // 選択肢だけなのでカウントしない
    vec!["\
    \\s[1111101]\\![*]\\q[突き飛ばす,OnHeadHit]\\n\\![*]\\q[やめておく,OnHeadHitCancel]\
    "
    .to_string()]
  } else if !is_aroused {
    get_touch_info!("0headdoubleclick").reset(); // 初回はカウントしない
    vec!["h1121414痛っ……\\nh1311204あら、その気になってくれた？".to_string()]
  } else {
    // 各段階ごとのセリフ
    let suffixes_list = vec![
      vec![
        "h1311204すてき。h1311207もっとして。".to_string(),
        "h1311206ああ、星が舞っているわ。\\nh1311215痛くて苦しくて、死んでしまいそう。".to_string(),
        "h1221104ひどい。h1221110ひどいわ。\\nh1322513癖になってしまったらどうするの？".to_string(),
      ],
      vec![
        "h1311204あは、ずきずきする。\\nh1311307血は通っていないはずなのに、脈打ってる。"
          .to_string(),
        "h1311104ああ、痛い。h1321407痛いのがいいの。".to_string(),
        "h1321104目の奥が痛むわ。\\nh1321207容赦がないの、好きよ。".to_string(),
      ],
      vec![
        "h1311308あぁ……こんなに幸せでいいのかしら。".to_string(),
        "h1321304だめな、だめなことなのに、\\n求める気持ちが止まらないの。".to_string(),
        "h1321408んう……h1322304もう少し、もう少しなの。".to_string(),
      ],
      vec![
        "h1311308……ん、ぐっ".to_string(),
        "h1321208……あは、あは、は……".to_string(),
        "h1321808ひゅー、ひゅ……んん、あ、は……".to_string(),
      ],
      vec!["\\t\\*\
      h1322808っ、～～h1000000～～～……！\\n\
      \\1ひときわ大きく震えて、彼女はへたりこんだ。\\n\
      \\0………………。\\n\
      うふ、ふふふ。\\n\
      h1211308よかったわ、とても。\\n\
      \\n\
      h1211310………………h1111310ふー。\\n\
      h1111205\\1……落ち着いたようだ。\\n\
      "
      .to_string()],
    ];

    let (suffixes, is_last) = phased_talks(count, suffixes_list);

    if is_last {
      vars.volatility.set_aroused(false);
      get_touch_info!("0headdoubleclick").reset();
      return suffixes;
    }

    let prefixes = [
      "h1221410ぐっ……\\n".to_string(),
      "h1221714づっ……\\n".to_string(),
      "h1221710うぅっ……\\n".to_string(),
    ];
    let mut result = Vec::new();
    for j in 0..suffixes.len() {
      result.push(format!("{}{}", prefixes[j % prefixes.len()], suffixes[j]));
    }
    result
  }
}

pub fn phased_talks(count: u32, phased_talk_list: Vec<Vec<String>>) -> (Vec<String>, bool) {
  let dialog_lengthes = phased_talk_list
    .iter()
    .map(|x| x.len() as u32)
    .collect::<Vec<u32>>();
  let dialog_cumsum = dialog_lengthes
    .iter()
    .scan(0, |sum, x| {
      *sum += x;
      Some(*sum)
    })
    .collect::<Vec<u32>>();

  for i in 0..dialog_cumsum.len() - 1 {
    if count < dialog_cumsum[i] {
      return (phased_talk_list[i].clone(), false);
    }
  }
  (phased_talk_list.last().unwrap().to_owned(), true)
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
