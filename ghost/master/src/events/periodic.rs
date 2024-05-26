use crate::events::aitalk::on_ai_talk;
use crate::events::common::*;
use crate::events::first_boot::FIRST_RANDOMTALKS;
use crate::variables::{get_global_vars, EventFlag};
use chrono::Timelike;
use rand::prelude::SliceRandom;
use shiorust::message::{Request, Response};

pub fn on_notify_user_info(req: &Request) -> Response {
  let vars = get_global_vars();
  let refs = get_references(req);
  let user_name = refs[0].to_string();
  if vars.user_name().is_none() {
    vars.set_user_name(Some(user_name));
  }
  new_response_nocontent()
}

pub fn on_second_change(req: &Request) -> Response {
  let vars = get_global_vars();

  // 最小化中かどうかに関わらず実行する処理
  let total_time = vars.total_time().unwrap();
  vars.set_total_time(Some(total_time + 1));
  vars
    .volatility
    .set_ghost_up_time(vars.volatility.ghost_up_time() + 1);

  // 初回起動イベントが終わるまではランダムトークなし
  if !vars.flags().check(&EventFlag::FirstRandomTalkDone(
    FIRST_RANDOMTALKS.len() as u32 - 1,
  )) {
    return new_response_nocontent();
  }

  let refs = get_references(req);
  let idle_secs = refs[4].parse::<i32>().unwrap();
  vars.volatility.set_idle_seconds(idle_secs);

  if vars.random_talk_interval().unwrap() > 0
    && (vars.volatility.ghost_up_time() - vars.volatility.last_random_talk_time())
      > vars.random_talk_interval().unwrap()
    && !vars
      .volatility
      .status()
      .get("minimizing")
      .is_some_and(|v| v)
  {
    match vars.cumulative_talk_count() {
      5 => {
        // 5回話したら従者関連のトークを解放
        vars.flags_mut().done(EventFlag::ServantIntroduction);
      }
      10 => {
        // 10回話したらロア解放
        vars.flags_mut().done(EventFlag::LoreIntroduction);
      }
      20 => {
        // 20回話したら没入度解放
        vars.flags_mut().done(EventFlag::ImmersionIntroduction);
      }
      _ => {}
    }
    return on_ai_talk(req);
  }

  let mut text = String::new();
  if vars.volatility.ghost_up_time() % 60 == 0 && !vars.volatility.status().get("talking").unwrap()
  {
    // 1分ごとにサーフェスを重ね直す
    text += STICK_SURFACE;
  }

  let now = chrono::Local::now();
  if now.minute() == 0 && now.second() == 0 {
    let tanka_list = [
      tanka(
        "もう二度と死ななくてよい安らぎに\\n見つめてゐたり祖母の寝顔を",
        "梶原さい子",
      ),
      tanka(
        "眼のまはり真紅(まあか)くなして泣きやめぬ\\n妻のうしろに吾子死にてあり",
        "木下利玄",
      ),
      tanka(
        "我が母よ死にたまひゆく我が母よ\\n我(わ)を生まし乳足(ちた)らひし母よ",
        "斎藤茂吉",
      ),
      tanka(
        "眠られぬ母のためわが誦む童話\\n母の寝入りし後王子死す",
        "岡井隆",
      ),
      tanka(
        "死せる犬またもわが眼にうかび来ぬ、\\nかの川ばたの夕ぐれの色",
        "金子薫園",
      ),
      tanka(
        "死に一歩踏み入りしとふ実感は\\nひるがへつて生の実感なりし",
        "後藤悦良",
      ),
      tanka(
        "蛍光灯のカヴァーの底を死場所としたる\\nこの世の虫のかずかず",
        "小池光",
      ),
      tanka(
        "死に向かふ生の底知れぬ虚無の淵を\\nのぞき見たりき彼の夜の君に",
        "柴生田稔",
      ),
      tanka(
        "やわらかく厚い果肉を掘りすすみ\\n核の付近で死んでいる虫",
        "北辻千展",
      ),
      tanka(
        "死にし子をまつたく忘れてゐる日あり\\n百日忌日(ひやくにちきじつ)にそれをしぞ嘆く",
        "吉野秀雄",
      ),
      tanka(
        "十トンの恐竜もゐしこの星に\\n四十八キロの妻生きて死す",
        "高野公彦",
      ),
      tanka(
        "生まれてはつひに死ぬてふことのみぞ\\n定めなき世に定めありける",
        "平維盛",
      ),
    ];

    text += &format!(
      "\\1\\_q{}時\\n{}",
      now.hour(),
      tanka_list.choose(&mut rand::thread_rng()).unwrap()
    );
  }

  if text.is_empty() {
    new_response_nocontent()
  } else {
    new_response_with_value(text, TranslateOption::simple_translate())
  }
}

fn tanka(text: &str, author: &str) -> String {
  format!("{}\\n\\f[align,right]({})", text, author)
}

pub fn on_surface_change(req: &Request) -> Response {
  let refs = get_references(req);
  let surface = refs[0].parse::<i32>().unwrap();

  get_global_vars().volatility.set_current_surface(surface);

  new_response_nocontent()
}
