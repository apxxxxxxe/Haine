use crate::check_error;
use crate::error::ShioriError;
use crate::events::common::*;
use crate::events::first_boot::FIRST_RANDOMTALKS;
use crate::events::input::InputId;
use crate::events::talk::randomtalk::random_talks;
use crate::events::tooltip::show_tooltip;
use crate::events::TalkingPlace;
use crate::variables::{get_global_vars, EventFlag};
use shiorust::message::{Request, Response};

pub fn on_menu_exec(_req: &Request) -> Response {
  let current_talk_interval = get_global_vars().random_talk_interval().unwrap_or(180);
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

  let close_button = format!(
    "\\_l[0,0]\\f[align,right]\\__q[script:\\e]{}\\__q",
    Icon::Cross
  );
  let vars = get_global_vars();
  let m = format!(
    "\\_q{}{}",
    REMOVE_BALLOON_NUM,
    if !get_global_vars()
      .flags()
      .check(&EventFlag::FirstRandomTalkDone(
        (FIRST_RANDOMTALKS.len() - 1) as u32,
      ))
    {
      "\
      \\_l[0,3em]\\![*]\\q[話の続き,OnAiTalk]\\n[150]\
      \\![*]\\q[その名前で呼ばれたくない,OnChangingUserName]\\n\
      "
      .to_string()
        + &close_button
    } else {
      format!(
        "\
        \\_l[0,1.5em]\
        \\![*]\\q[なにか話して,OnAiTalk]\\n\
        {}\
        \\![*]\\q[トーク統計,OnCheckTalkCollection]\
        \\_l[0,@1.75em]\
        \\![*]\\q[手紙を書く,OnWebClapOpen]\
        \\_l[0,@1.75em]\
        \\![*]\\q[呼び名を変える,OnChangingUserName]\\n\
        {}\
        {}\
        \\_l[0,0]{}\
        ",
        if vars.volatility.talking_place() == TalkingPlace::Library {
          "".to_string()
        } else {
          "\\![*]\\q[話しかける,OnTalk]\\n".to_string()
        },
        talk_interval_selector,
        close_button,
        if vars.volatility.talking_place() == TalkingPlace::Library {
          show_bar_with_simple_label(
            100,
            vars.volatility.immersive_degrees(),
            "ハイネは応えない。",
          )
        } else {
          show_bar_with_caption(
            100,
            vars.volatility.immersive_degrees(),
            "没入度",
            "WhatIsImersiveDegree",
          )
        },
      )
    },
  );

  new_response_with_value_with_notranslate(m, TranslateOption::balloon_surface_only())
}

fn show_minute(m: &u64) -> String {
  match m {
    0 => "黙る".to_string(),
    _ => format!("{}分", m),
  }
}

fn make_bar_chips(length: u32) -> Vec<String> {
  let mut v = Vec::new();
  if length == 0 {
    return v;
  }
  v.push("●".to_string());
  for _i in 1..length - 1 {
    v.push("■".to_string());
  }
  v.push("●".to_string());
  v
}

fn show_bar_with_caption(max: u32, current: u32, label: &str, tooltip_id: &str) -> String {
  const SPEED: u32 = 10; // 何文字分の表示時間でバーを描画するか
  const BAR_WIDTH: u32 = 16; // バーの長さを何文字分で描画するか
  const BAR_HEIGHT: u32 = 10;
  let rate = ((current * 100) as f32 / max as f32) as u32;
  let bar_width = BAR_WIDTH * 2; // 重ねながら描画するので2倍
  let bar_chip_wait = (50.0 * ((SPEED as f32) / (BAR_WIDTH as f32))) as u32; // 1文字分の表示時間

  format!(
    "\
    \\f[height,{}]\\_l[{}em,]\\f[height,default]\\![quicksection,true]{}: {}%\\_l[@5,]{}\
    \\f[height,{}]\\_l[0,@3]\\f[color,80,80,80]{}\
    \\f[height,{}]\\_l[0,]\\f[color,120,0,0]{}\
    \\![quicksection,false]\\f[color,default]\\f[height,default]\
    ",
    BAR_HEIGHT,
    BAR_WIDTH + 1,
    label,
    rate,
    show_tooltip(tooltip_id),
    BAR_HEIGHT,
    make_bar_chips(bar_width).join("\\_l[@-0.5em,]"),
    BAR_HEIGHT,
    make_bar_chips(bar_width * rate / 100).join(&format!("\\_w[{}]\\_l[@-0.5em,]", bar_chip_wait)),
  )
}

fn show_bar_with_simple_label(max: u32, current: u32, label: &str) -> String {
  const SPEED: u32 = 10; // 何文字分の表示時間でバーを描画するか
  const BAR_WIDTH: u32 = 16; // バーの長さを何文字分で描画するか
  const BAR_HEIGHT: u32 = 10;
  let rate = ((current * 100) as f32 / max as f32) as u32;
  let bar_width = BAR_WIDTH * 2; // 重ねながら描画するので2倍
  let bar_chip_wait = (50.0 * ((SPEED as f32) / (BAR_WIDTH as f32))) as u32; // 1文字分の表示時間

  format!(
    "\
    \\f[height,{}]\\_l[{}em,]\\f[height,default]\\f[height,-1]\\![quicksection,true]{}\
    \\f[height,{}]\\_l[0,@3]\\f[color,80,80,80]{}\
    \\f[height,{}]\\_l[0,]\\f[color,120,0,0]{}\
    ",
    BAR_HEIGHT,
    BAR_WIDTH + 1,
    label,
    BAR_HEIGHT,
    make_bar_chips(bar_width).join("\\_l[@-0.5em,]"),
    BAR_HEIGHT,
    make_bar_chips(bar_width * rate / 100).join(&format!("\\_w[{}]\\_l[@-0.5em,]", bar_chip_wait)),
  )
}

pub fn on_talk_interval_changed(req: &Request) -> Result<Response, ShioriError> {
  let refs = get_references(req);
  let v = check_error!(refs[0].parse::<u64>(), ShioriError::ParseIntError);
  get_global_vars().set_random_talk_interval(Some(v));

  Ok(on_menu_exec(req))
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Question(u32);

impl Question {
  const HOW_OLD_ARE_YOU: Self = Self(0);
  const HOW_TALL_ARE_YOU: Self = Self(1);
  const HOW_WEIGHT_ARE_YOU: Self = Self(2);
  const HOW_MUCH_IS_YOUR_BWH: Self = Self(3);
  const ARE_YOU_MASTER: Self = Self(4);
  const FEELING_OF_DEATH: Self = Self(5);
  const FATIGUE_OF_LIFE: Self = Self(6);

  fn theme(&self) -> String {
    match *self {
      Question::HOW_OLD_ARE_YOU => "何歳？".to_string(),
      Question::HOW_TALL_ARE_YOU => "身長はどれくらい？".to_string(),
      Question::HOW_WEIGHT_ARE_YOU => "体重は？".to_string(),
      Question::HOW_MUCH_IS_YOUR_BWH => "スリーサイズを教えて".to_string(),
      Question::ARE_YOU_MASTER => "あなたはここの主なの？".to_string(),
      Question::FEELING_OF_DEATH => "死んだ感想は？".to_string(),
      Question::FATIGUE_OF_LIFE => "生きるのって苦しいね".to_string(),
      _ => unreachable!(),
    }
  }

  fn to_script(self) -> String {
    format!("\\![*]\\__q[OnTalkAnswer,{}]{}\\__q", self.0, self.theme())
  }

  fn talk(&self) -> String {
    let m = match *self {
      Question::ARE_YOU_MASTER => "\
      \\1幽霊なのに、あなたはここの主なの？\\n\
      h1111204ええ、そうよ。\\n\
      私が、私だけが、この家の主なの。\
      "
      .to_string(),
      Question::FEELING_OF_DEATH => "\
      h1111104\\1幽霊ということは、一度死んだんだよね？\\n\
      どんな感じだった？何か思うことはある？\
      h1111204別に、何も。\\n\
      私の求める変化はそこには無いし、何より私はまだ死ねていない。\\n\
      自我を手放してこその死でしょう？h1111205だから、これからよ。\\n\
      "
      .to_string(),
      Question::FATIGUE_OF_LIFE => "\
      \\1生きるのは苦しい。どうしていいかわからない。\\n\
      h1111205そう、そうね。\\n\
      …………1111205悪いけれど、私はその答えを持っていない。\\n\
      h1111204あなたが満足できるまで話を聞くわ。それから、どうするかを決めなさい。\
      "
      .to_string(),
      Question::HOW_TALL_ARE_YOU => "\
      \\1身長はどれくらい？\\n\
      h1111204今の姿ならおおよそ175cmね。\\n\
      ……h1111210今の、と言ったのは、\\n\
      私たち霊は身体を自由に変化させることができるから。\\n\
      子供になることも、老人になることも、\\n\
      h1111206人でない姿になることすら不可能ではないの。\\n\
      h1111204……まあ、生前はそのくらいだったと思ってくれていいわ。\
      "
      .to_string(),
      Question::HOW_WEIGHT_ARE_YOU => "\
      \\1体重は？\\n\
      h1111201……霊体に重さはないわ。\\n\
      h1111204……知りたいのはそういうことではないって？\\n\
      h1111210まあ、そうでしょうね。\\n\
      h1111205……55kgだったかしら。もう定かではないけれど。\
      "
      .to_string(),
      Question::HOW_MUCH_IS_YOUR_BWH => "\
      \\1スリーサイズを教えて。\\n\
      h1111601…………h1111201さっきから随分と果断ね。\\n\
      h1111204怒られるかもとか考えないのかしら。\\n\
      h1111205……79・56・81。\\n\
      ……h1111210何に使うか知らないけれど。\
      "
      .to_string(),
      Question::HOW_OLD_ARE_YOU => "\
      \\1何歳？\\n\
      h1141604……h1111204女性に年齢を聞くなんて。\\n\
      ……h1111205死んだ時は26よ。\\n\
      死んでからは……h1111511教えてあげない。\
      "
      .to_string(),
      _ => unreachable!(),
    };
    m + "\\x\\![raise,OnTalk]"
  }
}

/*
\\1どうして"主"になったの？\\n\
h1111210……。叔父がいたの。\\n\
祖父に似て学問が好きでね、私もずいぶんと可愛がってもらったわ。\\n\
身内の中では私が一番のお気に入りだったみたい。\\n\
私も、何でも教えてくれる叔父のことが大好きだった。\\n\
\\n\
叔父が変わったのは、彼の肺に病が見つかった時。\\n\
死の影に耐えられなかったのでしょう。\\n\
科学の徒だった彼は、次第にオカルトに傾倒していった。\\n\
私も頑固だったわ。彼の気持ちを理解しないまま、ひどくあたってしまった。\\n\
……彼は、意趣返しのつもりだったのでしょうね。\\n\
それとも、本気でこの結末を想像していたのかしら。\\n\
今となっては、ね。\
*/

pub fn on_talk(_req: &Request) -> Result<Response, ShioriError> {
  let mut questions = [
    Question::ARE_YOU_MASTER,
    Question::FEELING_OF_DEATH,
    Question::FATIGUE_OF_LIFE,
    Question::HOW_TALL_ARE_YOU,
    Question::HOW_WEIGHT_ARE_YOU,
    Question::HOW_MUCH_IS_YOUR_BWH,
    Question::HOW_OLD_ARE_YOU,
  ];
  questions.sort_by(|a, b| a.0.cmp(&b.0));

  let mut m = "\\_q".to_string();
  for q in questions.iter_mut() {
    m.push_str(&q.to_script());
    m.push_str("\\n");
  }
  m.push_str("\\q[戻る,OnMenuExec]");

  new_response_with_value_with_translate(m, TranslateOption::with_shadow_completion())
}

pub fn on_talk_answer(req: &Request) -> Result<Response, ShioriError> {
  let refs = get_references(req);
  let q = Question(check_error!(
    refs[0].parse::<u32>(),
    ShioriError::ParseIntError
  ));
  new_response_with_value_with_translate(q.talk(), TranslateOption::with_shadow_completion())
}

pub fn on_check_talk_collection(_req: &Request) -> Response {
  let mut lines = Vec::new();
  let mut sum = 0;
  let mut all_sum = 0;
  const DIMMED_COLOR: &str = "\\f[color,150,150,130]";
  let talk_collection = get_global_vars().talk_collection_mut();
  let vars = get_global_vars();
  let talking_place = vars.volatility.talking_place();
  lines.push(format!("[トーク統計: {}]\\n", talking_place));
  let talk_types = talking_place.talk_types();
  let is_unlocked_checks = talk_types
    .iter()
    .map(|t| vars.flags().check(&EventFlag::TalkTypeUnlock(*t)))
    .collect::<Vec<_>>();
  for i in 0..talk_types.len() {
    let talk_type = talk_types[i];
    if !is_unlocked_checks[i] {
      lines.push(format!("{}{}: 未解放\\f[default]", DIMMED_COLOR, talk_type));
    } else {
      let len = talk_collection.get(&talk_type).map_or(0, |v| v.len());
      let all_len = if let Some(v) = random_talks(talk_type) {
        v.len()
      } else {
        0
      };
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

pub fn on_changing_user_name(_req: &Request) -> Result<Response, ShioriError> {
  let vars = get_global_vars();
  let user_name = if let Some(user_name) = vars.user_name() {
    user_name
  } else {
    error!("User name is not set.");
    return Ok(new_response_nocontent());
  };
  new_response_with_value_with_translate(
    format!(
      "\\_q\\![open,inputbox,{},0]新しい呼び名を入力してください。\\n現在:{}",
      InputId::UserName,
      user_name,
    ),
    TranslateOption::with_shadow_completion(),
  )
}
