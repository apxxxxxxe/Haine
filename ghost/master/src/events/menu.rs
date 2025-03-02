use crate::check_error;
use crate::error::ShioriError;
use crate::events::common::*;
use crate::events::first_boot::FIRST_RANDOMTALKS;
use crate::events::input::InputId;
use crate::events::talk::randomtalk::random_talks;
use crate::events::TalkType;
use crate::events::TalkingPlace;
use crate::events::IMMERSIVE_RATE_MAX;
use crate::variables::PendingEvent;
use crate::variables::{get_global_vars, EventFlag};
use shiorust::message::{Request, Response};

pub(crate) fn on_menu_exec(_req: &Request) -> Response {
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
        \\![*]\\q[{},OnAiTalk]\\n\
        {}\
        \\![*]\\q[トーク統計,OnCheckTalkCollection]\\n\
        \\_l[0,@1.75em]\
        \\![*]\\q[手紙を書く,OnWebClapOpen]\
        \\_l[0,@1.75em]\
        \\![*]\\q[呼び名を変える,OnChangingUserName]\\n\
        {}\
        {}\
        \\1{}\
        \\0\\_l[0,0]\
        ",
        if vars.volatility.talking_place() == TalkingPlace::Library {
          "耳を澄ます"
        } else {
          "なにか話して"
        },
        if vars.volatility.talking_place() == TalkingPlace::Library {
          "".to_string()
        } else {
          "\\![*]\\q[話しかける,OnTalk]\\n".to_string()
        },
        talk_interval_selector,
        close_button,
        if let Some(event) = vars.pending_event_talk() {
          format!("\\![*]\\q[{},OnStoryEvent,{}]", event, event)
        } else {
          "".to_string()
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

pub(crate) fn on_talk_interval_changed(req: &Request) -> Result<Response, ShioriError> {
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
  const FEELING_OF_DEATH: Self = Self(5);
  const FATIGUE_OF_LIFE: Self = Self(6);
  const HOW_TO_GET_TEALEAVES: Self = Self(7);
  const DO_SERVENTS_HAVE_NAMES: Self = Self(8);
  const CALL_YOU_MASTER: Self = Self(9);
  const WHAT_DO_YOU_DO_WHEN_YOU_ARE_ALONE: Self = Self(10); // ひとりのときは何をして過ごしてる？
                                                            // 今日泊まってもいい？
  const IS_THERE_A_PLACE_TO_VISIT: Self = Self(12); // このあたりに観光できる場所はある？
                                                    // かわいい
                                                    // 美人
                                                    // 似顔絵を描く
                                                    // お腹が空いた

  fn theme(&self) -> String {
    match *self {
      Question::HOW_OLD_ARE_YOU => "何歳？".to_string(),
      Question::HOW_TALL_ARE_YOU => "身長はどれくらい？".to_string(),
      Question::HOW_WEIGHT_ARE_YOU => "体重は？".to_string(),
      Question::HOW_MUCH_IS_YOUR_BWH => "スリーサイズを教えて".to_string(),
      Question::FEELING_OF_DEATH => "死んだ感想は？".to_string(),
      Question::FATIGUE_OF_LIFE => "生きるのって苦しいね".to_string(),
      Question::HOW_TO_GET_TEALEAVES => "お茶はどこから手に入れているの？".to_string(),
      Question::DO_SERVENTS_HAVE_NAMES => "従者たちに名前はあるの？".to_string(),
      Question::CALL_YOU_MASTER => "ご主人様".to_string(),
      Question::WHAT_DO_YOU_DO_WHEN_YOU_ARE_ALONE => {
        "ひとりのときは何をして過ごしてる？".to_string()
      }
      Question::IS_THERE_A_PLACE_TO_VISIT => "このあたりに観光できる場所はある？".to_string(),
      _ => unreachable!(),
    }
  }

  fn to_script(self) -> String {
    format!("\\![*]\\__q[OnTalkAnswer,{}]{}\\__q", self.0, self.theme())
  }

  fn talk(&self) -> String {
    let m = match *self {
      Question::FEELING_OF_DEATH => "\
      h1111104\\1『幽霊ということは、一度死んだんだよね？\\n\
      どんな感じだった？何か思うことはある？』\
      h1111204いいえ、何も。\\n\
      h1111205私の求める変化はそこには無いし、何より私はまだ死ねていない。\\n\
      自我を手放してこその死でしょう？\\n\
      h1111210だから、これからよ。\\n\
      "
      .to_string(),
      Question::FATIGUE_OF_LIFE => "\
      \\1『生きるのは苦しい。どうしていいかわからない』\\n\
      h1111205そう、そうね。\\n\
      …………h1111204悪いけれど、私はその答えを持っていない。\\n\
      \\n\
      h1111204あなたが満足できるまで話を聞くわ。\\n\
      h1111210それから、どうするかを自分で決めなさい。\
      "
      .to_string(),
      Question::HOW_TALL_ARE_YOU => "\
      \\1『身長はどれくらい？』\\n\
      h1111204おおよそ175cmね。\\n\
      ……h1111210おおよそ、と言ったのは、\\n\
      生前の身長だから。\\n\
      h1111206今の身長は、測っても無駄なの。\\n\
      霊体は常に揺らめいていて、大きさが変動し続ける。\\n\
      h1111310……まあ、平均的にはそのくらいだと思ってちょうだい。\
      "
      .to_string(),
      Question::HOW_WEIGHT_ARE_YOU => "\
      \\1『体重は？』\\n\
      h1111201……霊体に重さはないわ。\\n\
      h1111204……知りたいのはそういうことではないって？\\n\
      h1111210まあ、そうでしょうね。\\n\
      h1111205……55kgだったかしら。もう定かではないけれど。\
      "
      .to_string(),
      Question::HOW_MUCH_IS_YOUR_BWH => "\
      \\1『スリーサイズを教えて』\\n\
      h1111601…………h1111201さっきから随分と果断ね。\\n\
      h1111204怒られるかもとか考えないのかしら。\\n\
      h1111205……79・56・81。\\n\
      ……h1111210知ってどうするのか知らないけれど。\
      "
      .to_string(),
      Question::HOW_OLD_ARE_YOU => "\
      \\1『何歳？』\\n\
      h1141604……h1111204女性に年齢を聞くなんて。\\n\
      ……h1111205死んだ時は26よ。\\n\
      死んでからは……h1111511教えてあげない。\\n\
      "
      .to_string(),
      Question::HOW_TO_GET_TEALEAVES => "\
      \\1『お茶はどこから手に入れているの？』\\n\
      h1111206行商人がいるのよ。私と同じ、実体を持つ霊。\\n\
      h1111210それでいて場所に囚われない、稀有な存在よ。\\n\
      それに定期的なお使いを頼んでいるの。\\n\
      良い茶葉を扱う店に、買い物を。\\n\
      \\n\
      h1111205勿論、対価も払わなければならない。\\n\
      それは自由に動ける代わりに、長い休眠を必要とするの。\\n\
      h1111210取引をする者たちはあれが無防備な間、身の安全を保障する契約なのよ。\
      "
      .to_string(),
      Question::DO_SERVENTS_HAVE_NAMES => "\
      \\1『従者たちに名前はあるの？』\\n\
      h1111210ええ、もちろん。\\n\
      でも、教えることはできないわ。\\n\
      \\n\
      h1111206霊にとって、自分が何者であるかは文字通り死活問題なの。\\n\
      肉の器がない分、簡単に存在が揺らいでしまうから。\\n\
      h1111210必要なときは偽名や通り名を名乗り、\\n\
      真の名前は基本的には契約する相手にしか明かさないのよ。\\n\
      \\n\
      ……h1111204「寂しい」って思った？……h1111211ふふ。\\n\
      h1111204あなたに教えた私の名前は偽名ではないわ。\\n\
      私は低級霊ではないから、多少は構わないの。\\n\
      h1111210生者の時間を奪うことへの、せめてもの礼儀よ。\
      "
      .to_string(),
      Question::CALL_YOU_MASTER => "\
      \\1『ご主人様』\\n\
      h1111101……h1111210ふふ、従者の仲間入りがしたいの？\\n\
      \\n\
      h1111304いいえ、あなたは客人よ。\\n\
      あなたにとって私は、ただのハイネ。\\n\
      \\n\
      h1111205なにかに身を委ねるのは簡単だけれどね。\\n\
      自分の手綱は自分で握るものよ。\\n\
      h1111210自分の意志でここにいる。\\n\
      そんなあなただから、こうして一緒にいるのよ。\
      "
      .to_string(),
      Question::WHAT_DO_YOU_DO_WHEN_YOU_ARE_ALONE => "\
      \\1『ひとりのときは何をして過ごしてる？』\\n\
      h1111105……ひとりのときというと、仕事がないときね。\\n\
      h1111204大抵は書斎で本を読むか、私室でお茶を飲んで休むか、\\n\
      h1111206ああ、ここで従者たちの相手をすることもあるわね。\\n\
      彼らも仕事の息抜きを欲しているから、定期的にね。\\n\
      ……h1111105これはひとりのときではないか。\\n\
      h1111204まあ、好きなように過ごしているわ。\
      "
      .to_string(),
      Question::IS_THERE_A_PLACE_TO_VISIT => "\
      \\1『このあたりに観光できる場所はある？』\\n\
      h1113205そうね……h1113304あなた、史跡は好き？\\n\
      h1113206今でこそ寂れた田舎町だけれど、\\n\
      その昔、ここは学問の中心地だったのよ。\\n\
      ここから東に行ったところに修道院跡があるわ。\\n\
      h1113210大半は焼失してしまったけれど、\\n\
      名物だった鐘楼はかろうじて原型を残しているの。\\n\
      h1111205あとは……北に小さな劇場もあるわね。\\n\
      あれもかつては貴族の社交場だったのだけれど、\\n\
      今はもう、地元の劇団が使う程度ね。\\n\
      ……h1111210まあ、\\n\
      退屈しのぎには良いんじゃないかしら。\\n\
      \\n\
      ……そんなことを聞くなんて、\\n\
      h1111204私との語らいには飽きてしまったのかしら？\\n\
      ……h1111310冗談よ。\\n\
      h1111304あなたの目はそう言っていないものね。\
      "
      .to_string(),
      _ => unreachable!(),
    };
    m + "\\x\\![raise,OnTalk]"
  }
}

pub(crate) fn on_talk(_req: &Request) -> Result<Response, ShioriError> {
  let mut questions = [
    Question::FEELING_OF_DEATH,
    Question::FATIGUE_OF_LIFE,
    Question::HOW_TALL_ARE_YOU,
    Question::HOW_WEIGHT_ARE_YOU,
    Question::HOW_MUCH_IS_YOUR_BWH,
    Question::HOW_OLD_ARE_YOU,
    Question::HOW_TO_GET_TEALEAVES,
    Question::DO_SERVENTS_HAVE_NAMES,
    Question::CALL_YOU_MASTER,
    Question::WHAT_DO_YOU_DO_WHEN_YOU_ARE_ALONE,
    Question::IS_THERE_A_PLACE_TO_VISIT,
  ];
  questions.sort_by(|a, b| a.0.cmp(&b.0));

  let mut m = "\\_q\\b[2]".to_string();
  for q in questions.iter_mut() {
    m.push_str(&q.to_script());
    m.push_str("\\n");
  }
  m.push_str("\\n\\q[戻る,OnMenuExec]");

  new_response_with_value_with_translate(m, TranslateOption::with_shadow_completion())
}

pub(crate) fn on_talk_answer(req: &Request) -> Result<Response, ShioriError> {
  let refs = get_references(req);
  let q = Question(check_error!(
    refs[0].parse::<u32>(),
    ShioriError::ParseIntError
  ));
  new_response_with_value_with_translate(q.talk(), TranslateOption::with_shadow_completion())
}

pub(crate) fn on_check_talk_collection(_req: &Request) -> Response {
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

pub(crate) fn on_changing_user_name(_req: &Request) -> Result<Response, ShioriError> {
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

pub(crate) fn on_immersive_degree_toggled(req: &Request) -> Response {
  let vars = get_global_vars();
  vars
    .volatility
    .set_is_immersive_degrees_fixed(!vars.volatility.is_immersive_degrees_fixed());
  on_menu_exec(req)
}

pub(crate) fn on_story_event(req: &Request) -> Result<Response, ShioriError> {
  let refs = get_references(req);
  let vars = get_global_vars();
  let s = if let Some(hoge) = PendingEvent::from_str(refs[0]) {
    match hoge {
      PendingEvent::ConfessionOfSuicide => {
        vars
          .flags_mut()
          .done(EventFlag::TalkTypeUnlock(TalkType::Past));
        suicide_coming_out()
      }
      PendingEvent::UnlockingLoreTalks => {
        get_global_vars()
          .flags_mut()
          .done(EventFlag::TalkTypeUnlock(TalkType::Lore));
        unlock_lore_talks()
      }
      PendingEvent::UnlockingServantsComments => {
        get_global_vars()
          .flags_mut()
          .done(EventFlag::TalkTypeUnlock(TalkType::Servant));
        unlock_servents_comments()
      }
    }
  } else {
    return Err(ShioriError::InvalidEvent);
  };
  new_response_with_value_with_translate(s, TranslateOption::with_shadow_completion())
}

fn suicide_coming_out() -> String {
  let vars = get_global_vars();
  vars.volatility.set_talking_place(TalkingPlace::Library);
  vars.volatility.set_immersive_degrees(IMMERSIVE_RATE_MAX);
  let achievements_message = render_achievement_message(TalkType::Past);
  format!(
    "\
    h1111105\\b[{}]……。\\1ハイネ……？\\n\
    彼女が思索に耽っているときに来てしまったようだ。\\n\
    ……しばらくそっとしておこう……。\
    \\0\\c\\b[{}]\\1\\b[-1]\\ch1000000───────────────\\_w[1200]\\c\
    h1111110…………幽霊にとって、自身の死の記憶はある種のタブー。\\n\
    誰もが持つがゆえの共通認識。\\n\
    ……自身の死は恥部であると。\\n\
    私も、彼らのそれには深く踏み込まない。\\n\
    けれど、あの子は生者だから。\\n\
    \\n\
    いいえ、だからこそ\\n\
    打ち明けることに意味がある。\
    \\x\
    h1000000───────────────\\_w[1200]\\c\
    h1111105\\b[{}]……h1111101。\\n\
    \\1ハイネ……？\\n\
    \\0……{{user_name}}。\\n\
    少し、話があるの。すぐに済むわ。\\x\
    h1111106……私の過去について、\\n\
    今まで話してこなかったわね。\\n\
    h1111110あなたの過去を根掘り葉掘り聞いているくせに、\\n\
    私はなにも明かさないのでは不公平だと思ったの。\\n\
    今更といえば今更なのだけど。\\n\
    ……だらだらと話しても仕方ないから、一つだけ。\\n\
    \\x[noclear]\\n\
    h1111305私はかつて、自殺をしたの。\\n\
    \\1……。\\n\
    \\0苦しみを終わらせたかったの。あなたと同じね。\\n\
    h1111310けれど、運が悪かった。\\n\
    この場所で自らを殺して、\\n\
    ここに縛り付けられてしまった。\\n\
    ずっと待っているというのは、この身の消滅。\\n\
    h1111305今度こそ終わらせたいのよ。\\n\
    h1111705\\_w[600]\\1言い終わると、彼女は深く息をついた。\\n\
    \\0……h1121304どう思おうが、構わないわ。\\n\
    ただ、私だけ明かさないのは嫌だったの。\\n\
    \\n\
    h1121310……h1111204さて、私の話は終わり。\\n\
    h1111211時間を取らせて悪かったわね。\\n\
    h1111206すぐにお茶を入れさせるわ。\\n\
    語らいましょう、いつものように。\
    \\1\\c{}",
    TalkingPlace::LivingRoom.balloon_surface(),
    TalkingPlace::Library.balloon_surface(),
    TalkingPlace::LivingRoom.balloon_surface(),
    achievements_message
  )
}

fn unlock_lore_talks() -> String {
  format!(
    "\
    h1111201死について。深く考えることはある？\\n\
    h1111206……あなたには聞くまでもないようね。\\n\
    h1111205私もそう。\\n\
    生きていたころから、なぜ生きるのか、死ぬとはどういうことかをずっと考えていたわ。\\n\
    いくつか不思議な話を知っているの。\\n\
    話の種に、いくつか語ってみましょうか。{}\
    ",
    if !get_global_vars()
      .flags()
      .check(&EventFlag::TalkTypeUnlock(TalkType::Lore))
    {
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
    h1111204\\1するとポットが浮き上がり、空になっていたカップにお茶が注がれた。\\n\
    \\0……h1111206彼らは私のことを「主」と呼ぶの。\\n\
    契約関係としては対等なのだけれど、彼ら自身がそう呼ぶのを好むのよ。\\n\
    \\n\
    h1111209耳を澄ませていれば、彼らの声が聞こえることもあるんじゃない？\\n\
    私を通して彼らとも縁ができているはずだから。{}{}\
    ",
    if !get_global_vars()
      .flags()
      .check(&EventFlag::TalkTypeUnlock(TalkType::Servant))
    {
      let user_name = if let Some(name) = get_global_vars().user_name() {
        name.to_string()
      } else {
        "お客".to_string()
      };
      format!(
        "\\![set,balloonnum,おや、本当だ。よろしくね、{}さん。]",
        user_name
      )
    } else {
      "".to_string()
    },
    if !get_global_vars()
      .flags()
      .check(&EventFlag::TalkTypeUnlock(TalkType::Servant))
    {
      render_achievement_message(TalkType::Servant)
    } else {
      "".to_string()
    },
  )
}
