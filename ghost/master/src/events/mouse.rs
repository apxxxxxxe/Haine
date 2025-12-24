use crate::check_error;
use crate::events::first_boot::FIRST_RANDOMTALKS;
use crate::events::menu::on_menu_exec;
use crate::events::on_ai_talk;
use crate::events::render_immersive_icon;
use crate::events::TalkingPlace;
use crate::events::IMMERSIVE_ICON_COUNT;
use crate::events::IMMERSIVE_RATE_MAX;
use crate::system::error::ShioriError;
use crate::system::response::*;
use crate::system::status::Status;
use crate::system::variables::{
  EventFlag, TouchInfo, FIRST_SEXIAL_TOUCH, FLAGS, GHOST_UP_TIME, IMMERSIVE_DEGREES,
  LAST_TOUCH_INFO, LIBRARY_TRANSITION_SEQUENSE_DIALOG_INDEX, TALKING_PLACE, TOUCH_INFO,
};
use once_cell::sync::Lazy;
use shiorust::message::{Parser, Request, Response};

use super::talk::TalkType;

const SOUND_LIGHT_CANDLE: &str = "マッチで火をつける.mp3";
const SOUND_BLOW_CANDLE: &str = "マッチの火を吹き消す.mp3";

#[macro_export]
macro_rules! get_touch_info {
  ($info:expr) => {
    TOUCH_INFO
      .write()
      .unwrap()
      .entry($info.to_string())
      .or_insert($crate::system::variables::TouchInfo::new())
  };
}

pub(crate) fn new_mouse_response(req: &Request, info: String) -> Result<Response, ShioriError> {
  let status = Status::from_request(req);

  // 同一に扱う
  let i = if info == "0bustdoubleclick" {
    "0bustnade".to_string()
  } else if info == "0handdoubleclick" {
    "0handnade".to_string()
  } else {
    info.clone()
  };

  if i != LAST_TOUCH_INFO.read().unwrap().as_str() {
    if let Some(touch_info) = TOUCH_INFO
      .write()
      .unwrap()
      .get_mut(LAST_TOUCH_INFO.read().unwrap().as_str())
    {
      touch_info.reset_if_timeover()?;
    }
    *LAST_TOUCH_INFO.write().unwrap() = i.clone();
  }

  if !FLAGS.read().unwrap().check(&EventFlag::FirstRandomTalkDone(
    FIRST_RANDOMTALKS.len() as u32 - 1,
  )) {
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
  TOUCH_INFO
    .write()
    .unwrap()
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

static DIALOG_SEXIAL_FIRST: Lazy<Vec<String>> =
  Lazy::new(|| vec!["h1111205……会って早々これ？\nなんというか……h1111204流石ね。".to_string()]);

static DIALOG_SEXIAL_SCOLD: Lazy<Vec<String>> = Lazy::new(|| {
  vec![
      "h1111202……いくら他人の目がないとはいえ、\\nh1111204品性を疑うわ。".to_string(),
      "h1111205これがあなたのやりたいこと？\\nh1111204くだらないのね。".to_string(),
      "h1111205スキンシップにしては\\nセンスが無いと思うわ。".to_string(),
      "h1111210情熱的という人もいるでしょうし、\\n野蛮で下劣という人もいるでしょうね。\\n\\nh1111204私は後者よ、お猿さん。".to_string(),
    ]
});

static DIALOG_SEXIAL_AKIRE: Lazy<Vec<String>> = Lazy::new(|| {
  vec![
    "h1111201さっきからずいぶん必死ね。\\nh1111304ばかみたいな顔してるわよ。".to_string(),
    "h1111304面白い顔。h1111310鏡で見せてあげたいわ。".to_string(),
    "h1111104悪戯がすぎるわよ。".to_string(),
    "h1111103はあ……h1111106何が楽しいんだか。".to_string(),
    "h1111204その熱意は買うけれど。\\n……h1111210虚しくないの？".to_string(),
    "h1111204…………退屈。".to_string(),
  ]
});

fn is_first_sexial_allowed() -> bool {
  !*FIRST_SEXIAL_TOUCH.read().unwrap()
    && *GHOST_UP_TIME.read().unwrap() < 30
    && FLAGS.read().unwrap().check(&EventFlag::FirstClose)
}

pub(crate) fn mouse_dialogs(req: &Request, info: String) -> Result<Response, ShioriError> {
  let touch_count = get_touch_info!(info.as_str()).count()?;

  // 通常の触り反応候補
  let common_response = match info.as_str() {
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
  if *TALKING_PLACE.read().unwrap() == TalkingPlace::Library {
    return Some(on_ai_talk(req));
  }

  let dialogs = vec![vec![
    "h1111210\\1触れた瞬間、冷たい感触が指先に伝わった。\\nh1111204髪も肌も、\\n生きている人間のようには温かくないのよ。\\n……h1111207構わない？h1111310そう、物好きね。".to_string(),
    "h1111204\\1さらさらだ……。\\nh1111205昔、家政婦がよく私の髪を褒めてくれたわ。\\nh1111210「お嬢様の髪は絹のようで」って。\\n今でも覚えているの。\\n……h1111205懐かしいものね。".to_string(),
    "h1111204\\1恐る恐る髪に触れる。\\nh1111205そんなに遠慮しなくてもいいのに。\\nh1111210中途半端にされる方が\\nくすぐったいのよ。\\nもう少し、しっかりと。".to_string(),
  ]];
  Some(common_choice_process(phased_talks(count, dialogs).0))
}

fn zero_face_nade(req: &Request, count: u32) -> Option<Result<Response, ShioriError>> {
  if *TALKING_PLACE.read().unwrap() == TalkingPlace::Library {
    return Some(on_ai_talk(req));
  }

  let dialogs = vec![vec![
    "h1111104\\1すべすべだ。h1111204……もういいかしら。".to_string(),
    "h1111204\\1柔らかいが、どこか頼りない感触だ。\\nh1111204あなたには奇妙な感触なのでしょうね。\\nh1111210霊体の肌よ。\\n見た目ほど確かではないの。".to_string(),
    "h1111201\\1触れられながら、彼女はじっと見つめ返している。\\nh1111204興味深い表情だわ。\\n親愛に、安心。\\nh1111310ずいぶん幸せそうね。".to_string(),
  ]];
  Some(common_choice_process(phased_talks(count, dialogs).0))
}

fn zero_hand_nade(req: &Request, count: u32) -> Option<Result<Response, ShioriError>> {
  if *TALKING_PLACE.read().unwrap() == TalkingPlace::Library {
    return Some(on_ai_talk(req));
  }

  let dialogs = vec![vec![
    "\
    h1111205\\1触れた手の感触は\\n\
    ゼリーを掴むような頼りなさだった。\
    \\0……手が冷えるわよ。h1111204ほどほどにね。\
    "
    .to_string(),
    "\
    h1111205あなたが何を伝えたいのかは、\\n\
    なんとなく分かるけれど。\\n\
    ……h1111204それは不毛というものよ。\
    "
    .to_string(),
    "\
    h1111205\\1彼女の指は長い。\\n\
    h1111210……うん。
    "
    .to_string(),
    "h1111204\\1冷たい手だ。\\nh1111205あなたの手、いつもこんなに温かいの？\\nh1111210私と対照的で、不思議な感覚だわ。".to_string(),
    "h1111205\\1そっと手を握る。\\nh1111204優しい握り方ね。h1111210こわれものを扱うみたいに。\\n……h1111205そんなに繊細じゃないわよ。".to_string(),
  ]];
  Some(common_choice_process(phased_talks(count, dialogs).0))
}

fn zero_skirt_up(_req: &Request, _count: u32) -> Option<Result<Response, ShioriError>> {
  if *TALKING_PLACE.read().unwrap() == TalkingPlace::Library {
    return None;
  }

  let mut conbo_parts: Vec<Vec<String>> = vec![vec!["hr2144402……！h1141102\\n".to_string()]];
  if is_first_sexial_allowed() {
    *FIRST_SEXIAL_TOUCH.write().unwrap() = true;
    conbo_parts.push(DIALOG_SEXIAL_FIRST.clone());
  } else {
    conbo_parts.push(vec![
      "h1111204いいもの見たって顔してる。\\nh1111210屈辱だわ。".to_string(),
      "h1111205ああ、ひどい人。\\nh1111210泣いてしまいそうだわ。".to_string(),
      "h1111211秘されたものほど暴きたくなるものね。\\n\
      h1111204……もちろん、\\n\
      相応の代償を払う用意はあるのでしょうね。"
        .to_string(),
      "h1111304悪餓鬼。".to_string(),
    ]);
  }
  Some(common_choice_process(all_combo(&conbo_parts)))
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
      h1111205\\1背の高い彼女の肩に手をかけると、\\n\
      柔らかい髪が指に触れた。\
      h1111204……それで？h1111210あなたは私をどうしたいのかしら。\
      "
      .to_string(),
    ],
  ];
  Some(common_choice_process(phased_talks(count, dialogs).0))
}

fn zero_bust_touch(req: &Request, count: u32) -> Option<Result<Response, ShioriError>> {
  if *TALKING_PLACE.read().unwrap() == TalkingPlace::Library {
    return Some(on_ai_talk(req));
  }

  let zero_bust_touch_threshold = 12;
  let mut zero_bust_touch = Vec::new();
  if is_first_sexial_allowed() {
    *FIRST_SEXIAL_TOUCH.write().unwrap() = true;
    zero_bust_touch.extend(DIALOG_SEXIAL_FIRST.clone());
  } else if count < zero_bust_touch_threshold / 3 {
    zero_bust_touch.extend(vec![
      "h1111205……ずいぶん嬉しそうだけれど、\\nh1111204そんなにいいものなのかしら？".to_string(),
      "h1111210気を引きたいだけなら、\\nもっと賢い方法があると思うわ。".to_string(),
      "h1111204……あなたは、私をそういう対象として\\n見ているの？".to_string(),
      "h1111205気安いのね。\\nあまり好きではないわ。".to_string(),
      "h1111304媚びた反応を期待してるの？\\nh1112204この身体にそれを求められても、\\nね。".to_string(),
      "h1111205\\1触れた瞬間、彼女は微かに身を引いた。\\nh1111204よくもまあ、躊躇いもなく……。\\nh1111310私が生きていた頃とは、\\n随分と常識も変わったものね。".to_string(),
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
  Some(common_choice_process(zero_bust_touch))
}

fn two_candle_double_click(_req: &Request, _count: u32) -> Option<Result<Response, ShioriError>> {
  if *TALKING_PLACE.read().unwrap() == TalkingPlace::Library {
    light_candle_fire()
  } else {
    blow_candle_fire()
  }
}

fn blow_candle_fire() -> Option<Result<Response, ShioriError>> {
  for i in 0..=IMMERSIVE_ICON_COUNT {
    let threshold = IMMERSIVE_RATE_MAX / IMMERSIVE_ICON_COUNT * i;
    if *IMMERSIVE_DEGREES.read().unwrap() < threshold {
      *IMMERSIVE_DEGREES.write().unwrap() = threshold;
      // セリフ
      let dialogs = [
        [
          "h1111206少し、薄暗くなってきたかしら。".to_string(), // 1本目：光の変化への気づき
          "h1111210明るい時には見えなかったものが、\\n影の中から浮かび上がってくる。\\nh1111105光は、\\n案外多くのものを隠しているのね。".to_string(), // 2本目：隠されたものの露呈
          "h1111110闇の中では、境界線が溶けて曖昧になる。\\nh1111306人と物、肌と空気、自分と他人。\\nやがて、自分がどこにいるのかも\\n分からなくなる。".to_string(), // 3本目：輪郭の喪失
          "h1111105でも、その曖昧さが心地よくもある。\\nh1111204はっきりしているのは、\\nときに苦痛なことだから。".to_string(), // 4本目：曖昧さへの逃避
          "h1111110見えなくなって、ようやく分かることもある。\\nh1111204光の中では、気づけなかった感覚に。".to_string(), // 5本目：闇が暴く真実
        ],
        [
          "h1111204静かね。そう、とても静か。".to_string(), // 1本目：静寂への気づき
          "h1111210外の音が聞こえなくなると、\\nh1111105かえって心の中の声が大きく響くの。".to_string(), // 2本目：内なる音の増幅
          "h1111110その声は、いつも同じことを囁いている。\\nh1111306私にだけ聞こえるように、\\nでも、確かに。".to_string(), // 3本目：孤独な内声
          "h1111105静寂って、実は最も騒がしいもの。\\nh1111102聞きたくない音で溢れかえっているの。".to_string(), // 4本目：静寂の欺瞞
          "h1111110静寂が責めている。\\nh1111204逃れられない真実を、突きつけてくるのよ。".to_string(), // 5本目：静寂による審判
        ],
        [
          "h1111210寒さは感じないけれど、\\n肌が疼くような、あの感覚はしばしばあるの。".to_string(), // 1本目：温度変化への気づき
          "h1111306温もりとは、失ってから気づくもの。\\nh1111105当たり前だと思っていたのに、案外脆いものね。".to_string(), // 2本目：温もりの脆さ
          "h1111110冷たさが染み込んでくると、感覚が鈍くなる。\\nh1111204痛みも、喜びも、全て遠くなっていく。".to_string(), // 3本目：感覚の鈍化
          "h1111105もしかすると、\\nそれは悪いことではないのかもしれない。\\nh1111102感じないということは、\\n傷つかないということだから。".to_string(), // 4本目：麻痺への逃避
          "h1111110でも、感じられないということは、\\nh1111204生きていないということと、\\n同じなのかもしれないわね。".to_string(), // 5本目：無感覚と死の等価性
        ],
        [
          "h1111206植物って、静かに成長していくものね。".to_string(), // 1本目：成長への着目
          "h1111210でも、いつかは成長も止まる。\\nh1111105満開の花も、やがては散っていくもの。".to_string(), // 2本目：成長の限界
          "h1111110枯れていく過程にも、独特の美しさがある。\\nh1111306生命力を失っていく、その静謐さ。".to_string(), // 3本目：枯死の美学
          "h1111105成長し続けることの方が、\\n実は不自然なのかもしれない。\\nh1111204立ち止まり、枯れることこそ摂理。".to_string(), // 4本目：停滞の正当化
          "h1111110私も、とっくに枯れ始めているのかもしれない。\\nh1111204気づかないふりをしているだけで。".to_string(), // 5本目：自己の枯死への気づき
        ],
        [
          "h1111211色とりどりのものを見ていると、\\n目が疲れるときがあるの。".to_string(), // 1本目：色彩への疲労
          "h1111105鮮やかな色って、時として攻撃的よね。\\nh1111110主張が強すぎて、心が休まらない。".to_string(), // 2本目：色彩の攻撃性
          "h1111204色が褪せていく過程は、どこか安らかで。\\nh1111306争いがなくなって、静寂が訪れるみたい。".to_string(), // 3本目：褪色の安らぎ
          "h1111105無彩色の世界なら、\\nもっと穏やかでいられるかもしれない。\\nh1111110白と黒と灰色、それだけでいい。".to_string(), // 4本目：単調さへの憧れ
          "h1111102色を失った世界で、h1111204ようやく\\n自分の輪郭が見えなくなるのかもしれないわ。".to_string(), // 5本目：自己の消失への憧れ
        ],
        [
          "h1111204記憶って、時として重いものね。".to_string(), // 1本目：記憶の重さ
          "h1111210覚えていたいものほど曖昧になって、\\nh1111105忘れたいものほど鮮明に残っている。".to_string(), // 2本目：記憶の皮肉
          "h1111110記憶は編集される。都合よく、都合悪く。\\nh1111306真実なんて、どこにもないのかもしれない。".to_string(), // 3本目：記憶の不確実性
          "h1111105忘れることができれば、\\nどれだけ楽になれるでしょう。\\nh1111102過去に縛られずに、ただ今を生きられるのに。".to_string(), // 4本目：忘却への憧れ
          "h1111110けれど、過去と現在は地続き。\\nh1111204過去だけを捨てることなど、できない。".to_string(), // 5本目：忘却の代償
        ],
        [
          "h1111210言葉って、不思議なものよね。".to_string(), // 1本目：言葉への着目
          "h1111306伝えたいことほど、うまく言葉にならない。\\nh1111105言葉にした瞬間、\\n何かが失われてしまう気がするの。".to_string(), // 2本目：言葉の限界
          "h1111110話せば話すほど、真意から遠ざかっていく。\\nh1111204言葉は、時として真実を覆い隠すのね。".to_string(), // 3本目：言葉の欺瞞性
          "h1111105沈黙の中にこそ、\\n本当の理解があるのかもしれない。\\nh1111102言葉なんて、所詮は表面的なもの。".to_string(), // 4本目：沈黙の価値
          "h1111110結局、誰にも伝わらない。\\nh1111205ならば、最初から何も言わなければ良いの？".to_string(), // 5本目：コミュニケーションの絶望
        ],
      ];
      // 前回とは別のセリフ群になるようにする
      if i == 1 {
        *LIBRARY_TRANSITION_SEQUENSE_DIALOG_INDEX.write().unwrap() += 1;
        if *LIBRARY_TRANSITION_SEQUENSE_DIALOG_INDEX.read().unwrap() as usize >= dialogs.len() {
          *LIBRARY_TRANSITION_SEQUENSE_DIALOG_INDEX.write().unwrap() = 0;
        }
      }
      let dialog = dialogs[*LIBRARY_TRANSITION_SEQUENSE_DIALOG_INDEX.read().unwrap() as usize]
        [(i - 1) as usize]
        .to_owned();

      // 話題解放メッセージ
      let system_message = if threshold == IMMERSIVE_RATE_MAX {
        *TALKING_PLACE.write().unwrap() = TalkingPlace::Library; // 没入度最大なら書斎へ移動
        let message = if FLAGS.read().unwrap().check(&EventFlag::FirstPlaceChange) {
          "".to_string()
        } else {
          // 初回は抽象・過去トークの開放を通知
          FLAGS.write().unwrap().done(EventFlag::FirstPlaceChange);
          let achieved_talk_types = [TalkType::Abstract];
          achieved_talk_types.iter().for_each(|t| {
            FLAGS.write().unwrap().done(EventFlag::TalkTypeUnlock(*t));
          });
          let achievements_messages = achieved_talk_types
            .iter()
            .map(|t| render_achievement_message(*t))
            .collect::<Vec<_>>();
          achievements_messages.join("\\n")
        };
        format!("\\1（話題の傾向が変わりました）\\n{}", message)
      } else {
        "".to_string()
      };
      return Some(new_response_with_value_with_translate(
        format!(
          "\\_v[{}]\\0{}{}\\p[2]{}{}{}",
          SOUND_BLOW_CANDLE,
          render_shadow(true),
          render_immersive_icon(),
          shake_with_notext(),
          dialog,
          system_message,
        ),
        TranslateOption::with_shadow_completion(),
      ));
    }
  }
  None
}

// 没入度を下げ、ろうそくを点ける
fn light_candle_fire() -> Option<Result<Response, ShioriError>> {
  if *IMMERSIVE_DEGREES.read().unwrap() == 0 {
    return None;
  }
  for i in (0..=IMMERSIVE_ICON_COUNT).rev() {
    let threshold = IMMERSIVE_RATE_MAX / IMMERSIVE_ICON_COUNT * i;
    if *IMMERSIVE_DEGREES.read().unwrap() > threshold {
      // 没入度0なら居間へ移動
      let m = if threshold == 0 && *TALKING_PLACE.read().unwrap() == TalkingPlace::Library {
        *TALKING_PLACE.write().unwrap() = TalkingPlace::LivingRoom;
        format!(
          "\\0\\b[{}]h1111705……。h1111101\\n\
          ……h1111110\\1ハイネはお茶を一口飲んだ。\\0\\b[{}]\\1\\n\
          \\n\\n[half](トーク傾向が元に戻りました)",
          TalkingPlace::Library.balloon_surface(),
          TalkingPlace::LivingRoom.balloon_surface(),
        )
      } else {
        "".to_string()
      };
      *IMMERSIVE_DEGREES.write().unwrap() = threshold;
      return Some(new_response_with_value_with_translate(
        format!(
          "\\_v[{}]\\0{}{}\\p[2]{}{}",
          SOUND_LIGHT_CANDLE,
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

pub(crate) fn phased_talks(count: u32, phased_talk_list: Vec<Vec<String>>) -> (Vec<String>, bool) {
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
