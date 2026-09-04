use std::str::FromStr;

use crate::events::first_boot::{FIRST_BOOT_TALK, FIRST_RANDOMTALKS};
use crate::events::input::InputId;
use crate::events::talk::randomtalk::{derivative_talks_per_talk_type, random_talks};
use crate::events::TalkType;
use crate::events::TalkingPlace;
use crate::system::error::ShioriError;
use crate::system::response::*;
use crate::system::variables::{get_read, get_write, EventFlag, PendingEvent, FLAGS, GUEST_ROOM_NIGHT_TABLE, PENDING_EVENT_TALK, RANDOM_TALK_INTERVAL, TALKING_PLACE, TALK_COLLECTION, USER_NAME};
use crate::{check_error, DERIVATIVE_TALK_REQUESTABLE};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;
use shiorust::message::{Request, Response};

use super::talk::first_boot::FIRST_CLOSE_TALK;

pub(crate) mod questions;

#[derive(Debug, Clone, Copy, FromPrimitive, ToPrimitive)]
#[repr(u32)]
enum HalloweenCostumeTrigger {
  AskToWear,
  GoatHorn,
  WitchHat,
  BlackRedCape,
}

impl std::str::FromStr for HalloweenCostumeTrigger {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let n = s.parse::<u32>().map_err(|_| ())?;
    HalloweenCostumeTrigger::from_u32(n).ok_or(())
  }
}

pub(crate) fn on_menu_exec(_req: &Request) -> Response {
  let current_talk_interval = *get_read(&RANDOM_TALK_INTERVAL);
  let mut selections = Vec::new();

  for i in [1, 3, 5, 7, 10, 0].iter() {
    if current_talk_interval == i * 60 {
      selections.push(format!(
        "\\f[underline,1]{}\\f[underline,0]",
        show_minute(i),
      ));
    } else {
      selections.push(format!(
        "\\q[{},OnTalkIntervalChanged,{}]",
        show_minute(i),
        i * 60,
      ));
    };
  }

  let talk_interval_selector = format!(
    "\
      ◆トーク頻度  【現在 {}】\\n\
      {}\
      ",
    show_minute(&(current_talk_interval / 60)),
    selections.join("  ")
  );

  let buttons = format!(
    "\\_l[0,0]\\f[align,right]{}\\__q[script:\\e]{}\\__q",
    if get_read(&FLAGS).check(&EventFlag::FirstRandomTalkDone(
      (FIRST_RANDOMTALKS.len() - 1) as u32,
    )) {
      format!("\\__q[OnConfigMenuExec]{}\\__q ", Icon::Cog)
    } else {
      "".to_string()
    },
    Icon::Cross
  );

  // ハロウィン専用メニュー項目
  let local_time = crate::system::windows::get_local_time();
  let halloween_menu = if local_time.wMonth == 10 && local_time.wDay == 31 {
    format!(
      "\\_l[0,@1.5em]\\![*]\\q[仮装してもらう,OnCostumeMenuExec,{}]\\n",
      HalloweenCostumeTrigger::AskToWear as u32
    )
  } else {
    "".to_string()
  };

  let m = format!(
    "\\_q{}{}",
    REMOVE_BALLOON_NUM,
    if !get_read(&FLAGS).check(&EventFlag::FirstRandomTalkDone(
      (FIRST_RANDOMTALKS.len() - 1) as u32,
    )) {
      "\
        \\_l[0,3em]\\![*]\\q[話の続き,OnAiTalk]\\n[150]\
        \\![*]\\q[その名前で呼ばれたくない,OnChangingUserName]\\n\
        "
      .to_string()
        + &buttons
    } else {
      format!(
        "\
          \\_l[0,1.5em]\
          \\![*]\\q[{},OnAiTalk]\\n\
          {}\
          \\![*]\\q[移動する(現在地: {}),OnChangeRooms]\\n\
          \\![*]\\q[トーク統計,OnCheckTalkCollection]\\n\
          \\![*]\\q[回想,OnStoryHistoryMenu]\
          \\_l[0,@2.5em]\
          \\![*]\\q[手紙を書く,OnWebClapOpen]\
          {}\
          \\_l[0,@2.5em]\
          {}\
          {}\
          \\1{}\
          \\0\\_l[0,0]\
          ",
        match *get_read(&TALKING_PLACE) {
          TalkingPlace::DEFAULT_LIVING_ROOM => "なにか話して",
          TalkingPlace::IMMERSED_LIVING_ROOM => "ハイネを見る",
          TalkingPlace::GuestRoom => "考えごとをする",
          TalkingPlace::Kitchen => "", // TODO
        },
        match *get_read(&TALKING_PLACE) {
          TalkingPlace::IMMERSED_LIVING_ROOM | TalkingPlace::GuestRoom => {
            "".to_string()
          }
          _ => {
            "\\![*]\\q[話しかける,OnTalk]\\n".to_string()
          }
        },
        get_read(&TALKING_PLACE),
        halloween_menu,
        talk_interval_selector,
        buttons,
        {
          let hoge = get_read(&PENDING_EVENT_TALK);
          if let Some(ref event) = *hoge {
            format!("\\![*]\\q[{},OnStoryEvent,{}]", event, event)
          } else {
            "".to_string()
          }
        }
      )
    },
  );

  new_response_with_value_with_notranslate(m, TranslateOption::balloon_surface_only())
}

pub(crate) fn on_config_menu_exec(_req: &Request) -> Response {
  let m = format!(
    "\
      \\_q\\_l[0,0]\\f[align,right]\\__q[OnMenuExec]{}\\__q \\__q[script:\\e]{}\\__q\
      \\_l[0,1.5em]\
      \\![*]\\q[呼び名を変える,OnChangingUserName]\\n\
      \\![*]\\q[リクエストボタンの表示,OnDerivativeTalkRequestButtonToggled]【現在 {}】\\n\
      ",
    Icon::ArrowLeft,
    Icon::Cross,
    if *get_read(&DERIVATIVE_TALK_REQUESTABLE) {
      "表示"
    } else {
      "非表示"
    },
  );

  new_response_with_value_with_notranslate(m, TranslateOption::balloon_surface_only())
}

pub(crate) fn on_costume_menu_exec(req: &Request) -> Result<Response, ShioriError> {
  let refs = get_references(req);
  let dialog = match check_error!(
    HalloweenCostumeTrigger::from_str(refs[0]),
    ShioriError::ParseIntError
  ) {
    HalloweenCostumeTrigger::AskToWear => "h1113101着てほしいもの？h1113204また面白いことを考えるのね。".to_string(),
    HalloweenCostumeTrigger::GoatHorn => "h1111210悪魔の象徴。h1111204拐かしてあげましょうか？".to_string(),
    HalloweenCostumeTrigger::WitchHat => "h1111210魔法、ではないけれど、近いことはできるわね。\\n\\n".to_string(),
    HalloweenCostumeTrigger::BlackRedCape => "h1111205吸血鬼かしら。\\nh1111206血は別に好みではないのだけど。\\n\\n".to_string(),
  };
  let m = format!(
    "\
      \\_l[0,3em]\\_q\
      \\![*]\\q[ヤギ角,\"script:\\![bind,頭,ヤギ角,1]\\![raise,OnCostumeMenuExec,{}]\"]\\_l[8em,@0]\\![*]\\q[外す,\"script:\\![bind,頭,ヤギ角,0]\\![raise,OnCostumeMenuExec,99]\"]\\n\
      \\![*]\\q[魔女帽,\"script:\\![bind,頭,魔女帽,1]\\![raise,OnCostumeMenuExec,{}]\"]\\_l[8em,@0]\\![*]\\q[外す,\"script:\\![bind,頭,魔女帽,0]\\![raise,OnCostumeMenuExec,99]\"]\\n\
      \\![*]\\q[黒赤マント,\"script:\\![bind,トップス+,黒赤マント,1]\\![raise,OnCostumeMenuExec,{}]\"]\\_l[8em,@0]\\![*]\\q[外す,\"script:\\![bind,トップス+,黒赤マント,0]\\![raise,OnCostumeMenuExec,99]\"]\\n\
      \\n\
      \\q[戻る,OnMenuExec]\\_q\\_l[0,0]{}\
      ",
    HalloweenCostumeTrigger::GoatHorn as u32,
    HalloweenCostumeTrigger::WitchHat as u32,
    HalloweenCostumeTrigger::BlackRedCape as u32,
    dialog,
  );

  new_response_with_value_with_translate(m, TranslateOption::with_shadow_completion())
}

fn show_minute(m: &u64) -> String {
  match m {
    0 => "黙る".to_string(),
    _ => format!("{}分", m),
  }
}

pub(crate) fn on_talk_interval_changed(req: &Request) -> Result<Response, ShioriError> {
  let refs = get_references(req);
  let v = check_error!(refs[0].parse::<u64>(), ShioriError::ParseIntError);
  *get_write(&RANDOM_TALK_INTERVAL) = v;

  Ok(on_menu_exec(req))
}

pub(crate) fn on_check_talk_collection(_req: &Request) -> Response {
  let mut lines = Vec::new();
  let mut sum = 0;
  let mut all_sum = 0;
  const DIMMED_COLOR: &str = "\\f[color,150,150,130]";
  let talk_collection = get_read(&TALK_COLLECTION);
  let talking_place = get_read(&TALKING_PLACE);
  lines.push(format!("[トーク統計: {}]\\n", talking_place));
  let talk_types = talking_place.talk_types();
  let is_unlocked_checks = talk_types
    .iter()
    .map(|t| get_read(&FLAGS).check(&EventFlag::TalkTypeUnlock(*t)))
    .collect::<Vec<_>>();
  for i in 0..talk_types.len() {
    let talk_type = talk_types[i];
    if !is_unlocked_checks[i] {
      lines.push(format!("{}{}: 未解放\\f[default]", DIMMED_COLOR, talk_type));
    } else {
      // 派生トーク込みの閲覧済みトーク数
      let len = talk_collection.get(&talk_type).map_or(0, |v| v.len());
      // 派生トークを除いた全トーク数
      let mut all_len = if let Some(v) = random_talks(talk_type) {
        v.len()
      } else {
        0
      };
      // 派生トークのトーク数を全トーク数に加える
      let derivative_talk_len = derivative_talks_per_talk_type()
        .get(&talk_type)
        .map_or(0, |v| v.len());
      all_len += derivative_talk_len;
      let anal = if len < all_len {
        format!(
          "\\n  \\f[height,13]\\q[未読トーク再生,OnCheckUnseenTalks,{}]\\f[default]",
          talk_type as u32
        )
      } else {
        "".to_string()
      };
      lines.push(format!("{}: {}/{}{}", talk_type, len, all_len, anal));
      sum += len;
      all_sum += all_len;
    }
  }

  new_response_with_value_with_notranslate(
    format!(
      "\\_q{}\\n[150]\
        ---\\n[150]\
        TOTAL: {}/{}\\n[200]\
        \\q[戻る,OnMenuExec]",
      lines.join("\\n"),
      sum,
      all_sum
    ),
    TranslateOption::balloon_surface_only(),
  )
}

pub(crate) fn on_change_rooms(_req: &Request) -> Response {
  let talking_place = get_read(&TALKING_PLACE);
  let mut room_s = "".to_string();
  let rooms = vec![
    TalkingPlace::DEFAULT_LIVING_ROOM,
    TalkingPlace::GuestRoom,
    // TalkingPlace::Kitchen,
  ];
  for room in rooms {
    if room == *talking_place {
      room_s.push_str(format!("\\![*]\\_q{}（現在地）\\_q\\n", room).as_str());
    } else {
      let m = match room {
        TalkingPlace::GuestRoom => format!("少し休む（{}へ）", room),
        _ => format!("{}へ移動", room),
      };
      room_s.push_str(format!("\\![*]\\q[{},OnChangeRoomsSelected,{}]\\n", m, room).as_str());
    }
  }
  new_response_with_value_with_notranslate(
    format!(
      "\
        \\1\\b[4]どこへ行く？\\n\
        {}\
        ",
      room_s,
    ),
    TranslateOption::balloon_surface_only(),
  )
}

pub(crate) fn on_change_rooms_selected(req: &Request) -> Result<Response, ShioriError> {
  let refs = get_references(req);
  let talking_place = check_error!(refs[0].parse::<TalkingPlace>(), ShioriError::ParseRoomError);
  let current_talking_place = *get_read(&TALKING_PLACE);
  *get_write(&TALKING_PLACE) = talking_place;
  let m = match (current_talking_place, talking_place) {
    (TalkingPlace::LivingRoom(_), TalkingPlace::GuestRoom) => {
      let message = if get_read(&FLAGS).check(&EventFlag::TalkTypeUnlock(TalkType::GuestRoom)) {
        "".to_string()
      } else {
        // 初回はゲストルームトークの開放を通知
        let achieved_talk_types = [TalkType::GuestRoom];
        achieved_talk_types.iter().for_each(|t| {
          get_write(&FLAGS).done(EventFlag::TalkTypeUnlock(*t));
        });
        let achievements_messages = achieved_talk_types
          .iter()
          .map(|t| render_achievement_message(*t))
          .collect::<Vec<_>>();
        achievements_messages.join("\\n")
      };
      // 客間に入るときは卓上を空にする(render_room_itemがこの状態をbindで送る)
      *get_write(&GUEST_ROOM_NIGHT_TABLE) = None;
      format!(
        "\
          h1111101\\1『しばらく休みたい』\\n\
          h1111104確かに、顔色が悪いわね。\\n\
          h1111210部屋に案内しましょう。\\n\
          \\1\\c\\0h1000000\\c───────────\\_w[1200]\\c\
          {}h1111210さあ、ベッドに。遠慮しないで。\\n\\n[half]\
          \\1言われるがまま、ベッドに横になる。\\n\
          h1111206ここにあるものは好きに使って。\\n\
          h1111204お茶は持ってこさせるから、少し待っていてね。\\n\
          h1000000{}\
          ",
        render_room_item(&TalkingPlace::GuestRoom),
        message
      )
    }
    (TalkingPlace::GuestRoom, TalkingPlace::LivingRoom(_)) => {
      // 部屋を出るとき、卓上の一式は従者が下げる
      *get_write(&GUEST_ROOM_NIGHT_TABLE) = None;
      format!(
        "\
          {}\\1……調子が戻ってきた。そろそろ戻ろう。\\n\
          立ち上がろうとすると、\\n\
          ティーセットがふわりと浮き、静かに運ばれていった\\p[2]{}\\1。\\n\
          \\1\\c\\0h1000000\\c───────────\\_w[1200]\\c\
          h1111105\\1(カチャ)\\n\\n[half]\
          h1111201\\b[{}]あら、お帰りなさい。\\n\
          ……h1111204うん、顔色もよくなったわね。\\n\
          ",
        render_room_item(&TalkingPlace::GuestRoom),
        night_table_bind_script(None),
        TalkingPlace::DEFAULT_LIVING_ROOM.balloon_surface_sakura(),
      )
    }
    (TalkingPlace::Kitchen, _) | (_, TalkingPlace::Kitchen) => unreachable!(), // TODO
    (TalkingPlace::LivingRoom(_), TalkingPlace::LivingRoom(_)) | (TalkingPlace::GuestRoom, TalkingPlace::GuestRoom) => unreachable!(),
  };
  new_response_with_value_with_translate(m, TranslateOption::simple_translate())
}

pub(crate) fn on_changing_user_name(_req: &Request) -> Result<Response, ShioriError> {
  new_response_with_value_with_translate(
    format!(
      "\\_q\\![open,inputbox,{},0]新しい呼び名を入力してください。\\n現在:{}",
      InputId::UserName,
      *get_read(&USER_NAME)
    ),
    TranslateOption::with_shadow_completion(),
  )
}

pub(crate) fn on_derivative_talk_request_button_toggled(req: &Request) -> Response {
  let is_derivative_talks_enabled;
  {
    is_derivative_talks_enabled = *get_read(&DERIVATIVE_TALK_REQUESTABLE);
  }
  *get_write(&DERIVATIVE_TALK_REQUESTABLE) = !is_derivative_talks_enabled;

  on_config_menu_exec(req)
}

pub(crate) fn on_story_event(req: &Request) -> Result<Response, ShioriError> {
  let refs = get_references(req);
  let s = if let Some(hoge) = PendingEvent::from_str(refs[0]) {
    let callback = || {
      *get_write(&PENDING_EVENT_TALK) = None;
    };
    match hoge {
      PendingEvent::ConfessionOfSuicide => {
        error!("Unexpected ConfessionOfSuicide");
        return Err(ShioriError::InvalidEvent);
      }
      PendingEvent::UnlockingLoreTalks => {
        // 解放メッセージの有無を判定するため、トーク生成後にフラグを立てる
        let s = unlock_lore_talks();
        get_write(&FLAGS).done(EventFlag::TalkTypeUnlock(TalkType::Lore));
        callback();
        s
      }
      PendingEvent::UnlockingServantsComments => {
        // 同上
        let s = unlock_servents_comments();
        get_write(&FLAGS).done(EventFlag::TalkTypeUnlock(TalkType::Servant));
        callback();
        s
      }
      _ => {
        error!("Unexpected pending event: {:?}", hoge);
        return Err(ShioriError::InvalidEvent);
      }
    }
  } else {
    return Err(ShioriError::InvalidEvent);
  };
  new_response_with_value_with_translate(s, TranslateOption::with_shadow_completion())
}

pub fn on_story_history_menu(_req: &Request) -> Response {
  let mut events = vec![("初回起動".to_string(), PendingEvent::FirstBoot, true)];
  for (i, _event) in FIRST_RANDOMTALKS.iter().enumerate() {
    events.push((
      format!("初回ランダムトーク{}/{}", i + 1, FIRST_RANDOMTALKS.len()),
      PendingEvent::FirstRandomTalk(i as u32),
      true,
    ));
  }
  events.push((
    "初回終了".to_string(),
    PendingEvent::FirstClose,
    get_read(&FLAGS).check(&EventFlag::FirstClose),
  ));
  events.push((
    "ロアトーク開放".to_string(),
    PendingEvent::UnlockingLoreTalks,
    get_read(&FLAGS).check(&EventFlag::TalkTypeUnlock(TalkType::Lore)),
  ));
  events.push((
    "従者コメント開放".to_string(),
    PendingEvent::UnlockingServantsComments,
    get_read(&FLAGS).check(&EventFlag::TalkTypeUnlock(TalkType::Servant)),
  ));

  let mut m = "\\_q\\b[2]イベント回想\\n\\n".to_string();
  for event in events {
    if event.2 {
      m.push_str(&format!(
        "\\![*]\\q[{},OnStoryHistoryExec,{}]\\n",
        event.0, event.1
      ));
    } else {
      m.push_str("\\![*]？？？\\n");
    }
  }
  m.push_str("\\n\\q[戻る,OnMenuExec]");
  new_response_with_value_with_notranslate(m, TranslateOption::none())
}

pub fn on_story_history_exec(req: &Request) -> Result<Response, ShioriError> {
  let refs = get_references(req);
  let s = if let Some(hoge) = PendingEvent::from_str(refs[0]) {
    match hoge {
      PendingEvent::FirstBoot => (FIRST_BOOT_TALK.clone(), TranslateOption::simple_translate()),
      PendingEvent::FirstRandomTalk(n) => (
        FIRST_RANDOMTALKS[n as usize].clone(),
        TranslateOption::simple_translate(),
      ),
      PendingEvent::FirstClose => (
        FIRST_CLOSE_TALK.to_string(),
        TranslateOption::simple_translate(),
      ),
      PendingEvent::UnlockingLoreTalks => (
        unlock_lore_talks(),
        TranslateOption::with_shadow_completion(),
      ),
      PendingEvent::UnlockingServantsComments => (
        unlock_servents_comments(),
        TranslateOption::with_shadow_completion(),
      ),
      _ => {
        return Err(ShioriError::InvalidEvent);
      }
    }
  } else {
    return Err(ShioriError::InvalidEvent);
  };
  new_response_with_value_with_translate(s.0, s.1)
}

fn unlock_lore_talks() -> String {
  format!(
    "\
      h1111201死について。深く考えることはある？\\n\
      h1111206……あなたには聞くまでもないわよね。\\n\
      h1111205私もそうなの。\\n\
      生きていたころから、なぜ生きるのか、\\n\
      死ぬとはどういうことかをずっと考えていたわ。\\n\
      いくつか不思議な話を知っているの。\\n\
      話の種に、語ってみましょうか。{}\
      ",
    if !get_read(&FLAGS).check(&EventFlag::TalkTypeUnlock(TalkType::Lore)) {
      render_achievement_message(TalkType::Lore)
    } else {
      "".to_string()
    },
  )
}

fn unlock_servents_comments() -> String {
  format!(
    "\
      \\1……h1111101\\1お茶がなくなってしまった。\\n\
      最初にハイネに言われたのを思いだし、\\n\
      部屋の隅に向って手を上げてみせる。\\n\
      h1111204\\1するとポットが浮き上がり、\\n\
      空になっていたカップにお茶が注がれた。\\n\
      \\0……h1111206彼らは私のことを「主」と呼ぶの。\\n\
      契約関係としては対等なのだけれど、\\n\
      彼ら自身がそう呼ぶのを好むのよ。\\n\
      \\n\
      h1111209耳を澄ませていれば、\\n\
      彼らの声が聞こえることもあるんじゃない？\\n\
      私を通して彼らとも縁ができているはずだから。{}\
      ",
    if !get_read(&FLAGS).check(&EventFlag::TalkTypeUnlock(TalkType::Servant)) {
      render_achievement_message(TalkType::Servant)
    } else {
      "".to_string()
    },
  )
}
