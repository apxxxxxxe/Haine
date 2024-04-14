use crate::events::aitalk::{
  random_talks, register_talk_collection, Talk, TalkType, FIRST_RANDOMTALKS,
};
use crate::events::common::*;
use crate::events::tooltip::show_tooltip;
use crate::variables::{get_global_vars, EventFlag};
use rand::seq::SliceRandom;
use shiorust::message::{Request, Response};

pub fn on_menu_exec(_req: &Request) -> Response {
  if !get_global_vars()
    .flags()
    .check(&EventFlag::FirstRandomTalkDone(
      (FIRST_RANDOMTALKS.len() - 1) as u32,
    ))
  {
    return on_menu_when_first_boot();
  }

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
    \\_l[0,1.5em]\
    \\![*]\\q[なにか話して,OnAiTalk]\\n\
    \\![*]\\q[話しかける,OnTalk]\\n\
    \\![*]\\q[ひと息つく,OnBreakTime]\\n\
    \\![*]\\q[トーク統計,OnCheckTalkCollection]\
    \\_l[0,@1.75em]\
    \\![*]\\q[手紙を書く,OnWebClapOpen]\
    \\_l[0,@1.75em]\
    {}
    \\_l[0,11em]\\__q[script:\\e]{}\\__q\
    \\_l[0,0]{}\
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

  new_response_with_value(m, TranslateOption::balloon_surface_only())
}

fn on_menu_when_first_boot() -> Response {
  let m = format!(
    "\
  \\_q\
  \\_l[0,3em]\\![*]\\q[話の続き,OnAiTalk]\\n\
  \\_l[0,11em]\\__q[script:\\e]{}\\__q\
  ",
    Icon::Cross,
  );

  new_response_with_value(m, TranslateOption::balloon_surface_only())
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

pub fn on_break_time(_req: &Request) -> Response {
  let m = "\
      h1111101\\1……少し話に集中しすぎていたようだ。\\n\
      h1111204\\1カップを傾け、一息つく。\\n\
      h1113705\\1ハイネはこちらの意図を察して、同じように一口飲んだ。\\n\
      \\n\
      \\![embed,OnImmersiveRateReduced]\
      "
  .to_string();

  new_response_with_value(m, TranslateOption::with_shadow_completion())
}

pub fn on_immersive_rate_reduced(_req: &Request) -> Response {
  // 没入度を下げる
  let vars = get_global_vars();
  let current_immersive_degrees = vars.volatility.immersive_degrees();
  vars
    .volatility
    .set_immersive_degrees((current_immersive_degrees / 2).saturating_sub(1));

  let m = "\
  \\Ch1111210\\1(没入度が半減した)\\x\\![raise,OnMenuExec]\
  "
  .to_string();

  new_response_with_value(m, TranslateOption::with_shadow_completion())
}

pub fn on_talk_interval_changed(req: &Request) -> Response {
  let refs = get_references(req);
  let v = refs[0].parse::<u64>().unwrap();
  get_global_vars().set_random_talk_interval(Some(v));

  on_menu_exec(req)
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

pub fn on_talk(_req: &Request) -> Response {
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

  new_response_with_value(m, TranslateOption::with_shadow_completion())
}

pub fn on_talk_answer(req: &Request) -> Response {
  let refs = get_references(req);
  let q = Question(refs[0].parse::<u32>().unwrap());
  new_response_with_value(q.talk(), TranslateOption::with_shadow_completion())
}

pub fn on_check_talk_collection(_req: &Request) -> Response {
  let mut s = String::new();
  let mut sum = 0;
  let mut all_sum = 0;
  let talk_collection = get_global_vars().talk_collection_mut();
  for talk_type in TalkType::all() {
    let len = talk_collection.get(&talk_type).map_or(0, |v| v.len());
    let all_len = random_talks(talk_type).len();
    let anal = if len < all_len {
      format!(
        "\\q[未読トーク再生,OnCheckUnseenTalks,{}]",
        talk_type as u32
      )
    } else {
      "".to_string()
    };
    s.push_str(&format!("{}: {}/{} {}\\n", talk_type, len, all_len, anal));
    sum += len;
    all_sum += all_len;
  }

  new_response_with_value(
    format!(
      "\\_q{}\
    ---\\n\
    TOTAL: {}/{}\\n\
    \\q[戻る,OnMenuExec]",
      s, sum, all_sum
    ),
    TranslateOption::balloon_surface_only(),
  )
}

pub fn on_check_unseen_talks(req: &Request) -> Response {
  let refs = get_references(req);
  let talk_type = TalkType::from_u32(refs[0].parse::<u32>().unwrap()).unwrap();
  let talk_collection = get_global_vars().talk_collection();
  let seen_talks = talk_collection.get(&talk_type).unwrap();

  let talks = Talk::get_unseen_talks(talk_type, seen_talks);
  let choosed_talk = talks.choose(&mut rand::thread_rng()).unwrap().to_owned();

  register_talk_collection(&choosed_talk);

  new_response_with_value(choosed_talk.text, TranslateOption::with_shadow_completion())
}
