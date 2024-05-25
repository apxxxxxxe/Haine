use crate::events::common::*;
use crate::events::first_boot::FIRST_RANDOMTALKS;
use crate::events::talk::randomtalk::random_talks;
use crate::events::talk::{register_talk_collection, TalkType, TalkingPlace};
use crate::get_touch_info;
use crate::variables::{get_global_vars, EventFlag, IDLE_THRESHOLD};
use shiorust::message::{parts::HeaderName, Request, Response};

// トーク1回あたりに上昇する没入度
const IMMERSIVE_RATE: u32 = 8;

pub fn on_ai_talk(_req: &Request) -> Response {
  let vars = get_global_vars();
  let if_consume_talk_bias = vars.volatility.idle_seconds() < IDLE_THRESHOLD;

  vars
    .volatility
    .set_last_random_talk_time(vars.volatility.ghost_up_time());

  let text_count = FIRST_RANDOMTALKS.len();
  for (i, text) in FIRST_RANDOMTALKS.iter().enumerate() {
    if !vars
      .flags()
      .check(&EventFlag::FirstRandomTalkDone(i as u32))
    {
      vars
        .flags_mut()
        .done(EventFlag::FirstRandomTalkDone(i as u32));
      let m = if i == text_count - 1 {
        let achieved_talk_types = [TalkType::SelfIntroduce, TalkType::WithYou];
        achieved_talk_types.iter().for_each(|t| {
          vars.flags_mut().done(EventFlag::TalkTypeUnlock(*t));
        });
        let achievements_messages = achieved_talk_types
          .iter()
          .map(|t| render_achievement_message(*t))
          .collect::<Vec<_>>();
        format!(
          "{}\\1\\c{}",
          text.clone(),
          &achievements_messages.join("\\n")
        )
      } else {
        text.clone()
      };
      let mut res = new_response_with_value(m, TranslateOption::with_shadow_completion());
      res.headers.insert_by_header_name(
        HeaderName::from("Marker"),
        format!("邂逅({}/{})", i + 2, text_count + 1),
      );
      return res;
    }
  }

  if vars.volatility.aroused() {
    vars.volatility.set_aroused(false);
    get_touch_info!("0headdoubleclick").reset();
    let mut talk_parts = vec![vec![
      "\\0\\![bind,ex,流血,0]h1111705ふー…………。\\n\\1ハイネは深く息を吐いた。……落ち着いたようだ。"
        .to_string(),
    ]];
    talk_parts.push(if !vars.flags().check(&EventFlag::FirstHitTalkDone) {
      vars.flags_mut().done(EventFlag::FirstHitTalkDone);
      vec!["\\0……h1111204これで終わり？そう。\\n\
        では今回は、終わりにしましょう。\\n\
        h1111211次に期待しているわ、{user_name}。"
        .to_string()]
    } else {
      vec!["\\0……h1111204もっと殴ってもよかったのに。".to_string()]
    });
    let talks = all_combo(&talk_parts);
    let choosed_talk = talks[choose_one(&talks, if_consume_talk_bias).unwrap()].to_string();
    return new_response_with_value(choosed_talk, TranslateOption::with_shadow_completion());
  }

  // 没入度を上げる
  let immersive_degrees = std::cmp::min(vars.volatility.immersive_degrees() + IMMERSIVE_RATE, 100);

  vars.set_cumulative_talk_count(vars.cumulative_talk_count() + 1);

  if immersive_degrees >= 100 {
    let (previous_talking_place, current_talking_place) = match vars.volatility.talking_place() {
      TalkingPlace::LivingRoom => (TalkingPlace::LivingRoom, TalkingPlace::Library),
      TalkingPlace::Library => (TalkingPlace::Library, TalkingPlace::LivingRoom),
    };

    let messages: Vec<String> = {
      let parts: Vec<Vec<String>> = if vars.flags().check(&EventFlag::FirstPlaceChange) {
        vars.flags_mut().done(EventFlag::FirstPlaceChange);
        vec![
          vec![format!(
            "\\0\\b[{}]h1000000……。\\1ふと目を離した間に、ハイネは姿を消していた。\\n\
            \\0\\c\\1\\c…………。\
            他の部屋を探し、\\0\\b[{}]\\1{}に入ったとき、彼女はそこにいた。\\n\
            ",
            previous_talking_place.balloon_surface(),
            current_talking_place.balloon_surface(),
            current_talking_place
          )],
          match current_talking_place {
            TalkingPlace::Library => {
              vars
                .flags_mut()
                .done(EventFlag::TalkTypeUnlock(TalkType::Abstract));
              vars
                .flags_mut()
                .done(EventFlag::TalkTypeUnlock(TalkType::Past));
              vec![format!(
                "h1111204あなた、書斎は初めてね。\\n\
                \\1……客間より少し狭い程度の間取りに、所狭しと本棚が設置されている。\\n\
                窓すら本棚に覆われていて、ハイネは蝋燭の灯りで本を読んでいるようだった。\\n\
                h1111210ここは私の私室でもあるの。\\n\
                h1111204……あなたは、本を読むのは好き？\\n\
                h1111306私は好きよ。巨人の肩に乗って遠くが見える。\\n\
                h1111305あるいは、ここではないどこかへ、遠くへ行ける。\
                h1111204あなたも自由に読み、そして考えなさい。\\n\
                h1111310ここはそういう場所よ。{}{}\
                ",
                render_achievement_message(TalkType::Abstract),
                render_achievement_message(TalkType::Past),
              )]
            }
            TalkingPlace::LivingRoom => vec!["これが表示されることはないはず".to_string()],
          },
        ]
      } else {
        vec![
          vec![format!(
            "\\0\\b[{}]h1000000……。\\n\\n\\1また、ハイネが姿を消してしまった。\\n\
            \\0\\b[{}]\\1前回のように{}を探しに行くと、彼女はそこにいた。\\n\
          ",
            previous_talking_place.balloon_surface(),
            current_talking_place.balloon_surface(),
            current_talking_place
          )],
          match current_talking_place {
            TalkingPlace::Library => vec!["\
            h1111210さて、仕切り直しましょう。\\n\
            ……h1111206もちろん、読みたい本があれば御自由にどうぞ。\
            "
            .to_string()],
            TalkingPlace::LivingRoom => vec!["\
            h1111206さあ、お茶を淹れ直させましょう。\\n\
            h1111204お席にどうぞ、お客人。\
            "
            .to_string()],
          },
        ]
      };
      all_combo(&parts)
    };

    vars.volatility.set_talking_place(current_talking_place);
    vars.volatility.set_immersive_degrees(0);

    return new_response_with_value(
      messages[choose_one(&messages, true).unwrap()].to_owned(),
      TranslateOption::with_shadow_completion(),
    );
  } else {
    vars.volatility.set_immersive_degrees(immersive_degrees);
  }

  let talks = vars
    .volatility
    .talking_place()
    .talk_types()
    .iter()
    .filter(|t| vars.flags().check(&EventFlag::TalkTypeUnlock(**t)))
    .flat_map(|t| random_talks(*t))
    .collect::<Vec<_>>();

  let choosed_talk = talks[choose_one(&talks, if_consume_talk_bias).unwrap()].clone();

  if if_consume_talk_bias {
    // ユーザが見ているときのみトークを消費する
    register_talk_collection(&choosed_talk);
  }

  let comments = [
    "霧が濃い。",
    "「主に誉れあれ。」",
    "「館近くのパン屋は絶品だった。」",
    "彼女の声は低いがよく通る。",
    "彼女の赤い瞳の奥の思考は伺い知れない。",
    "「主には秘密が多いのだ。」",
    "「主は客人をたいそうお気に入りのようだ。」",
    "「古木のように主は佇む。」",
    "「常に主に心からの賛辞を。」",
    "「街角の喫茶店は素晴らしいコーヒーを出していた。」",
    "「主の思考は大樹のように広がっている。」",
    "「主には永遠の美しさが宿っている。」",
    "「主に語りかけることは奇跡的な経験だ。」",
    "「街の端にある花屋は色とりどりの花で溢れていた。」",
    "「昔ながらの本屋は知識の宝庫だった。」",
  ];
  let comment = comments[choose_one(&comments, false).unwrap()];

  new_response_with_value(
    format!(
      "\\0\\![set,balloonnum,{}]{}",
      comment,
      choosed_talk.consume()
    ),
    TranslateOption::with_shadow_completion(),
  )
}

pub fn on_anchor_select_ex(req: &Request) -> Response {
  let refs = get_references(req);
  let id = refs[1].to_string();
  let user_dialog = refs.get(2).unwrap_or(&"").to_string();

  let mut m = String::from("\\C");
  m += "\\0\\n\\f[align,center]\\_q─\\w1──\\w1───\\w1─────\\w1────\\w1──\\w1──\\w1─\\w1─\\n\\_w[750]\\_q\\_l[@0,]";
  if !user_dialog.is_empty() {
    m += &format!("\\1『{}』\\_w[500]", user_dialog);
  }
  match id.as_str() {
    "Fastened" => {
      m += "\
      h1111205文字通りの意味よ。\\n\
      私はこの街から出られない。物理的にね。\\n\
      h1111210私の身体はここに縛られている。\\n\
      h1111205きっと、それは消滅する瞬間まで変わらないでしょう。\\n\
      ";
    }
    "Misemono" => {
      m += "\
      h1111203かつてある国で催された、\\n\
      死刑囚の遺体を使った\\n\
      「復活の奇術」という趣向。\\n\
      h1111204その異様さと目新しさから、\\n\
      誰もがそれを目的に訪れる人気の演目だった……けれど、\\n\
      h1111205一方で不安と混乱も生まれたの。\\n\
      h1113210罪人の蘇生なんて、冷静になってみれば恐ろしいものね。\\n\\n\
      h1113206見かねたお国が鎮静を促すために\\n\
      「生き返った死刑囚は再度絞首刑に処すように」\\_w[400]\\n\
      というお触れを出したのだけど、\\n\
      h1113210かえって真実味を上乗せするだけだったみたい。\
      ";
    }
    _ => return new_response_nocontent(),
  }
  new_response_with_value(m, TranslateOption::with_shadow_completion())
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::variables::get_global_vars;
  use shiorust::message::parts::*;
  use shiorust::message::Request;
  use std::collections::HashMap;

  #[test]
  fn test_aitalk() -> Result<(), Box<dyn std::error::Error>> {
    let vars = get_global_vars();
    vars.load()?;
    vars.volatility.set_idle_seconds(1);

    let req = Request {
      method: Method::GET,
      version: Version::V20,
      headers: Headers::new(),
    };
    let mut results = HashMap::new();
    for _i in 0..100 {
      let res = on_ai_talk(&req);
      let value = res.headers.get(&HeaderName::from("Value")).unwrap();
      let md5 = format!("{:x}", md5::compute(value.as_bytes()));
      let n = results.get(&md5).unwrap_or(&0);
      results.insert(md5, n + 1);
    }
    for (k, v) in results.iter() {
      println!("{}: {}", k, v);
    }
    Ok(())
  }
}
