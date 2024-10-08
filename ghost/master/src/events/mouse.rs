use crate::check_error;
use crate::error::ShioriError;
use crate::events::common::*;
use crate::events::first_boot::FIRST_RANDOMTALKS;
use crate::events::menu::on_menu_exec;
use crate::events::on_ai_talk;
use crate::events::randomtalk::moving_to_library_talk;
use crate::events::randomtalk::moving_to_living_room_talk;
use crate::events::render_immersive_icon;
use crate::events::TalkingPlace;
use crate::events::IMMERSIVE_ICON_COUNT;
use crate::events::IMMERSIVE_RATE_MAX;
use crate::sound::play_sound;
use crate::status::Status;
use crate::variables::{get_global_vars, EventFlag, GlobalVariables, TouchInfo};
use once_cell::sync::Lazy;
use shiorust::message::{Parser, Request, Response};

const SOUND_LIGHT_CANDLE: &str = "マッチで火をつける.mp3";
const SOUND_BLOW_CANDLE: &str = "マッチの火を吹き消す.mp3";

pub enum BodyPart {
  Head,
  Face,
  Mouth,
  Bust,
  Shoulder,
  Skirt,
  Hand,
}

impl BodyPart {
  pub fn from_str(s: &str) -> Option<Self> {
    match s {
      "head" => Some(BodyPart::Head),
      "face" => Some(BodyPart::Face),
      "mouth" => Some(BodyPart::Mouth),
      "bust" => Some(BodyPart::Bust),
      "shoulder" => Some(BodyPart::Shoulder),
      "skirt" => Some(BodyPart::Skirt),
      "hand" => Some(BodyPart::Hand),
      _ => None,
    }
  }
}

impl std::fmt::Display for BodyPart {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      BodyPart::Head => write!(f, "頭"),
      BodyPart::Face => write!(f, "顔"),
      BodyPart::Mouth => write!(f, "口"),
      BodyPart::Bust => write!(f, "胸"),
      BodyPart::Shoulder => write!(f, "肩"),
      BodyPart::Skirt => write!(f, "スカート"),
      BodyPart::Hand => write!(f, "手"),
    }
  }
}

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

pub fn new_mouse_response(req: &Request, info: String) -> Result<Response, ShioriError> {
  let vars = get_global_vars();
  let last_touch_info = vars.volatility.last_touch_info();
  let status = Status::from_request(req);

  // 同一に扱う
  let i = if info == "0bustdoubleclick" {
    "0bustnade".to_string()
  } else if info == "0handdoubleclick" {
    "0handnade".to_string()
  } else {
    info.clone()
  };

  if i != last_touch_info.as_str() {
    if let Some(touch_info) = vars
      .volatility
      .touch_info_mut()
      .get_mut(last_touch_info.as_str())
    {
      touch_info.reset_if_timeover()?;
    }
    vars.volatility.set_last_touch_info(i.clone());
  }

  if !get_global_vars()
    .flags()
    .check(&EventFlag::FirstRandomTalkDone(
      FIRST_RANDOMTALKS.len() as u32 - 1,
    ))
  {
    if info.as_str().contains("doubleclick") && !status.talking {
      let dummy_req = check_error!(
        Request::parse(DUMMY_REQUEST),
        ShioriError::ParseRequestError
      );
      return Ok(on_menu_exec(&dummy_req));
    } else {
      return Ok(new_response_nocontent());
    }
  }

  let response = mouse_dialogs(req, i.clone())?;

  // 一括で回数を増やす
  vars
    .volatility
    .touch_info_mut()
    .entry(i)
    .or_insert(TouchInfo::new())
    .add();

  Ok(response)
}

fn common_choice_process(dialogs: Vec<String>) -> Result<Response, ShioriError> {
  let index = choose_one(&dialogs, true).ok_or(ShioriError::ArrayAccessError)?;
  new_response_with_value_with_translate(
    format!(
      "{}{}{}",
      REMOVE_BALLOON_NUM,
      render_immersive_icon(),
      dialogs[index].clone()
    ),
    TranslateOption::with_shadow_completion(),
  )
}

static DIALOG_SEXIAL_WHILE_HITTING: Lazy<Vec<String>> = Lazy::new(|| {
  vec!["h1221204ねえ、焦らさないで。\\nもっと叩いて。\\n死を、感じさせて。".to_string()]
});

static DIALOG_TOUCH_WHILE_HITTING: Lazy<Vec<String>> =
  Lazy::new(|| vec!["h1211104優しくしないで。退屈になるじゃない。".to_string()]);

static DIALOG_SEXIAL_FIRST: Lazy<Vec<String>> =
  Lazy::new(|| vec!["h1111205……会って早々これ？\nなんというか……h1111204流石ね。".to_string()]);

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

fn is_first_sexial_allowed(vars: &mut GlobalVariables) -> bool {
  !vars.volatility.first_sexial_touch()
    && vars.volatility.ghost_up_time() < 30
    && vars.flags().check(&EventFlag::FirstClose)
}

pub fn mouse_dialogs(req: &Request, info: String) -> Result<Response, ShioriError> {
  let touch_count = get_touch_info!(info.as_str()).count()?;

  // 通常の触り反応候補
  let common_response = match info.as_str() {
    "0headdoubleclick" => head_hit_dialog(req, touch_count),
    "0headnade" => zero_head_nade(req, touch_count),
    "0facenade" => zero_face_nade(req, touch_count),
    "0handnade" => zero_hand_nade(req, touch_count),
    "0bustnade" => zero_bust_touch(req, touch_count),
    "0skirtup" => zero_skirt_up(req, touch_count),
    "0shoulderdown" => zero_shoulder_down(req, touch_count),
    "2candledoubleclick" => two_candle_double_click(req, touch_count),
    _ => None,
  };

  // その他特殊な条件で発生する触り反応
  let other_response = if info.starts_with('0') && info.contains("doubleclick") {
    // 触り反応のない部分をダブルクリックでメニュー
    Some(Ok(on_menu_exec(req)))
  } else {
    None
  };

  common_response
    .or(other_response)
    .unwrap_or_else(|| Ok(new_response_nocontent()))
}

fn zero_head_nade(req: &Request, count: u32) -> Option<Result<Response, ShioriError>> {
  let vars = get_global_vars();

  if vars.volatility.talking_place() == TalkingPlace::Library {
    return Some(on_ai_talk(req));
  }

  let results = if vars.volatility.aroused() {
    DIALOG_TOUCH_WHILE_HITTING.clone()
  } else {
    let dialogs = vec![vec![
      "h1111205何のつもり？".to_string(),
      "h1111304それ、あまり好きではないわ。".to_string(),
      "h1111207軽んじられている気がするわ。".to_string(),
    ]];
    phased_talks(count, dialogs).0
  };
  Some(common_choice_process(results))
}

fn zero_face_nade(req: &Request, count: u32) -> Option<Result<Response, ShioriError>> {
  let vars = get_global_vars();

  if vars.volatility.talking_place() == TalkingPlace::Library {
    return Some(on_ai_talk(req));
  }

  let results = if vars.volatility.aroused() {
    DIALOG_TOUCH_WHILE_HITTING.clone()
  } else {
    let dialogs = vec![vec![
      "h1111204……気安いのね。".to_string(),
      "h1111201\\1……冷たい。h1111304触れられるだけよ。\\n人間のような触れあいを求められても困るわ。".to_string(),
      "h1111104\\1すべすべだ。h1111204……もういいかしら。".to_string(),
    ]];
    phased_talks(count, dialogs).0
  };
  Some(common_choice_process(results))
}

fn zero_hand_nade(req: &Request, count: u32) -> Option<Result<Response, ShioriError>> {
  let vars = get_global_vars();

  if vars.volatility.talking_place() == TalkingPlace::Library {
    return Some(on_ai_talk(req));
  }

  let results = if vars.volatility.aroused() {
    DIALOG_TOUCH_WHILE_HITTING.clone()
  } else {
    let dialogs = vec![vec![
      "\
        h1111205\\1触れた手の感触はゼリーを掴むような頼りなさだった。\
        \\0……手が冷えるわよ。h1111204ほどほどにね。\
        "
      .to_string(),
      "\
        h1111205あなたが何を伝えたいのかは、なんとなく分かるけれど。\\n\
        ……h1111204それは不毛というものよ。\
        "
      .to_string(),
      "\
        h1111205\\1彼女の指は長い。
        h1111210……うん。\\n\
        "
      .to_string(),
    ]];
    phased_talks(count, dialogs).0
  };
  Some(common_choice_process(results))
}

fn zero_skirt_up(_req: &Request, _count: u32) -> Option<Result<Response, ShioriError>> {
  let vars = get_global_vars();

  if vars.volatility.talking_place() == TalkingPlace::Library {
    return None;
  }

  let results = if vars.volatility.aroused() {
    DIALOG_SEXIAL_WHILE_HITTING.clone()
  } else {
    let mut conbo_parts: Vec<Vec<String>> = vec![vec!["hr2144402……！h1141102\\n".to_string()]];
    if is_first_sexial_allowed(vars) {
      vars.volatility.set_first_sexial_touch(true);
      conbo_parts.push(DIALOG_SEXIAL_FIRST.clone());
    } else {
      conbo_parts.push(vec![
        "h1111204いいもの見たって顔してる。h1111210屈辱だわ。".to_string(),
        "h1111205ああ、ひどい人。h1111210泣いてしまいそうだわ。".to_string(),
        "h1111211秘されたものほど暴きたくなるものね。\\n\
        h1111204ところで、相応の代償を払う用意はあるのでしょうね。"
          .to_string(),
        "h1111304悪餓鬼。".to_string(),
      ]);
    }
    all_combo(&conbo_parts)
  };
  Some(common_choice_process(results))
}

fn zero_shoulder_down(_req: &Request, count: u32) -> Option<Result<Response, ShioriError>> {
  let dialogs = vec![
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
  ];
  Some(common_choice_process(phased_talks(count, dialogs).0))
}

fn zero_bust_touch(req: &Request, count: u32) -> Option<Result<Response, ShioriError>> {
  let vars = get_global_vars();

  if vars.volatility.talking_place() == TalkingPlace::Library {
    return Some(on_ai_talk(req));
  }

  let results = if vars.volatility.aroused() {
    DIALOG_TOUCH_WHILE_HITTING.clone()
  } else {
    let zero_bust_touch_threshold = 12;
    let mut zero_bust_touch = Vec::new();
    if is_first_sexial_allowed(vars) {
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
  };
  Some(common_choice_process(results))
}

pub fn on_head_hit_cancel(_req: &Request) -> Result<Response, ShioriError> {
  let m = "\\1……踏みとどまった。".to_string();
  new_response_with_value_with_translate(m, TranslateOption::simple_translate())
}

pub fn on_head_hit(_req: &Request) -> Result<Response, ShioriError> {
  let vars = get_global_vars();
  vars.flags_mut().done(EventFlag::FirstHitTalkStart);
  to_aroused();
  let m = "\\t\\*\
    h1111201あら、どうし…h1111112…h1000000っ！？\\n\
    \\1半ば衝動的に、彼女を突き飛ばした。\\n\
    \\0\\![bind,ex,流血,1]h1112401……。\\n\
    \\1立ち上がったハイネ。頭から血が流れている。\\n\
    彼女は呆けたように私を見つめ…………h1222804\\1笑った。\\n\
    \\0……殴られるなんて、ずいぶん久しぶりだわ。\\n\
    h1222310フフ、あなたのせいで思い出しちゃった。\\n\
    痛みってこういうものだったわね。\\n\
    ……h1222505脳天が痺れてる。素敵だわ、とても。\\n\
    h1222204ねえ、せっかくの機会だわ。もっとやってみて。\\n\
    私に悪感情を抱いたから突き飛ばしたのでしょう？\\n\
    h1222208こんなにしておいて、わざとじゃなかったなんて言わせないわ。\\n\
    h1222204遠慮はいらない。あなたの気が済むまで殴って。\\n\
    さあ。h1322813さあ！\
    \\1……！\
    "
  .to_string();
  new_response_with_value_with_translate(m, TranslateOption::simple_translate())
}

pub fn head_hit_dialog(req: &Request, count: u32) -> Option<Result<Response, ShioriError>> {
  let vars = get_global_vars();

  if vars.volatility.talking_place() == TalkingPlace::Library {
    return Some(on_ai_talk(req));
  }

  let is_aroused = vars.volatility.aroused();
  if !vars.flags().check(&EventFlag::FirstHitTalkStart) {
    get_touch_info!("0headdoubleclick").reset(); // 選択肢だけなのでカウントしない
    let dialogs = vec!["\
    \\s[1111101]\\![*]\\q[突き飛ばす,OnHeadHit]\\n\\![*]\\q[やめておく,OnHeadHitCancel]\
    "
    .to_string()];
    Some(common_choice_process(dialogs))
  } else if !is_aroused {
    get_touch_info!("0headdoubleclick").reset(); // 初回はカウントしない
    to_aroused();
    let dialogs = vec![
      "h1000000痛っ……\\n\\0\\![bind,ex,流血,1]h1311204あら、その気になってくれた？".to_string(),
    ];
    Some(common_choice_process(dialogs))
  } else {
    // 各段階ごとのセリフ
    let suffixes_list = vec![
      vec![
        "h1311204すてき。h1311207もっとして。".to_string(),
        "h1311206ああ、星が舞っているわ。\\nh1311215痛くて苦しくて、死んでしまいそう。".to_string(),
        "h1221104ひどい。h1221110ひどいわ。\\nh1322513癖になってしまったらどうするの？".to_string(),
      ],
      vec![
        "h1311204あは、ずきずきする。\\nh1311307もはや血は通っていないはずなのに、脈打ってる。"
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
      h1211310………………\\0\\![bind,ex,流血,0]h1111310ふー。\\n\
      h1111205\\1……落ち着いたようだ。\\n\
      "
      .to_string()],
    ];

    let (suffixes, is_last) = phased_talks(count, suffixes_list);

    if is_last {
      vars.volatility.set_aroused(false);
      get_touch_info!("0headdoubleclick").reset();
      return Some(common_choice_process(suffixes));
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
    Some(common_choice_process(result))
  }
}

fn two_candle_double_click(_req: &Request, _count: u32) -> Option<Result<Response, ShioriError>> {
  let vars = get_global_vars();
  // 没入度固定時は何もしない
  if vars.volatility.is_immersive_degrees_fixed() {
    return None;
  }
  if vars.volatility.talking_place() == TalkingPlace::Library {
    light_candle_fire()
  } else {
    blow_candle_fire()
  }
}

fn blow_candle_fire() -> Option<Result<Response, ShioriError>> {
  let vars = get_global_vars();
  let immersive_degrees = vars.volatility.immersive_degrees();
  for i in 0..=IMMERSIVE_ICON_COUNT {
    let threshold = IMMERSIVE_RATE_MAX / IMMERSIVE_ICON_COUNT * i;
    if immersive_degrees < threshold {
      vars.volatility.set_immersive_degrees(threshold);
      if play_sound(SOUND_BLOW_CANDLE).is_err() {
        return Some(Err(ShioriError::PlaySoundError));
      }
      // 没入度最大なら書斎へ移動
      let m = if threshold == IMMERSIVE_RATE_MAX {
        vars.volatility.set_talking_place(TalkingPlace::Library);
        let messages = match moving_to_library_talk() {
          Ok(v) => v,
          Err(e) => return Some(Err(e)),
        };
        let index = match choose_one(&messages, true).ok_or(ShioriError::TalkNotFound) {
          Ok(v) => v,
          Err(e) => return Some(Err(e)),
        };
        messages[index].to_owned()
      } else if !vars.flags().check(&EventFlag::FirstPlaceChange) {
        match i {
          1 => "\\1火を消した。\\nなんだか胸騒ぎがする。".to_string(),
          2 => "h1111105\\1ハイネの目線が虚ろになってきている気がする。".to_string(),
          4 => "h1111105\\1残り一本だ……".to_string(),
          _ => "".to_string(),
        }
      } else {
        "".to_string()
      };
      return Some(new_response_with_value_with_translate(
        format!(
          "\\0{}{}\\p[2]{}{}",
          render_shadow(true),
          render_immersive_icon(),
          shake_with_notext(),
          m
        ),
        TranslateOption::with_shadow_completion(),
      ));
    }
  }
  None
}

// 没入度を下げ、ろうそくを点ける
fn light_candle_fire() -> Option<Result<Response, ShioriError>> {
  let vars = get_global_vars();
  let immersive_degrees = vars.volatility.immersive_degrees();
  if immersive_degrees == 0 || vars.volatility.is_immersive_degrees_fixed() {
    return None;
  }
  for i in (0..=IMMERSIVE_ICON_COUNT).rev() {
    let threshold = IMMERSIVE_RATE_MAX / IMMERSIVE_ICON_COUNT * i;
    if immersive_degrees > threshold {
      if play_sound(SOUND_LIGHT_CANDLE).is_err() {
        return Some(Err(ShioriError::PlaySoundError));
      }
      // 没入度0なら居間へ移動
      let m = if threshold == 0 && vars.volatility.talking_place() == TalkingPlace::Library {
        vars.volatility.set_talking_place(TalkingPlace::LivingRoom);
        let messages = match moving_to_living_room_talk() {
          Ok(v) => v,
          Err(e) => return Some(Err(e)),
        };
        let index = match choose_one(&messages, true).ok_or(ShioriError::TalkNotFound) {
          Ok(v) => v,
          Err(e) => return Some(Err(e)),
        };
        messages[index].to_owned()
      } else {
        "".to_string()
      };
      vars.volatility.set_immersive_degrees(threshold);
      return Some(new_response_with_value_with_translate(
        format!(
          "\\0{}{}\\p[2]{}{}",
          render_shadow(true),
          render_immersive_icon(),
          shake_with_notext(),
          m
        ),
        TranslateOption::with_shadow_completion(),
      ));
    }
  }
  None
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
