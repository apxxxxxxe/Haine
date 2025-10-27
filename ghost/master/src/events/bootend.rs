use crate::error::ShioriError;
use crate::events::check_story_events;
use crate::events::common::*;
use crate::events::first_boot::{
  FIRST_BOOT_MARKER, FIRST_BOOT_TALK, FIRST_CLOSE_TALK, FIRST_RANDOMTALKS,
};
use crate::events::TalkingPlace;
use crate::variables::*;
use chrono::{Datelike, Timelike};
use rand::seq::SliceRandom;
use shiorust::message::{parts::HeaderName, Response, *};

pub(crate) fn on_boot(_req: &Request) -> Result<Response, ShioriError> {
  *TOTAL_BOOT_COUNT.write().unwrap() += 1;

  // 初回起動
  if !FLAGS.read().unwrap().check(&EventFlag::FirstBoot) {
    FLAGS.write().unwrap().done(EventFlag::FirstBoot);
    let mut res = new_response_with_value_with_translate(
      FIRST_BOOT_TALK.to_string(),
      TranslateOption::simple_translate(),
    )?;
    res.headers.insert_by_header_name(
      HeaderName::from("Marker"),
      format!("{}(1/{})", FIRST_BOOT_MARKER, FIRST_RANDOMTALKS.len() + 1),
    );
    return Ok(res);
  }

  check_story_events();

  // トーク内容の決定（日付イベント or 通常トーク）
  let talk_content = if let Some(event_talk) = check_date_event_talk() {
    event_talk
  } else {
    let talks = all_combo(&vec![
      vec![render_immersive_icon()],
      vec!["h1113105\\1今日も、霧が濃い。".to_string()],
      vec![format!(
        "\
        h1113105……h1113101\\_w[300]h1113201あら。\\n\
        h1111204{}、{{user_name}}。\
        ",
        {
          let hour = chrono::Local::now().hour();
          if hour <= 3 || hour >= 19 {
            "こんばんは"
          } else if hour < 11 {
            "おはよう"
          } else {
            "こんにちは"
          }
        }
      )],
    ]);
    let index = choose_one(&talks, false).ok_or(ShioriError::ArrayAccessError)?;
    talks[index].clone()
  };

  let v = format!(
    "\\0\\s[{}]{}\\![embed,OnStickSurface]{}{}",
    TRANSPARENT_SURFACE,
    RESET_BINDS,
    randomize_underwear(),
    talk_content,
  );
  new_response_with_value_with_translate(v, TranslateOption::simple_translate())
}

pub(crate) fn on_close(_req: &Request) -> Result<Response, ShioriError> {
  let mut parts = vec![vec![RESET_BINDS.to_string()]];

  if *TALKING_PLACE.read().unwrap() == TalkingPlace::Library {
    parts.push(vec![format!(
      "\\0\\b[{}]h1111705……。\
      \\1ネ……\\n\
      イネ……。\
      \\0\\b[{}]hr1141112φ！\
      \\1\\nハイネ！\
      \\0…………\\n\\n\
      h1111101……h1111204あら、{{user_name}}。\\n\
      \\1\\n\\n……戻ってきたようだ。\\n\
      \\0h1111210……そう、今日はおしまいにするのね。\\n\\n\\1\\b[-1]",
      TalkingPlace::Library.balloon_surface(),
      TalkingPlace::LivingRoom.balloon_surface(),
    )]);
  }
  if !FLAGS.read().unwrap().check(&EventFlag::FirstClose) {
    FLAGS.write().unwrap().done(EventFlag::FirstClose);
    parts.push(vec![FIRST_CLOSE_TALK.to_string()]);
  } else {
    parts.extend(vec![
      vec!["h1111210".to_string(), "h1111211".to_string()],
      vec!["あなたに".to_string()],
      vec![
        "すばらしき朝".to_string(),
        "蜜のようなまどろみ".to_string(),
        "暗くて静かな安らぎ".to_string(),
        "良き終わり".to_string(),
        "孤独と救い".to_string(),
      ],
      vec!["がありますように。\\nh1111204またね、{user_name}。\\_w[1200]".to_string()],
    ]);
  }
  let talks = all_combo(&parts);
  let index = choose_one(&talks, true).ok_or(ShioriError::ArrayAccessError)?;
  new_response_with_value_with_translate(
    format!("{}{}\\-", RESET_BINDS, talks[index].clone()),
    TranslateOption::simple_translate(),
  )
}

pub(crate) fn on_vanish_selecting(_req: &Request) -> Response {
  let m = "\\1※Vanishイベントは未実装です。".to_string();
  new_response_with_value_with_notranslate(m, TranslateOption::none())
}

fn randomize_underwear() -> String {
  let mut rng = rand::thread_rng();
  let candidates = ["A", "B"];
  format!(
    "\\0\\![bind,下着,{},1]",
    candidates.choose(&mut rng).unwrap()
  )
}

fn check_date_event_talk() -> Option<String> {
  let now = chrono::Local::now();
  let year = now.year() as u32;
  let month = now.month();
  let day = now.day();

  // 既に今年のイベントを閲覧済みならスキップ
  if FLAGS.read().unwrap().check_season_event(year, month, day) {
    return None;
  }

  // イベントトークを取得
  let talk = match (month, day) {
    (10, 31) => Some(halloween_boot_talk()),
    // 今後の日付イベントはここに追加
    // (12, 25) => Some(christmas_boot_talk()),
    // (1, 1) => Some(newyear_boot_talk()),
    _ => None,
  };

  // イベントトークがあれば閲覧済みフラグを立てる
  if talk.is_some() {
    FLAGS.write().unwrap().mark_season_event(year, month, day);
  }

  talk
}

pub(crate) fn halloween_boot_talk() -> String {
  "\
    h1000000\\1\\b[10]今日も館に足を運ぶ。\\n\
    空は薄曇りで、街並みがいつもより静寂に包まれている。\\n\
    \\n[half]\
    いつもの重い門をくぐり、石畳を歩く。\\n\
    落ち葉が足元で小さく音を立てた。\\n\
    乾いた霧は、\\n\
    館の周りでより一層濃くなっているようだった。\\x\
    \\1\\b[10]呼び鈴を押し、しばらく待つ。\\n\
    \\n[half]\
    …………。\\n\
    \\n[half]\
    いつもならハイネが迎えてくれるはずだが、返事がない。\\n\
    今まで留守ということはなかったのだが……。\\n\
    \\n[half]\
    今日は帰ろうか、と考えたその瞬間……ギィ、と扉が開いた。\\n\
    そこには誰もいない。しかし、入れと言われていることは明白だった。\\n\
    \\n[half]\
    今日は少し冷える。従者の誰かが\\n\
    気を利かせてくれたのだろうか……。\\x\
    \\1\\b[10]開いた扉の隙間を抜け、館に入る。\\n\
    背後で静かに扉が閉まり、\\n\
    静まり返った玄関ホールが薄闇の中にぼんやりと見えた。\\n\
    \\n[half]\
    勝手に客間に入っていいものだろうか……。\\n\
    迷いつつ一歩踏み出したそのとき、\\b[-1]\\n\
    \\n[half]\
    h1111604ばあ\\n\
    \\n[half]\
    \\1\\b[0]\\c\\![set,balloonwait,0.75]φ！φ！φ！φ！φ！φ！φ！φ！φ！φ！φ！\\![set,balloonwait,1]\\w_[1200]\
    \\c……背後に、ハイネがいた。\\n\
    \\n[half]\
    \\0……h1221710ぷっ\\n\
    h1221911あはっ、ははははは……h1000000\\n\
    \\1見たことのない勢いで笑っている……。\\c\
    h1000000\\c――――――――\\_w[2000]\\c\
    \\0h1223304はあ、そんなに驚くとは思わなくて。\\n\
    h1223210ふふ、 ごめんなさい。\\n\
    ……h1223205あら、腰が抜けてしまったの？\\n\
    h1000000ほら、手に捕まって。\\n\
    \\1支えにするには頼りない腕に捕まり、なんとか立ち上がる。\\n\
    h1211201\\1なぜ、こんな真似を……。\\n\
    \\n[half]\
    h1211201なぜって、今日はハロウィンでしょう？ \\n\
    \\n[half]\
    \\1よほどおかしかったのか、\\n\
    ハイネの頬が珍しく紅潮している。\\n\
    \\n[half]\
    今日はハロウィン……。すっかり忘れていた。\\n\
    そういえば、街中でそれらしい装飾を見かけたような気もする。\\n\
    \\n[half]\
    それにしても、なんというか……無邪気な悪戯だ。\\n\
    『あなたはこういうことしないと思ってた』\\_w[1200]\\n\
    \\n[half]\
    h1211204堅物で高慢なだけの主だと思ってた？\\n\
    h1211210いいえ、私は私。\\n\
    h1211206お祭りも悪戯も、嫌いじゃないのよ。\\n\
    \\n[half]\
    h1211204\\1ハイネは満足そうに微笑んでいる。\\n\
    よく見ると、屋敷はいつもより華やかな装いをしているようだった。\\n\
    いつも重たい印象のカーテンには、橙色の飾りが軽やかに下がっている。\\n\
    \\n[half]\
    h1211204今夜は特別なのよ。年に一度、死者と生者の境界が曖昧になる夜。\\n\
    h1211206私たちにとっても無縁ではない、唯一の宴。\\n\
    \\n[half]\
    \\1客間には、嗅ぎ慣れない甘い香りが漂っている。\\n\
    \\n[half]\
    h1211206お菓子を用意したの。\\n\
    生者の世界の伝統に倣ってね。\\n\
    私に用意できるものは限られているけれど……\\n\
    それでも、この夜を一緒に過ごせることが嬉しいわ。\\n\
    \\n[half]\
    \\1テーブルの上には、普段あまり目にしない美しい菓子が並んでいた。\\n\
    ぼんやりと照らされるさまは幻想的で、どこかこの世ならざる雰囲気を感じる。\\n\
    \\_w[1200]\\n[half]\
    h1211204\\1ハイネは優雅に手を差し出した。\\n\
    \\n[half]\
    h1211210どう？私の悪戯は成功だったかしら。\\n\
    Trick or Treat……h1111204今度はあなたの番よ。\\n\
    何か面白いことを見せてくれる？h1111210それとも……。h1111204\
    \\1\\cハイネの瞳が、いつもより輝いて見えた.\
  ".to_string()
}
