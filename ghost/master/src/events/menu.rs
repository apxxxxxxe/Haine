use crate::error::ShioriError;
use crate::events::common::*;
use crate::events::first_boot::{FIRST_BOOT_TALK, FIRST_RANDOMTALKS};
use crate::events::input::InputId;
use crate::events::talk::randomtalk::{derivative_talks_per_talk_type, random_talks};
use crate::events::TalkType;
use crate::events::TalkingPlace;
use crate::variables::PendingEvent;
use crate::variables::{
  EventFlag, FLAGS, IS_IMMERSIVE_DEGREES_FIXED, PENDING_EVENT_TALK, RANDOM_TALK_INTERVAL,
  TALKING_PLACE, TALK_COLLECTION, USER_NAME,
};
use crate::{check_error, DERIVATIVE_TALK_REQUESTABLE, IMMERSIVE_DEGREES};
use shiorust::message::{Request, Response};

use super::aitalk::IMMERSIVE_RATE_MAX;
use super::talk::first_boot::FIRST_CLOSE_TALK;
use super::talk::randomtalk::moving_to_library_talk_parts;

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

  let buttons = format!(
    "\\_l[0,0]\\f[align,right]{}\\__q[script:\\e]{}\\__q",
    if FLAGS.read().unwrap().check(&EventFlag::FirstRandomTalkDone(
      (FIRST_RANDOMTALKS.len() - 1) as u32,
    )) {
      format!("\\__q[OnConfigMenuExec]{}\\__q ", Icon::Cog)
    } else {
      "".to_string()
    },
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
        + &buttons
    } else {
      format!(
        "\
        \\_l[0,1.5em]\
        \\![*]\\q[{},OnAiTalk]\\n\
        {}\
        \\![*]\\q[トーク統計,OnCheckTalkCollection]\\n\
        \\![*]\\q[回想,OnStoryHistoryMenu]\
        \\_l[0,@2.5em]\
        \\![*]\\q[手紙を書く,OnWebClapOpen]\
        \\_l[0,@2.5em]\
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
        buttons,
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
    if *DERIVATIVE_TALK_REQUESTABLE.read().unwrap() {
      "表示"
    } else {
      "非表示"
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
  const I_DREW_YOUR_PORTRAIT: Self = Self(15); // 『似顔絵を描いた』 ：描いた似顔絵を見せる
  const I_AM_HUNGRY: Self = Self(16); // 『お腹が空いた』
  const WHAT_IS_YOUR_FAVORITE_SNACK: Self = Self(17); // 好きなお茶菓子は何？
  const CAN_I_TALK_TO_YOUR_SERVANTS: Self = Self(18); // 従者たちと話してもいい？
  const CALL_YOU_HAINE_1: Self = Self(19); // 「ハイネ」
  const CALL_YOU_HAINE_2: Self = Self(20); // 「ハイネさん」
  const CALL_YOU_HAINE_3: Self = Self(21); // 「ハイネちゃん」
  const WHEN_DO_YOU_WAKE_UP: Self = Self(22); // ふだん何時に寝起きしてる？
  const WHY_IS_YOUR_BODY_COLD: Self = Self(23); // どうして体温が低い？
  const AM_I_BOTHERING_YOU: Self = Self(24); // 迷惑じゃない？
  const CALL_YOU_MOTHER: Self = Self(25); // 「お母さん」
  const CALL_YOU_SISTER: Self = Self(26); // 「お姉ちゃん」
  const LET_ME_PLAY: Self = Self(27); // 「遊びに行こう」
  const CAN_I_PET_YOU: Self = Self(28); // 「なでていい？」

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
      Question::WHEN_DO_YOU_WAKE_UP => "ふだん何時に寝起きしてる？".to_string(),
      Question::CAN_I_TALK_TO_YOUR_SERVANTS => "従者たちと話してもいい？".to_string(),
      Question::CALL_YOU_HAINE_1 => "ハイネ".to_string(),
      Question::CALL_YOU_HAINE_2 => "ハイネさん".to_string(),
      Question::CALL_YOU_HAINE_3 => "ハイネちゃん".to_string(),
      Question::WHY_IS_YOUR_BODY_COLD => "どうして体温が低い？".to_string(),
      Question::AM_I_BOTHERING_YOU => "迷惑じゃない？".to_string(),
      Question::CALL_YOU_MOTHER => "お母さん".to_string(),
      Question::CALL_YOU_SISTER => "お姉ちゃん".to_string(),
      Question::WHAT_IS_YOUR_FAVORITE_SNACK => "好きなお茶菓子は何？".to_string(),
      Question::I_DREW_YOUR_PORTRAIT => "似顔絵を描いた".to_string(),
      Question::LET_ME_PLAY => "遊びに行こう".to_string(),
      Question::CAN_I_PET_YOU => "なでていい？".to_string(),
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
    Question::CAN_I_TALK_TO_YOUR_SERVANTS => "\
    \\1『従者たちと話してもいい？』\\n\
    h1111101それは……h1111104許容しかねるわ。\\n\
    h1111110彼らは外部からの影響に弱いの。\\n\
    h1113206挨拶程度なら構わないけれど、それ以上…ただの雑談だとしても、\\n\
    あなたから漏れる悪意なき偏見が、彼らの存在を不可逆に再定義してしまうかもしれない。\\n\
    h1113210生者から死者へ言葉を送るという行為がもつ意味は、\\n\
    あなたが想像するより遥かに重いの。\\n\
    この、互いの声が漏れ聞こえている状況が限界点。\\n\
    h1111204分かってちょうだいね。\
    ".to_string(),
    Question::CALL_YOU_HAINE_1 => "\
    \\1『ハイネ』\\n\
    h1111201ええ、何？\
    ".to_string(),
    Question::CALL_YOU_HAINE_2 => "\
    \\1『ハイネさん』\\n\
    h1111204……どうしたの、かしこまって。\\n\
    ".to_string(),
    Question::CALL_YOU_HAINE_3 => "\
    \\1『ハイネちゃん』\\n\
    h1111210うん……h1111201うん？\
    ".to_string(),
    Question::WHEN_DO_YOU_WAKE_UP => "\
    \\1『ふだん何時に寝起きしてる？』\\n\
    h1111204……質問に答えるなら、数日起きて、数日寝ているわ。\\n\
    h1111206私も彼らも、必要ならば何日でも活動し続ける。\\n\
    h1111210霊は基本的に眠りを必要としないから、\\n\
    起きていようと思えばいくらでも起きていられるのよ。\\n\
    \\n\
    h1121211とはいえ、それでは倦んでしまうから。\\n\
    h1111210必要に応じて…というか、起きている必要のないときは眠るの。\\n\
    \\n\
    h1111204……最近は、ある人間のおかげで退屈しなくて済んでいるけれど、ね。\\n\
    ".to_string(),
    Question::WHY_IS_YOUR_BODY_COLD => "\
    \\1『どうして体温が低い？』\\n\
    h1113205一言で言えば、血が通っていないからでしょうね。\\n\
    生物に体温があるのは、代謝……生存のための化学反応に温度が必要だから。\\n\
    h1113204血液を回し、酸素を通わせ、栄養をエネルギーに、そして熱に変える。\\n\
    h1113206一方で……私達がどのような原理で存在しているのかは未解明だけれど、温度を必要としない在り方なのでしょう。\\n\
    h1113204まあ、そのせいであなたには冷たい思いをさせてしまうけれど。\\n\
    h1113205……私の手をろうそくで炙れば、少しは温かくなるかしら？\\n\
    痛覚もさほどh1113101……h1121210冗談よ。そんな顔しないで。\
    ".to_string(),
    Question::AM_I_BOTHERING_YOU => "\
    \\1『迷惑じゃない？』\\n\
    h1111204……今更よ、そんなこと。\\n\
    h1111210この場所は私のもの。h1111204ここにいるのは、私が必要とするものだけよ。\\n\
    h1111204あなたが必要だから、あなたはここにいるの。\\n\
    \\n\
    \\_w[1200]h1111210さあ、くだらないことを考えるのはおしまい。\\n\
    h1111204いつものように、あなたの話を聞かせてちょうだい。\
    ".to_string(),
    Question::CALL_YOU_MOTHER => "\
    \\1『お母さん』\\n\
    h1111101……h1111304聞き間違いかしら？\\n\
    h1111210先生のことを間違えてそう呼んでしまうという笑い話はよく聞くけれど。\\n\
    h1111204まさか、私を母親と間違えたわけではないでしょう？\\n\
    h1111210私に母親の素質なんてないものね。\\n\
    ".to_string(),
    Question::CALL_YOU_SISTER => "\
    \\1『お姉ちゃん』\\n\
    h1111204……h1111210きょうだいにしては、歳が離れているわね。\\n\
    そういう戯れの気分なのかしら？h1111204{user_name}ちゃん。\
    ".to_string(),
    Question::WHAT_IS_YOUR_FAVORITE_SNACK => "\
    \\1『好きなお茶菓子は何？』\\n\
    h1111205そうね……硬く焼き締めた菓子が好きなの。\\n\
    h1111206ビスケットやラスクのような、\\n\
    日持ちがして、片手でつまめるもの。\\n\
    \\n\
    h1111204生前の私は寝食を忘れて本を読むことが多くてね。\\n\
    h1111210食事の時間になっても席を立たず、\\n\
    家政婦を困らせていたのよ。\\n\
    \\n\
    h1111206見かねた彼女が、作業をしながらでも食べられるように\\n\
    硬く焼いた菓子を用意してくれたの。\\n\
    h1111210それが思いのほか美味しくて、\\n\
    読書の合間につまむのが習慣になったわ。\\n\
    \\n\
    h1111205……あれは私の体調を案じてくれた\\n\
    優しい工夫だったのでしょう。\\n\
    h1111206だからこそ、今でもあの味を懐かしく思うのよ。\
    ".to_string(),
    Question::I_DREW_YOUR_PORTRAIT => "\
    \\1『似顔絵を描いた』\\n\
    h1111101あら、私を？\\n\
    h1111204……見せてもらえるかしら？\\n\
    \\n\
    h1111210\\1手に持っていたスケッチブックを見せると、\\n\
    ハイネがゆっくりと手を伸ばしてページをめくった。\\n\
    \\0h1111205……h1111206なるほど。\\n\
    あなたの目には、私はこんな風に映っているのね。\\n\
    \\n\
    h1111210絵の技術もなかなかのものだけど、\\n\
    h1111204それよりも、あなたが私を見つめていた時間を思うと……\\n\
    h1111205少し照れくさいわね。h1111210ありがとう。\\n\
    \\n\
    h1111204大切にしてちょうだい。\\n\
    私にとっても、あなたにとっても、\\n\
    この瞬間の証になるものだから。\
    ".to_string(),
    Question::LET_ME_PLAY => "\
    \\1『遊びに行こう』\\n\
    h1111210……遊び。\\n\
    h1111204その「遊び」とは、どのようなものかしら。\\n\
    h1111206私にとっての娯楽といえば、読書や音楽鑑賞程度だけれど、\\n\
    h1111205生きている人間の「遊び」は、もっと動的なものでしょう？\\n\
    \\n\
    h1111210……でも、面白そうね。\\n\
    h1111204あなたがどのような遊びを望むのか、\\n\
    聞かせてちょうだい。\\n\
    h1111204この館の中でできることなら、私も一緒に楽しませてもらうわ。\\n\
    h1111206……もしくは、見学させてもらうかしら。\\n\
    h1111210霊体では制約も多いものだから。\
    ".to_string(),
    Question::CAN_I_PET_YOU => "\
    \\1『なでていい？』\\n\
    h1111101……h1111201なでる？\\n\
    h1111204ああ、頭のことね。\\n\
    h1111205構わないわ。\\n\
    \\n\
    h1111210\\1そっと手を伸ばすと、\\n\
    ハイネの髪は思った通り柔らかく、\\n\
    冷たい感触が指先に伝わってくる。\\n\
    h1111105……h1111210久しぶりね、人の手の温もりを感じるのは。\\n\
    h1111206生前、最後に誰かに触れられたのは……\\n\
    h1111210……もう覚えていないわ。\\n\
    \\n\
    h1111205あなたの手は温かいのね。\\n\
    私が冷たいからそう感じるのかもしれないけれど、\\n\
    h1111210それでも、温かい。\
    ".to_string(),
      _ => unreachable!(),
    };
    m + "\\x\\![raise,OnTalk]"
  }
}

pub(crate) const QUESTIONS: [Question; 28] = [
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
  Question::I_DREW_YOUR_PORTRAIT,
  Question::I_AM_HUNGRY,
  Question::WHAT_IS_YOUR_FAVORITE_SNACK,
  Question::CAN_I_TALK_TO_YOUR_SERVANTS,
  Question::CALL_YOU_HAINE_1,
  Question::CALL_YOU_HAINE_2,
  Question::CALL_YOU_HAINE_3,
  Question::WHEN_DO_YOU_WAKE_UP,
  Question::WHY_IS_YOUR_BODY_COLD,
  Question::AM_I_BOTHERING_YOU,
  Question::CALL_YOU_MOTHER,
  Question::CALL_YOU_SISTER,
  Question::LET_ME_PLAY,
  Question::CAN_I_PET_YOU,
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

pub(crate) fn on_derivative_talk_request_button_toggled(req: &Request) -> Response {
  let is_derivative_talks_enabled;
  {
    is_derivative_talks_enabled = *DERIVATIVE_TALK_REQUESTABLE.read().unwrap();
  }
  *DERIVATIVE_TALK_REQUESTABLE.write().unwrap() = !is_derivative_talks_enabled;

  on_config_menu_exec(req)
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
      _ => {
        unreachable!();
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
    FLAGS.read().unwrap().check(&EventFlag::FirstClose),
  ));
  events.push((
    "ロアトーク開放".to_string(),
    PendingEvent::UnlockingLoreTalks,
    FLAGS
      .read()
      .unwrap()
      .check(&EventFlag::TalkTypeUnlock(TalkType::Lore)),
  ));
  events.push((
    "従者コメント開放".to_string(),
    PendingEvent::UnlockingServantsComments,
    FLAGS
      .read()
      .unwrap()
      .check(&EventFlag::TalkTypeUnlock(TalkType::Servant)),
  ));
  events.push((
    "初回独白モード移行".to_string(),
    PendingEvent::FirstPlaceChange,
    FLAGS.read().unwrap().check(&EventFlag::FirstPlaceChange),
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
      PendingEvent::FirstPlaceChange => {
        // 没入度を実際に変更しないと影の描写が再現されない
        // トーク再生中にembedでsmooth_blinkが入りそのタイミングでも没入度が参照されるため、
        // この時点で戻すわけにもいかない
        // この回想を見ると独白モードへ移行するという仕様にする
        *IMMERSIVE_DEGREES.write().unwrap() = IMMERSIVE_RATE_MAX;
        *TALKING_PLACE.write().unwrap() = TalkingPlace::Library;

        let parts = moving_to_library_talk_parts(true)?;
        match all_combo(&parts).first() {
          Some(v) => (
            format!(
              "\\p[2]{}\\0{}{}",
              render_immersive_icon(),
              IMMERSIVE_DEGREES.read().unwrap(),
              v
            ),
            TranslateOption::with_shadow_completion(),
          ),
          None => {
            return Err(ShioriError::InvalidEvent);
          }
        }
      }
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
    私を通して彼らとも縁ができているはずだから。{}\
    ",
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
