use crate::events::common::*;
use crate::variables::get_global_vars;
use shiorust::message::{Response, *};

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

fn show_bar(max: u32, current: u32, label: &str, tooltip_id: &str) -> String {
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

  let m = format!(
    "\\_q\
    \\_l[0,3em]\
    \\![*]\\q[なにか話して,OnAiTalk]\\n\
    \\![*]\\q[話しかける,OnTalk]\\n\
    \\![*]\\q[ひと息つく,OnBreakTime]\\n\
    \\n\
    \\![*]\\q[手紙を書く,OnWebClapOpen]\\n\\n\
    {}
    \\_l[0,13em]\\__q[script:\\e]{}\\__q\
    \\_l[0,1em]{}\
    ",
    talk_interval_selector,
    Icon::Cross,
    show_bar(
      100,
      get_global_vars().volatility.immersive_degrees(),
      "没入度",
      "WhatIsImersiveDegree",
    ),
  );

  new_response_with_value(m, true)
}

pub fn on_break_time(_req: &Request) -> Response {
  // 没入度を下げる
  let vars = get_global_vars();
  let current_immersive_degrees = vars.volatility.immersive_degrees();
  vars
    .volatility
    .set_immersive_degrees((current_immersive_degrees / 2).saturating_sub(1));

  let m = "\
      h1111101\\1……少し話に集中しすぎていたようだ。\\n\
      h1111204\\1カップを傾け、一息つく。\\n\
      h1000000\\1ハイネはこちらの意図を察して、同じように一口飲んだ。\\n\
      h1111209\\1\\n(没入度が下がった)\\x\\![raise,OnMenuExec]\
      "
  .to_string();

  new_response_with_value(m, true)
}

pub fn show_tooltip(id: &str) -> String {
  format!(
    "\\__q[OnBalloonTooltip,{}]\
    {}
    \\__q",
    id,
    Icon::Info,
  )
}

pub fn on_balloon_tooltip(_req: &Request) -> Response {
  new_response_with_value("\\C\\_l[0,0] ".to_string(), false)
}

pub fn balloon_tooltip(req: &Request) -> Response {
  let refs = get_references(req);
  if refs[1] != "OnBalloonTooltip" {
    return new_response_nocontent();
  }
  match refs[2] {
    "WhatIsImersiveDegree" => new_response_with_value(
      "没入度は、ハイネとあなたがどれだけ深い内容の会話をしているかを表します。\\n\
                                  没入度が高いほど、より抽象的でクリティカルな話題を扱います。"
        .to_string(),
      false,
    ),
    _ => new_response_nocontent(),
  }
}

pub fn on_web_clap_open(_req: &Request) -> Response {
  let m = "\
             \\1\\![open,inputbox,OnWebClapInput,0]Web拍手を送ります。\\n\
             感想やバグ報告、要望などをお送り下さい。\
             "
  .to_string();
  new_response_with_value(m, true)
}

pub fn on_talk_interval_changed(req: &Request) -> Response {
  let refs = get_references(req);
  let v = refs[0].parse::<u64>().unwrap();
  get_global_vars().set_random_talk_interval(Some(v));

  on_menu_exec(req)
}

#[derive(Copy, Clone)]
enum Question {
  AreYouMaster,
  FeelingOfDeath,
  FatigueOfLife,
  HowTallAreYou,
}

impl Question {
  fn from_u32(v: u32) -> Option<Self> {
    match v {
      0 => Some(Question::AreYouMaster),
      1 => Some(Question::FeelingOfDeath),
      2 => Some(Question::FatigueOfLife),
      3 => Some(Question::HowTallAreYou),
      _ => None,
    }
  }

  fn theme(&self) -> String {
    match self {
      Question::AreYouMaster => "あなたはここの主なのφ？".to_string(),
      Question::FeelingOfDeath => "死んだ感想はφ？".to_string(),
      Question::FatigueOfLife => "生きるのって苦しいね".to_string(),
      Question::HowTallAreYou => "身長はどれくらい？".to_string(),
    }
  }

  fn to_script(self) -> String {
    format!("\\![*]\\q[{},OnTalkAnswer,{}]", self.theme(), self as u32)
  }

  fn talk(&self) -> String {
    let m = match self {
      Question::AreYouMaster => "\
      \\1幽霊なのに、あなたはここの主なの？\\n\
      h1111204ええ、そうよ。\\n\
      私が、私だけが、この家の主なの。\
      "
      .to_string(),
      Question::FeelingOfDeath => "\
      h1111104\\1幽霊ということは、一度死んだんだよね？\\n\
      どんな感じだった？何か思うことはある？\
      h1111204別に、何も。\\n\
      私の求める変化はそこには無いし、何より私はまだ死ねていない。\\n\
      自我を手放してこその死でしょう？h1111205だから、これからよ。\\n\
      "
      .to_string(),
      Question::FatigueOfLife => "\
      \\1生きるのは苦しい。どうしていいかわからない。\\n\
      h1111205そう、そうね。\\n\
      …………1111205悪いけれど、私はその答えを持っていない。\\n\
      h1111204あなたが満足できるまで話を聞くわ。それから、どうするかを決めなさい。\
      "
      .to_string(),
      Question::HowTallAreYou => "\
      \\1身長はどれくらい？\\n\
      h1111204まず前提として、霊は身体を自由に変化させることができるわ。\\n\
      h1111209子供になることも、老人になることも、\\n\
      h1111206人でない姿になることすら不可能ではないの。\\n\
      ……h1111204それを踏まえて。今の姿の身長は、おおよそ1.75mね。\
      "
      .to_string(),
    };
    m + "\\x\\![raise,OnTalk]"
  }
}

pub fn on_talk(_req: &Request) -> Response {
  let mut questions = [
    Question::AreYouMaster,
    Question::FeelingOfDeath,
    Question::FatigueOfLife,
  ];

  let mut m = "".to_string();
  for q in questions.iter_mut() {
    m.push_str(&q.to_script());
    m.push_str("\\n");
  }
  m.push_str("\\q[戻る,OnMenuExec]");

  new_response_with_value(m, true)
}

pub fn on_talk_answer(req: &Request) -> Response {
  let refs = get_references(req);
  let q = Question::from_u32(refs[0].parse::<u32>().unwrap()).unwrap();
  new_response_with_value(q.talk(), true)
}
