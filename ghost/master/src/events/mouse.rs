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
  get_read, get_write, EventFlag, TouchInfo, CHAIN_TALK_STATE, FLAGS, GHOST_UP_TIME,
  IMMERSIVE_DEGREES, LAST_TOUCH_INFO, LIBRARY_TRANSITION_SEQUENSE_DIALOG_INDEX, TALKING_PLACE,
  TOUCH_INFO,
};
use shiorust::message::{Parser, Request, Response};

use super::talk::TalkType;

const SOUND_LIGHT_CANDLE: &str = "マッチで火をつける.mp3";
const SOUND_BLOW_CANDLE: &str = "マッチの火を吹き消す.mp3";

#[macro_export]
macro_rules! get_touch_info {
  ($info:expr) => {
    get_write(&TOUCH_INFO)
      .entry($info.to_string())
      .or_insert($crate::system::variables::TouchInfo::new())
  };
}

pub(crate) fn new_mouse_response(req: &Request, info: String) -> Result<Response, ShioriError> {
  let status = Status::from_request(req);

  if info != get_read(&LAST_TOUCH_INFO).as_str() {
    if let Some(touch_info) = get_write(&TOUCH_INFO).get_mut(get_read(&LAST_TOUCH_INFO).as_str()) {
      touch_info.reset_if_timeover()?;
    }
    *get_write(&LAST_TOUCH_INFO) = info.clone();
  }

  if !get_read(&FLAGS).check(&EventFlag::FirstRandomTalkDone(
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

  let response = mouse_dialogs(req, info.clone())?;

  // 一括で回数を増やす
  get_write(&TOUCH_INFO)
    .entry(info)
    .or_insert(TouchInfo::new())
    .add();

  Ok(response)
}

/// 単層プールからトークを1本選んで応答を返す。
/// 空文字列は候補から除外し、候補が無ければ無反応を返す。
fn common_choice_process(dialogs: Vec<&str>) -> Result<Response, ShioriError> {
  let dialogs: Vec<&str> = dialogs.into_iter().filter(|s| !s.is_empty()).collect();
  if dialogs.is_empty() {
    return Ok(new_response_nocontent());
  }
  let index = choose_one(&dialogs, true).ok_or(ShioriError::ArrayAccessError)?;
  let text = dialogs[index];

  new_response_with_value_with_translate(
    format!("{}{}{}", REMOVE_BALLOON_NUM, render_immersive_icon(), text,),
    TranslateOption::with_shadow_completion(),
  )
}

pub(crate) fn mouse_dialogs(req: &Request, info: String) -> Result<Response, ShioriError> {
  let touch_count = get_touch_info!(info.as_str()).count()?;

  // チェイントーク発火チェック
  if let Some(chain_response) = check_chain_talk(&info) {
    return chain_response;
  }

  // 通常の触り反応候補
  let common_response = match info.as_str() {
    // nade
    "0headnade" => zero_head_nade(req, touch_count),
    "0facenade" => zero_face_nade(req, touch_count),
    "0handnade" => zero_hand_nade(req, touch_count),
    "0mouthnade" => zero_mouth_nade(req, touch_count),
    // wheel
    "0handdown" => zero_hand_down(req, touch_count),
    "0shoulderdown" => zero_shoulder_down(req, touch_count),
    // doubleclick
    "0facedoubleclick" => zero_face_doubleclick(req, touch_count),
    "0handdoubleclick" => zero_hand_doubleclick(req, touch_count),
    "0shoulderdoubleclick" => zero_shoulder_doubleclick(req, touch_count),
    "0mouthdoubleclick" => zero_mouth_doubleclick(req, touch_count),
    // 蝋燭
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

// 頭を撫でる
fn zero_head_nade(req: &Request, _count: u32) -> Option<Result<Response, ShioriError>> {
  if *get_read(&TALKING_PLACE) == TalkingPlace::Library {
    return Some(on_ai_talk(req));
  }
  let dialogs = vec![
    "h1111205あなたの手、少し震えてる。\\n緊張しているのね。",
    "h1111205あなたの手、\\n前より震えなくなった。\\nh1111207慣れたものね。\\nh1111205……私も、かしら。",
    "h1111210……頭を撫でられるのにも慣れたわ。\\nh1111204あなたにとっては",
    "h1111206頭を撫でるというのは、\\n本来は上位者が下位者にする行為。\\nh1111204あなたは私にそうしたいの？\\nh1111210……度胸があるのね。",
    "h1111206上位者が下位者にする行為……\\nと、前に言ったかしら。\\nh1111205……今は、そうではない。\\nけれど、それが何かというのは分からない。\\nh1111204……困ったわ。",
    "h1111210……好きにしなさい。\\nh1111208\\1指を通すたび、かすかに頭が寄る。\\n撫でているのか、撫でさせられているのか\\n分からなくなった。",
    "h1111205\\1冷たい髪が指の間を滑る。\\nh1111204随分と自然にやるのね。\\n誰にでもそうしているの？\\nh1111206それとも、幽霊が珍しいだけ？",
    "h1111205\\1指先で髪を梳く。\\nh1111204……何か気の利いたことを\\n言おうとしたのだけれど。\\nh1111210……まあ、いいわ。",
    "h1111210\\1手を止めてみる。\\n\\0……h1111104終わり？\\n\\1首を振ると、\\nh1111110\\1また目を閉じた。",
  ];
  Some(common_choice_process(dialogs))
}

// 顔を撫でる
fn zero_face_nade(req: &Request, _count: u32) -> Option<Result<Response, ShioriError>> {
  if *get_read(&TALKING_PLACE) == TalkingPlace::Library {
    return Some(on_ai_talk(req));
  }

  let dialogs = vec![
    "h1111206……顔。\\nh1111204握手でも、手を繋ぐのでもなく、\\n顔に触れる。\\nh1111205それがどういう行為か分かっているのかしら。",
    "h1111205……。h1111205前は、こういうとき\\nもう少し器用に返せたのだけれど。\\nh1111210……待って。今、考えているから。",
    "h1111208\\1触れた頬が、ほんの少しだけ、\\n手のひらに押し返してくる。",
    "h1000000……っ。\\nh1111204……驚かせないで。\\nh1111210触る前に、\\n一言くらいあってもいいでしょう。",
    "h1111205……あなたの突飛な行動にも慣れてきたわ。\\nh1111204……あなたが何を求めているのかも、ね。h1111206\\1ハイネが私の手に手を添えた。",
    "h1000000\\1頬に触れると、かすかに頬が動いた。\\n笑っている。h1111204……何よ。",
    "h1111204何を考えているか、\\n顔に書いてあるわよ。\\nh1111210……隠すのが下手ね。",
    "h1111205あなたの目。\\nh1111204……今何を言っているか、分かるわよ。\\nh1111205……分かるから、困っているの。",
    "h1111206\\1頬に触れていると、\\nこちらの手に頬を押し当ててきた。\\nh1111210……温かい。\\nh1111205忘れていたわ、こういうの。",
  ];
  Some(common_choice_process(dialogs))
}

// 手を撫でる
fn zero_hand_nade(req: &Request, _count: u32) -> Option<Result<Response, ShioriError>> {
  if *get_read(&TALKING_PLACE) == TalkingPlace::Library {
    return Some(on_ai_talk(req));
  }

  let dialogs = vec![
    "h1111205\\1触れた指先が冷たい。\\nh1111204あなたの手は温かいわね。\\nh1111206生きている身体の熱。\\n……ずいぶん久しぶりに触れた気がする。",
    "h1111205……h1111204何か伝えたいことがあるなら、\\n書いてくれた方が確実よ。\\nh1111206……こういうのは、得意じゃないの。",
    "h1111205……慣れていないのよ。\\n言葉以外で、何かを伝え合うことに。",
    "h1111205……言葉よりも遥かに饒舌に、\\nあなたの意図が伝わってくる。\\nh1111210知らなかったわ、こういうのは。",
    "h1111205あなたの指、\\nペンだこがあるのね。\\nh1111204……描く人の手。\\nh1111206触れば分かるわ、\\nそういうことは。",
    "h1111205……ペンだこ、少し固くなったわね。\\nh1111210ずっと描いているものね。",
    "h1111205\\1指を絡めているうち、\\n中指に多く触れられていることに気づいた。\\n……ペンだこ。\\nh1111205……愛らしい、というのは変かしら。",
  ];
  Some(common_choice_process(dialogs))
}

// 抱き寄せる
fn zero_shoulder_down(_req: &Request, _count: u32) -> Option<Result<Response, ShioriError>> {
  let dialogs = vec![
    "h1000000！\\1背中に手を回す。\\nハイネの体がこわばるのを感じた。h1111205……驚いたじゃない。",
  ];
  Some(common_choice_process(dialogs))
}

/// チェイントーク発火チェック。
/// チェイン待機中かつ対象部位が一致し、制限時間内なら発火。
fn check_chain_talk(info: &str) -> Option<Result<Response, ShioriError>> {
  let state = get_read(&CHAIN_TALK_STATE).clone();
  if let Some(chain) = state {
    let now = *get_read(&GHOST_UP_TIME);
    if now <= chain.expires_at && info == chain.target_part {
      // チェイン発火
      *get_write(&CHAIN_TALK_STATE) = None;
      if let Some(cb) = chain.callback {
        cb();
      }
      return Some(new_response_with_value_with_translate(
        format!(
          "{}{}{}",
          REMOVE_BALLOON_NUM,
          render_immersive_icon(),
          chain.chain_text,
        ),
        TranslateOption::with_shadow_completion(),
      ));
    }
    // 期限切れならクリア
    if now > chain.expires_at {
      *get_write(&CHAIN_TALK_STATE) = None;
    }
  }
  None
}

fn zero_mouth_nade(_req: &Request, _count: u32) -> Option<Result<Response, ShioriError>> {
  let dialogs = vec![
    "h1111204\\1手を優しく掴まれた。h1111206唇なんて、触るものじゃないでしょう。",
    "h1121210……h1000000\\1！\\nh1111210\\1軽く噛まれた……。\\nh1111204……驚いた？h1111210お返しよ。",
    "h1111210\\1尖った歯が覗く。\\n犬歯が人間よりも長いのだ。h1111304……満足かしら？",
  ];
  Some(common_choice_process(dialogs))
}

// ── wheel系 ──

// 手を引く
fn zero_hand_down(req: &Request, _count: u32) -> Option<Result<Response, ShioriError>> {
  if *get_read(&TALKING_PLACE) == TalkingPlace::Library {
    return Some(on_ai_talk(req));
  }

  let dialogs = vec![
    "h1111205\\1腕に抱きつく。h1111210……べたべたしないでちょうだい。\\nお互い、やることがあるのだから。",
    "h1111205\\1腕に抱きつく。h1111206……本が、読みづらいわ。",
    "h1111205\\1腕に抱きつく。h1111206……本が、読みづらいわ。\\nh1111210また後でね。",
  ];
  Some(common_choice_process(dialogs))
}

// ── doubleclick系 ──

fn zero_face_doubleclick(req: &Request, _count: u32) -> Option<Result<Response, ShioriError>> {
  if *get_read(&TALKING_PLACE) == TalkingPlace::Library {
    return Some(on_ai_talk(req));
  }

  let dialogs = vec![
    "h1111204……。\\nh1111306頬をつつく……h1111204子供にするような真似ね。\\nh1111206私をいくつだと思っているのかしら。",
    "h1111207……ん。\\nh1111210\\1つつくと、\\n今度は隠さずに笑った。\\nh1111207……もう一回。",
    "h1111101……顔になにかついていたかしら。\\n……h1111204悪戯なら、やめてくれる？",
    "h1111208\\1つつくと、\\n頬がわずかに動いた。\\n笑いを堪えている。\\nh1111204……笑ってないわ。",
    "h1111207……。\\_w[300]\\nh1111210\\1つついた指を、\\nそっと手で包まれた。\\nh1111208……捕まえた。",
  ];
  Some(common_choice_process(dialogs))
}

// 手をつつく、手に触れる
fn zero_hand_doubleclick(req: &Request, _count: u32) -> Option<Result<Response, ShioriError>> {
  if *get_read(&TALKING_PLACE) == TalkingPlace::Library {
    return Some(on_ai_talk(req));
  }

  let dialogs = vec![
    "h1111206\\1指の一本一本をたどる。\\n関節の節が、生きている人より硬い。\\nh1111204……丁寧ね。\\nh1111205壊れ物を扱うみたい。",
    "h1111205……手のひらの線を、なぞられるのが好き。\\nh1111206……言葉にすると恥ずかしいものね。",
  ];
  Some(common_choice_process(dialogs))
}

fn zero_shoulder_doubleclick(_req: &Request, _count: u32) -> Option<Result<Response, ShioriError>> {
  let dialogs: Vec<&str> = vec![];
  Some(common_choice_process(dialogs))
}

// 口に手を当てる、口を触る
fn zero_mouth_doubleclick(_req: &Request, _count: u32) -> Option<Result<Response, ShioriError>> {
  let dialogs: Vec<&str> = vec![];
  Some(common_choice_process(dialogs))
}

fn two_candle_double_click(_req: &Request, _count: u32) -> Option<Result<Response, ShioriError>> {
  if *get_read(&TALKING_PLACE) == TalkingPlace::Library {
    light_candle_fire()
  } else {
    blow_candle_fire()
  }
}

fn blow_candle_fire() -> Option<Result<Response, ShioriError>> {
  for i in 0..=IMMERSIVE_ICON_COUNT {
    let threshold = IMMERSIVE_RATE_MAX / IMMERSIVE_ICON_COUNT * i;
    if *get_read(&IMMERSIVE_DEGREES) < threshold {
      *get_write(&IMMERSIVE_DEGREES) = threshold;
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
        *get_write(&LIBRARY_TRANSITION_SEQUENSE_DIALOG_INDEX) += 1;
        if *get_read(&LIBRARY_TRANSITION_SEQUENSE_DIALOG_INDEX) as usize >= dialogs.len() {
          *get_write(&LIBRARY_TRANSITION_SEQUENSE_DIALOG_INDEX) = 0;
        }
      }
      let dialog = dialogs[*get_read(&LIBRARY_TRANSITION_SEQUENSE_DIALOG_INDEX) as usize]
        [(i - 1) as usize]
        .to_owned();

      // 話題解放メッセージ
      let system_message = if threshold == IMMERSIVE_RATE_MAX {
        *get_write(&TALKING_PLACE) = TalkingPlace::Library; // 没入度最大なら書斎へ移動
        let message = if get_read(&FLAGS).check(&EventFlag::FirstPlaceChange) {
          "".to_string()
        } else {
          // 初回は抽象・過去トークの開放を通知
          get_write(&FLAGS).done(EventFlag::FirstPlaceChange);
          let achieved_talk_types = [TalkType::Abstract];
          achieved_talk_types.iter().for_each(|t| {
            get_write(&FLAGS).done(EventFlag::TalkTypeUnlock(*t));
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
  if *get_read(&IMMERSIVE_DEGREES) == 0 {
    return None;
  }
  for i in (0..=IMMERSIVE_ICON_COUNT).rev() {
    let threshold = IMMERSIVE_RATE_MAX / IMMERSIVE_ICON_COUNT * i;
    if *get_read(&IMMERSIVE_DEGREES) > threshold {
      // 没入度0なら居間へ移動
      let m = if threshold == 0 && *get_read(&TALKING_PLACE) == TalkingPlace::Library {
        *get_write(&TALKING_PLACE) = TalkingPlace::LivingRoom;
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
      *get_write(&IMMERSIVE_DEGREES) = threshold;
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

const DUMMY_REQUEST: &str = "GET SHIORI/3.0\r\n\
Charset: UTF-8\r\n\
Sender: SSP\r\n\
SenderType: internal,raise\r\n\
SecurityLevel: local\r\n\
Status: choosing,balloon(0=0)\r\n\
ID: OnFirstBoot\r\n\
BaseID: OnBoot\r\n\
Reference0: 1\r\n\r\n";

#[cfg(test)]
mod tests {
  use super::*;
  use crate::system::variables::{ChainTalkState, CHAIN_TALK_STATE, GHOST_UP_TIME};
  use std::sync::atomic::{AtomicBool, Ordering};

  static CALLBACK_CALLED: AtomicBool = AtomicBool::new(false);

  fn reset_chain_state() {
    *get_write(&CHAIN_TALK_STATE) = None;
    CALLBACK_CALLED.store(false, Ordering::SeqCst);
  }

  fn set_test_chain(target: &str, expires_at: u64, with_callback: bool) {
    *get_write(&CHAIN_TALK_STATE) = Some(ChainTalkState {
      target_part: target.to_string(),
      chain_text: "チェインテスト".to_string(),
      expires_at,
      callback: if with_callback {
        Some(|| {
          CALLBACK_CALLED.store(true, Ordering::SeqCst);
        })
      } else {
        None
      },
    });
  }

  /// チェイントーク機構の全テスト。
  /// グローバル変数を使うため1つのテスト関数にまとめて直列実行する。
  #[test]
  fn test_chain_talk_mechanism() {
    // 1. 正しい部位・制限時間内で発火する
    reset_chain_state();
    *get_write(&GHOST_UP_TIME) = 10;
    set_test_chain("0handnade", 30, true);

    let result = check_chain_talk("0handnade");
    assert!(result.is_some(), "チェインが発火するべき");
    assert!(
      CALLBACK_CALLED.load(Ordering::SeqCst),
      "コールバックが呼ばれるべき"
    );
    assert!(
      get_read(&CHAIN_TALK_STATE).is_none(),
      "発火後にチェイン状態がクリアされるべき"
    );

    // 2. 別部位では発火しない
    reset_chain_state();
    *get_write(&GHOST_UP_TIME) = 10;
    set_test_chain("0handnade", 30, true);

    let result = check_chain_talk("0headnade");
    assert!(result.is_none(), "別部位ではチェインが発火しないべき");
    assert!(
      !CALLBACK_CALLED.load(Ordering::SeqCst),
      "コールバックが呼ばれないべき"
    );
    assert!(
      get_read(&CHAIN_TALK_STATE).is_some(),
      "チェイン状態が残っているべき"
    );

    // 3. 制限時間超過で発火しない
    reset_chain_state();
    *get_write(&GHOST_UP_TIME) = 31;
    set_test_chain("0handnade", 30, true);

    let result = check_chain_talk("0handnade");
    assert!(result.is_none(), "期限切れではチェインが発火しないべき");
    assert!(
      !CALLBACK_CALLED.load(Ordering::SeqCst),
      "コールバックが呼ばれないべき"
    );
    assert!(
      get_read(&CHAIN_TALK_STATE).is_none(),
      "期限切れでチェイン状態がクリアされるべき"
    );

    // 4. コールバックなしでも発火する
    reset_chain_state();
    *get_write(&GHOST_UP_TIME) = 10;
    set_test_chain("0shoulderdown", 30, false);

    let result = check_chain_talk("0shoulderdown");
    assert!(
      result.is_some(),
      "コールバックなしでもチェインは発火するべき"
    );
    assert!(
      get_read(&CHAIN_TALK_STATE).is_none(),
      "発火後にチェイン状態がクリアされるべき"
    );

    // 5. チェイン状態がなければNone
    reset_chain_state();
    *get_write(&GHOST_UP_TIME) = 10;
    let result = check_chain_talk("0handnade");
    assert!(result.is_none(), "チェイン状態がなければNoneを返すべき");
  }
}
