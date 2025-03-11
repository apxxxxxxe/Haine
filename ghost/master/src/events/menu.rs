use crate::check_error;
use crate::error::ShioriError;
use crate::events::common::*;
use crate::events::first_boot::FIRST_RANDOMTALKS;
use crate::events::input::InputId;
use crate::events::talk::randomtalk::random_talks;
use crate::events::TalkType;
use crate::events::TalkingPlace;
use crate::variables::PendingEvent;
use crate::variables::{
  EventFlag, FLAGS, IS_IMMERSIVE_DEGREES_FIXED, PENDING_EVENT_TALK, RANDOM_TALK_INTERVAL,
  TALKING_PLACE, TALK_COLLECTION, USER_NAME,
};
use shiorust::message::{Request, Response};

pub(crate) fn on_menu_exec(_req: &Request) -> Response {
  let current_talk_interval = *RANDOM_TALK_INTERVAL.read().unwrap();
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
  let m = format!(
    "\\_q{}{}",
    REMOVE_BALLOON_NUM,
    if !FLAGS.read().unwrap().check(&EventFlag::FirstRandomTalkDone(
      (FIRST_RANDOMTALKS.len() - 1) as u32,
    )) {
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
        if *TALKING_PLACE.read().unwrap() == TalkingPlace::Library {
          "耳を澄ます"
        } else {
          "なにか話して"
        },
        if *TALKING_PLACE.read().unwrap() == TalkingPlace::Library {
          "".to_string()
        } else {
          "\\![*]\\q[話しかける,OnTalk]\\n".to_string()
        },
        talk_interval_selector,
        close_button,
        {
          let hoge = PENDING_EVENT_TALK.read().unwrap();
          if hoge.is_some() {
            format!(
              "\\![*]\\q[{},OnStoryEvent,{}]",
              hoge.as_ref().unwrap(),
              hoge.as_ref().unwrap()
            )
          } else {
            "".to_string()
          }
        }
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
  *RANDOM_TALK_INTERVAL.write().unwrap() = v;

  Ok(on_menu_exec(req))
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct Question(pub(crate) u32);

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
  const CAN_I_STAY_TONIGHT: Self = Self(11); // 今日泊まってもいい？
  const IS_THERE_A_PLACE_TO_VISIT: Self = Self(12); // このあたりに観光できる場所はある？
  const YOU_ARE_CUTE: Self = Self(13); // 『かわいい』
  const YOU_ARE_BEAUTIFUL: Self = Self(14); // 『美人』
                                            // 『似顔絵を描く』
  const I_AM_HUNGRY: Self = Self(16); // 『お腹が空いた』

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
      Question::CAN_I_STAY_TONIGHT => "今日泊まってもいい？".to_string(),
      Question::IS_THERE_A_PLACE_TO_VISIT => "このあたりに観光できる場所はある？".to_string(),
      Question::YOU_ARE_CUTE => "かわいい".to_string(),
      Question::YOU_ARE_BEAUTIFUL => "美人".to_string(),
      Question::I_AM_HUNGRY => "お腹が空いた".to_string(),
      _ => unreachable!(),
    }
  }

  fn to_script(self) -> String {
    format!("\\![*]\\__q[OnTalkAnswer,{}]{}\\__q", self.0, self.theme())
  }

  pub(crate) fn talk(&self) -> String {
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
      Question::CAN_I_STAY_TONIGHT => "\
      \\1『今日泊まってもいい？』\\n\\n\
      h1111201あら、泊まりたいの？h1111204ええ、構わないわよ。\\n\
      h1111206客室は常に手入れされているし、\\n\
      寝心地も保証するわ。\\n\
      \\n\
      ……h1111204いっそ、ここへ住んでもいいのよ？\\n\
      あなたの食事は用意できないけれど、\\n\
      それ以外で不自由はさせないわ。\\n\\n\
      h1111104\\1『そうしたいのは山々だけど、\\n\
      生活があるから…』\\n\
      \\0…………h1111205ええ、そうよね。\\n\
      \\n\
      h1111204部屋にはあとで案内させるわ。\\n\
      h1111210くつろいでちょうだい。\
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
      h1111205娯楽でいえば、北側に小さな劇場もあるわね。\\n\
      あれもかつては貴族の社交場だったのだけれど、\\n\
      今はもう、地元の劇団が使う程度ね。\\n\
      それでも細々と公演を続けているわ。\\n\
      ……h1111210まあ、\\n\
      退屈しのぎには良いんじゃないかしら。\\n\
      \\n\
      ……そんなことを聞くなんて、\\n\
      h1111204私との語らいには飽きてしまったのかしら？\\n\
      ……h1111310冗談よ。\\n\
      h1111304あなたの目はそう言っていないものね。\
      "
      .to_string(),
      Question::YOU_ARE_CUTE => "\
      \\1『かわいい』\\n\
      h1111101……h1111204「可愛げがない」ではなくて？\\n\
	  h1111210ふふ、わかっているわ。\\n\
	  h1111205生前はそういう言葉をよく聞いたのよ。\\n\
	  \\n\
	  h1113210それにしても慣れないものね。\\n\
	  h1113204参考までに、私のどこを「かわいい」と感じたのか聞いても良いかしら？\
      ".to_string(),
	  Question::YOU_ARE_BEAUTIFUL => "\
	  \\1『美人』\\n\
	  h1111210……まあ、「かわいい」よりは言われ慣れているわね。\\n\
	  h1111206着飾るのも、それを見せるのもあまり興味がないうえ、\\n\
	  不必要に言い寄られることもあったものだから……\\n\
	  h1111205あまり好ましいとは思わないのだけれど。\\n\
	  h1111210……それでも、こうしてあなたを喜ばせられているのならば、それはきっと良いことなのでしょうね。\
	  ".to_string(),
	  Question::I_AM_HUNGRY => "\
	  \\1『お腹が空いた』\\n\
	  h1111104あら、もうそんな時間？\\n\
	  h1111206……悪いけれど、ここには食事の用意はないの。\\n\
	  h1111210私たちの食事はもっぱら娯楽として行うもので、それも茶菓子程度だから。\\n\
	  \\n\
	  だから、あなた自身で外から持って来てもらうことになるのだけど……\\n\
	  h1111205…………その、できればここで食べて見せてほしいわ。\\n\
	  h1111210……娯楽だけでない、生きる糧としての食事を眺めていたいの。\
	  ".to_string(),
      _ => unreachable!(),
    };
    m + "\\x\\![raise,OnTalk]"
  }
}

pub(crate) const QUESTIONS: [Question; 15] = [
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
  Question::CAN_I_STAY_TONIGHT,
  Question::IS_THERE_A_PLACE_TO_VISIT,
  Question::YOU_ARE_CUTE,
  Question::YOU_ARE_BEAUTIFUL,
  Question::I_AM_HUNGRY,
];

pub(crate) fn on_talk(_req: &Request) -> Result<Response, ShioriError> {
  let mut questions = QUESTIONS.to_vec();
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
  let talk_collection = TALK_COLLECTION.read().unwrap();
  let talking_place = TALKING_PLACE.read().unwrap();
  lines.push(format!("[トーク統計: {}]\\n", talking_place));
  let talk_types = talking_place.talk_types();
  let is_unlocked_checks = talk_types
    .iter()
    .map(|t| FLAGS.read().unwrap().check(&EventFlag::TalkTypeUnlock(*t)))
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
  new_response_with_value_with_translate(
    format!(
      "\\_q\\![open,inputbox,{},0]新しい呼び名を入力してください。\\n現在:{}",
      InputId::UserName,
      *USER_NAME.read().unwrap()
    ),
    TranslateOption::with_shadow_completion(),
  )
}

pub(crate) fn on_immersive_degree_toggled(req: &Request) -> Response {
  let i;
  {
    i = *IS_IMMERSIVE_DEGREES_FIXED.read().unwrap();
  }
  *IS_IMMERSIVE_DEGREES_FIXED.write().unwrap() = !i;

  on_menu_exec(req)
}

pub(crate) fn on_story_event(req: &Request) -> Result<Response, ShioriError> {
  let refs = get_references(req);
  let s = if let Some(hoge) = PendingEvent::from_str(refs[0]) {
    let callback = || {
      *PENDING_EVENT_TALK.write().unwrap() = None;
    };
    match hoge {
      PendingEvent::ConfessionOfSuicide => {
        unreachable!();
      }
      PendingEvent::UnlockingLoreTalks => {
        FLAGS
          .write()
          .unwrap()
          .done(EventFlag::TalkTypeUnlock(TalkType::Lore));
        callback();
        unlock_lore_talks()
      }
      PendingEvent::UnlockingServantsComments => {
        FLAGS
          .write()
          .unwrap()
          .done(EventFlag::TalkTypeUnlock(TalkType::Servant));
        callback();
        unlock_servents_comments()
      }
    }
  } else {
    return Err(ShioriError::InvalidEvent);
  };
  new_response_with_value_with_translate(s, TranslateOption::with_shadow_completion())
}

fn unlock_lore_talks() -> String {
  format!(
    "\
    h1111201死について。深く考えることはある？\\n\
    h1111206……あなたには聞くまでもないわよね。\\n\
    h1111205私もそうなの。\\n\
    生きていたころから、なぜ生きるのか、死ぬとはどういうことかをずっと考えていたわ。\\n\
    いくつか不思議な話を知っているの。\\n\
    話の種に、語ってみましょうか。{}\
    ",
    if !FLAGS
      .read()
      .unwrap()
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
    if !FLAGS
      .read()
      .unwrap()
      .check(&EventFlag::TalkTypeUnlock(TalkType::Servant))
    {
      format!(
        "\\![set,balloonnum,おや、本当だ。よろしくね、{}さん。]",
        *USER_NAME.read().unwrap()
      )
    } else {
      "".to_string()
    },
    if !FLAGS
      .read()
      .unwrap()
      .check(&EventFlag::TalkTypeUnlock(TalkType::Servant))
    {
      render_achievement_message(TalkType::Servant)
    } else {
      "".to_string()
    },
  )
}
