use crate::events::common::*;
use crate::variables::get_global_vars;
use shiorust::message::{Response, *};

fn show_minute(m: &u64) -> String {
  match m {
    0 => "黙る".to_string(),
    _ => format!("{}分", m),
  }
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
    \\_l[0,1em]\
    \\![*]\\q[なにか話して,OnAiTalk]\\n\\n\
    \\![*]\\q[話しかける,OnTalk]\\n\\n\
    {}
    \\_l[0,12em]\\q[×,]\
    ",
    talk_interval_selector
  );

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
}

impl Question {
  fn from_u32(v: u32) -> Option<Self> {
    match v {
      0 => Some(Question::AreYouMaster),
      1 => Some(Question::FeelingOfDeath),
      2 => Some(Question::FatigueOfLife),
      _ => None,
    }
  }

  fn theme(&self) -> String {
    match self {
      Question::AreYouMaster => "あなたはここの主なの？".to_string(),
      Question::FeelingOfDeath => "死んだ感想は？".to_string(),
      Question::FatigueOfLife => "生きるのって苦しいね".to_string(),
    }
  }

  fn to_script(&self) -> String {
    format!("\\![*]\\q[{},OnTalkAnswer,{}]", self.theme(), *self as u32)
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
    };
    m + "\\x\\![raise,OnTalk]"
  }
}

pub fn on_talk(_req: &Request) -> Response {
  let questions = [
    Question::AreYouMaster,
    Question::FeelingOfDeath,
    Question::FatigueOfLife,
  ];

  let mut m = "".to_string();
  for q in questions.iter() {
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
