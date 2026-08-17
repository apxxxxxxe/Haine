use crate::get_write;
use crate::system::windows::get_local_time;
use crate::LAST_SELFTALK_PHRASE;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;

use crate::system::variables::{get_read, GHOST_UP_TIME};

use crate::events::talk::{Talk, TalkType};

use super::DerivaliveTalk;

// 私/主: 50代の身綺麗な男
// 僕/主様: 30代のおとなしい男
// わたし/主さま: 20代の活発な女
// ぼく/ご主人さま: 10代の男の子
pub(crate) const RANDOMTALK_COMMENTS_LIVING_ROOM: [&str; 11] = [
  "霧が濃い。",
  "彼女の声は低いがよく通る。",
  "彼女の赤い瞳の奥の思考は伺い知れない。",
  "「主　に　誉れあれ」",
  "「主は客人　お気に入りのようだ」",
  "「主様　秘密が多い　僕等も知らない」",
  "「主さま　ここの誰　よりも美しい」",
  "「主さま　我儘　そんなところも　好き」",
  "「主さ　は　優しいひと」",
  "「かけっこ　ご主人さま　遅い」",
  "「ご主人さま　元気ない　たまに」",
];

pub(crate) const RANDOMTALK_COMMENTS_LIBRARY_INACTIVE: [&str; 6] = [
  "薄暗い中に、彼女の声だけが響く。",
  "彼女の目は、ここではないどこかを見ているようだ。",
  "",
  "",
  "",
  "",
];

fn is_near_night() -> bool {
  let st = get_local_time();
  let hour = st.wHour;
  (17..=19).contains(&hour)
}

fn is_night() -> bool {
  let st = get_local_time();
  let hour = st.wHour;
  hour <= 3 || hour >= 19
}

fn is_winter() -> bool {
  let st = get_local_time();
  let month = st.wMonth;
  month == 12 || month <= 2
}

struct RandomTalk {
  id: String,
  text: String,
  required_condition: Option<fn() -> bool>,
  callback: Option<fn()>,
}

pub(crate) fn random_talks(talk_type: TalkType) -> Option<Vec<Talk>> {
  let strings: Vec<RandomTalk> = match talk_type {
    TalkType::AboutMe => vec![
      RandomTalk {
        id: "別れの悲しみ".to_string(),
        text: "\
          h1111110「別れがこんなに悲しいなら、\\n\
          最初から出会わなければよかった」\\n\
          h1111205……使い古された句よ。\\n\
          h1111210その通りだと思う日も、あるわ。\\n\\n[half]\
          h1111206それでも戸に鍵を掛けないのだから、\\n\
          私もいいかげんなものね。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      // - 霊は姿を変えることはできない
      // - ハイネは人目を気にして外出を避けている
      RandomTalk {
        id: "姿は変えられない".to_string(),
        text: "\
          h1111306霊は不定形だけれど、\\n\
          自由に形を変えられるわけではないわ。\\n\
          h1111310魂の形は一つしかない。\\n\
          変えられるとしたら、\\n\
          自分が誰かもわからなくなってしまった者ね。\\n\\n[half]\
          h1111206だから、私が昼に出歩くことはないわ。\\n\
          10年、20年経とうが姿の変わらない女。\\n\
          h1111310余計な面倒は避けるに越したことはないもの。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      // - ハイネは服装には無頓着
      RandomTalk {
        id: "服装へのこだわり".to_string(),
        text: "\
          h1111203服装にはどちらかというと無頓着なの。\\n\
          h1112305一度決めた「いつもの」を守り続けるだけ。\\n\
          h1112304そうすれば、余計なことを考えなくて良くなるわ。\\n\
          h1111210私のような霊に特有の悩みよ。\\n\
          h1111204低級霊はそもそも実体を持たないから、ね。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      // - ハイネは恋愛とは無縁の人生だった
      RandomTalk {
        id: "恋愛観".to_string(),
        text: "\
          h1111205幽霊は生前の想い……好みや恨みに執着するの。\\n\
          h1111210想い人がいればその人に、\\n\
          恨みがあればその相手に。\\n\
          h1111203逆に、死後新たな執着が生まれることは\\n\
          ほとんどないわ。\\n\
          だから幽霊同士、h1111206ましてや\\n\
          人と幽霊の間に恋愛が生まれることは\\n\
          皆無といっていいでしょう。\\n\\n[half]\
          h1111304……なに、その顔は。h1111310あいにく、\\n\
          私は生きていた頃から恋愛とは無縁よ。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      // - ハイネは強い霊
      // - ハイネは霊たちに慕われている
      RandomTalk {
        id: "霊力の多寡".to_string(),
        text: "\
          h1111204霊力の多寡は年月や才能、\\n\
          特別な契約の有無などで変わるけれど、\\n\
          最も大きな要因は環境──\\n\
          つまり、その地との関わりの深さによるの。\\n\
          h1111310私のように生家に根付いた霊は言わずもがな。\\n\
          h1111205……まあ、強いからといって\\n\
          良いことばかりでもないわ。\\n\
          h1111203霊にも社会がある。h1111205\\_a[AnchorTalk,NoblesseOblige,義務ってどんなこと？]上位者の義務\\_aというものも。\\n\\n[half]\
          h1111210長く、強くあるほど、\\n\
          消えていく者を見送る数も増える。\\n\
          ……h1111205いちいち悼んでいては、もたないわ。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      // - この街には霊が集まりやすい
      RandomTalk {
        id: "カンテルベリオという土壌".to_string(),
        text: "\
          h1111203カンテルベリオには、霊……正確には、\\n\
          死の意識が集まりやすい土壌があるの。\\n\
          ……h1111210あなたがここに来たのも、\\n\
          偶然ではないのかもしれないわね。\\n\\n[half]\
          この出会いが良きものでありますように。\\n\
          h1111305祈っておきましょう、お互いのために。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      // - ここはハイネの生家
      RandomTalk {
        id: "生家の広さ".to_string(),
        text: "\
          h1111210ここは私の生家なの。実際は別荘なのだけど。\\n\
          h1111206知っての通り、\\n従者がいなければ掃除が行き届かないほど広いの。\\n\
          h1111205……まあ、\\_a[AnchorTalk,LiveHome,別荘だけど長く住んでいたの？]勝手知ったる場所\\_aなのは\\n\
          不幸中の幸い、といえなくもないかしらね。\\n\
          h1111210くつろいで暮らすのにこれ以上の場所はないわ。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      // - ハイネは生家からあまり離れられない
      RandomTalk {
        id: "フィクションの価値".to_string(),
        text: "\
          h1111210良質なフィクションは現実を忘れさせてくれる。\\n\
          h1111205どこにでもついて回るはずの\\n\
          自己の存在を忘れ、\\n\
          つかの間であれどその外側へ行けるの。\\n\\n[half]\
          h1112310それは欠かせない体験だわ。\\n\
          h1112306出歩くのにも苦労する身体には、なおさら。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      // - 家はいとこの子孫が管理している
      // - いとこは帰っていない
      RandomTalk {
        id: "生活と人間との折り合い".to_string(),
        text: "\
          h1111206この家は、今は私の家の子孫が管理しているの。\\n\
          厳密には、いとこの子孫がね。\\n\
          h1111210ずいぶん帰っていないし、管理もおざなりよ。\\n\
          h1111204……まあ、\\_a[AnchorTalk,Poltergeist,物の配置が変わってたりしたら怪しまれない？]好き勝手にできる\\_aのは楽でいいわね。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      // - ハイネは生前から内向的だった
      // - 社交の場でも本を読んでいた
      RandomTalk {
        id: "生前の社交性".to_string(),
        text: "\
          h1111210生前は、社交的とは言えなかったわ。\\n\
          h1111205親戚の集まりや家庭教師との勉強会で\\n\
          人が集まる機会はあったけれど、\\n\
          h1111206会話に参加するより、\\n\
          持参した本を読んでいることの方が多かった。\\n\\n[half]\
          h1111205「失礼な子だ」と言われることもあったけれど……\\n\
          h1121210仕方ないわよね？つまらないのだから。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      // ハイネはインターネットにあえて触れていない
      RandomTalk {
        id: "スマホとインターネット".to_string(),
        text: "\
          h1111205近頃の携帯電話というのは随分便利なのね。\\n\
          写真はもはや当然で、\\nインターネットすら常に使えるなんて。\\n\\n[half]\
          h1111101私？h1111206私は使わないことにしているの。\\n\
          情報が多すぎて、速すぎて……\\n\
          h1111205一度に沢山のことが押し寄せてくる環境は、\\n\
          どうも私の性質に合わないのよ。\\n\
          h1111210静かに、一つずつ考えていく方が好きなの。\\n\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      // 泳げないまま死んだ、今さら溺れないのに気になる
      RandomTalk {
        id: "泳げない".to_string(),
        text: "\
          h1111206私、泳げないの。\\n\
          生前も習う機会がなくて。\\n\\n[half]\
          h1111205今さら溺れることはないのだけれど、\\n\
          水に入ったら……h1111210すり抜けるのか、浮くのか。\\n\\n[half]\
          h1113306……何か試す方法はないかしら。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      // 毎朝窓を開ける儀式的な習慣
      RandomTalk {
        id: "早起きの窓".to_string(),
        text: "\
          h1111210毎朝、日の出の頃に窓を開けるの。\\n\\n[half]\
          h1111206本来そんな必要はないのよ。\\n\
          空気の入れ替えなんて、霊体には関係ないもの。\\n\\n[half]\
          h1111210それでも何十年も続けていると、\\n\
          もう儀式みたいなもの……だったのだけれど。\\n\
          h1111204ここに来て意味が出てくるなんて、\\n\
          ままならないものよね。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      // 一人の時間が長いと声に出して考える癖がつく
      RandomTalk {
        id: "独り言の癖".to_string(),
        text: "\
          h1113105……{last_selftalk_phrase}……。\\n\
          \\1独り言……？\\n\
          h1121204……聞こえていたの？\\n\\n[half]\
          h1111206癖なのよ。\\n\
          一人の時間が長いせいか、\\n\
          考えをまとめようとすると声に出るの。\\n\\n[half]\
          h1111210別におかしなことは言っていなかったでしょう。\\n\
          h1121304……言っていなかったわよね？\
          "
        .to_string(),
        required_condition: Some(|| {
          let a: [&str; 3] = ["それは死人の", "ペン先", "違う、それは"];
          let mut rng = thread_rng();
          let choosed = a.choose(&mut rng).unwrap_or(&"");
          *get_write(&LAST_SELFTALK_PHRASE) = choosed.to_string();
          !choosed.is_empty()
        }),
        callback: None,
      },
      RandomTalk {
        id: "館の静寂".to_string(),
        text: "\
          h1111206夜の館は、静寂が深いでしょう。\\n\
          石の壁が音を吸い込むの。\\n\\n[half]\
          h1111210一人でいた頃は、よくこの静寂の中に、\\n\
          遠くで誰かが笑う声や、\\n\
          廊下を歩く足音が、\\n\
          h1111205混じって聞こえたものよ。\\n\
          h1111206どうも、気のせいと言うにはあまりに鮮明でね。\\n\\n[half]\
          h1111204……近頃は、あまり聞こえないの。\
          "
        .to_string(),
        required_condition: Some(is_night),
        callback: None,
      },
      RandomTalk {
        id: "記憶の整理".to_string(),
        text: "\
          h1111206死んでから、生前の記憶を\\n\
          整理する時間がたくさんあったの。\\n\
          h1111210出来事を順序立てて、感情を分析して、\\n\
          客観的に見つめ直して。\\n\\n[half]\
          h1111205当時は理解できなかった人の行動も、\\n\
          今なら合点がいくものが多いわ。\\n\
          怒りも、悲しみも、随分と色褪せた。\\n\
          h1111310そして凪いだ心中は……h1111305穏やかすぎるくらいね。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "科学への興味".to_string(),
        text: "\
          h1111210生きていた頃、科学に夢中だったわ。\\n\
          h1111206物質の構造、宇宙の謎、生命の起源。\\n\
          一見して無秩序なものたちが、\\n\
          単純な秩序で結びついている。\\n\
          h1111205目が覚めるように美しい、と思っていた。\\n\\n[half]\
          h1111206……でもね、時代が悪かったわ。\\n\
          h1111210女が夢中になるものではない、と\\n\
          何度言われたかしら。\\n\\n[half]\
          h1111204あなたの時代はどう？\\n\
          ……h1111205そう。h1111310それは、いいことだわ。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      // 外から帰ってきたタイミングのトークに使えそう
      // RandomTalk {
      //   id: "身体が弱い".to_string(),
      //   text: "\
      //     h1111105\\1なんとなく、ハイネの様子がおかしい。\\n\
      //     椅子に座ったまま動かないのだ。\\n\
      //     近づこうとすると──\\n\
      //     h1111106……いいから。そのまま座っていて。\\n\\n[half]\
      //     h1111205外は、疲れるの。\\n\
      //     生前からそうだったわ。\\n\\n[half]\
      //     h1111210おかしいでしょう。\\n\
      //     h1111206もう身体なんてないのに、\\n\
      //     まだ疲れるのよ。\
      //   ".to_string(),
      //   required_condition: None,
      //   callback: None,
      // },
      RandomTalk {
        id: "刺繍のハンカチ".to_string(),
        text: "\
          h1111105\\1ハイネが書き終えて、ペンを置いた。\\n\
          指先にインクが付いているのに気づいたようだ。\\n\\n[half]\
          \\1ポケットからハンカチを取り出して、そっと拭う。\\n\
          白い布に、小さな花模様の刺繍が入っている。\\n\
          \\1近くで見ると、花の形が少し歪んでいる。\\n\\n[half]\
          h1111204……ずいぶん昔のものよ。\\n\
          h1111205家政婦は、あまり得意じゃなかったみたいでね。\\n\
          h1111210形が歪んでいるけれど、それが好きなの。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      // - ハイネは虫が苦手（幼少時に極彩色の毛虫に刺された）
      RandomTalk {
        id: "虫が苦手".to_string(),
        text: "\
          h1111206私、虫が苦手なの。\\n\
          とりわけ、毛虫はいけないわ。\\n\\n[half]\
          h1121205幼い頃、極彩色のものに刺されて、\\n\
          三日三晩うなされたことがあってね。\\n\\n[half]\
          h1121210もう刺される肌もないのに、\\n\
          h1123306見つけると、今でも身がすくむの。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
    ],
    TalkType::WithYou => vec![
      RandomTalk {
        id: "食事中の読書".to_string(),
        text: "\
          h1111206……。\\n\
          \\1ハイネは器用に菓子を切り分け、口に運びながら、\\n\
          もう片方の手に開いた本を読んでいる。\
          h1111305……何か言いたそうね。\\n\
          h1111310分かっているわ。お行儀が悪いって。\\n\\n[half]\
          昔はこっぴどく怒られたもの。\\n\
          h1111211だから今やるのよ。鬼の居ぬ間にね。\\n\
          h1111306あなたもまた鬼なのならば、話は別だけれど。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "舌やけど".to_string(),
        text: "\
          h1113105……。\\n\
          \\1本を読みながら、\\n\
          ハイネが淹れたてのお茶を口に運ぶ。\\n\
          h1000000っ……熱……。\\n\\n[half]\
          h1121210はあ。h1121205ぼんやりしているとよくやるの。\\n\
          ……h1111206続きが、気になるところだったのよ。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "リップクリーム".to_string(),
        text: "\
          h1111201あら、それは何？\\n\
          h1111101リップクリーム……h1111104唇の保湿をするのね。\\n\\n[half]\
          h1111204それ、借りても良いかしら？初めて見たの。\\n\
          h1000000\\1スティックタイプのものを渡すと、\\n\
          ハイネは見様見真似で自分の唇に塗る。\\n\\n[half]\
          塗り終えると、唇を小指で軽く拭った。\
          h1113102\\n[half]ふむ……。油分で覆って乾燥を防ぐ。\\n\
          h1213205保湿のためとはいえ、べたつくのは少し嫌ね。\\n\\n[half]\
          h1111204ありがとう、返すわ。h1111205\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "ピアス".to_string(),
        text: "\
          h1111201あなたはいつも耳飾りをしているのね。\\n\
          h1111202よく、見せてくれる？h1000000\\n\\n[half]\
          \\1耳元を覗き込んで、ハイネは目を細めた。\\n\
          h1111105耳に穴を開けているのね。\\n\\n[half]\
          h1111206昔、母が開けるのを見たことがあるけれど、\\n\
          h1111210ひどく顔が歪んでいたのを覚えているわ。\\n\\n[half]\
          h1111204開けるときは、痛かったでしょう。\\n\
          h1111104……そうでもない？……h1111210そう、技術の進歩ね。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "くしゃみ".to_string(),
        text: "\
          h1000000……っくしゅん！\\n\\n[half]\
          ……h1112105ごめんなさい、ホコリか何かが……。\\n\
          h1111106空気を入れ替えましょう。\\n\\n[half]\
          \\1窓がかすかに開く。\\n\
          新鮮な空気がすっと入ってきた。\\n\\n[half]\
          『くしゃみするとき、目閉じないんだね』\
          h1111101……h1111102言われてみれば、そうね。\\n\
          h1113105もしかして、昔から？\\n\
          h1121204……あなた、よく見ているのね。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "食べ物の好き嫌い".to_string(),
        text: "\
          h1111205あら、レーズンは嫌い？\\n\
          嫌なら端によけておいてちょうだい。\\n\
          h1111206あとで、誰か欲しがるものがいるかも知れないわ。\\n\\n[half]\
          h1111210ちなみに私も嫌い。\\n\
          ……h1111304けれど、そうね、今は挑戦したい気分だわ。\\n\
          h1111206昔は気まぐれだと文句を言われたものだけれど、\\n\
          そのとき食べたくなったのだから仕方ないわよね。\\n\\n[half]\
          h1111205ねえ、h1111310それ、いただいても良いかしら？\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "蝋燭の交換".to_string(),
        text: "\
          h1111105\\1燃え尽きた蝋燭の欠片を、ハイネが素手で摘む。\\n\
          燭台の芯の周りにこびりついた蝋を、\\n\
          指でこそいでいる……。\\n\\n[half]\
          h1111101\\1『熱くないの？』\\n\\n[half]\
          h1111205平気よ。慣れてるから。\\n\
          \\1慣れの問題なのだろうか……。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      // - 幽霊は写真に写らない
      // - ハイネは現代の知識を持っている
      RandomTalk {
        id: "写真には写らない".to_string(),
        text: "\
          h1111210今は手軽に写真が撮れていいわね。\\n\
          h1111205印象的な光景を、いつでも手元に残しておける。\\n\\n[half]\
          ……h1111201あら、私？h1121210光栄だけれど、\\n\
          残念ながら写真には写らないわ。\\n\
          h1113206その点では、\\n\
          彼らのほうが望みがあるというのだから不思議ね。\\n\
          h1113210見たことあるでしょう？写真の背後に白い影……。\\n\
          h1113306どういうわけか、低級霊はたまに写るのよ。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      // - この街の霧は霊的なもの
      // - この街では霊が活発になる
      RandomTalk {
        id: "霧の力".to_string(),
        text: "\
          h1111206霧が、濃いでしょう。\\n\
          ただの霧ではないの。乾いた霧よ。\\n\
          むしろ、性質としては私たちに近い。\\n\
          h1111210ただの霊である私がここまで力を持っているのも、\\n\
          この地に根付いているもののおかげ。\\n\\n[half]\
          h1111206霧の濃い日は彼らも元気よ。\\n\
          私もいくらか身体が楽。\\n\
          h1111306生きた人々は厄介そうにしているけれどね。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      // - ユーザはゴスファッションをしている
      // - ハイネは個性的なファッションを重んじる
      RandomTalk {
        id: "あなたのゴスファッション".to_string(),
        text: "\
          h1111201あなたのその装い……\\n\
          ゴス・ファッション、と言うのよね。\\n\
          h1111202首元の十字架も、爪の先も、黒で。\\n\\n[half]\
          h1111205……黒は、服喪の色。\\n\
          h1111206私の頃は、進んで着るものではなかったわ。\\n\\n[half]\
          h1111204それをあなたは自分で選んで着ている。\\n\
          h1111205私の知らない黒色ね。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      // - ハイネは生前食が細かった(作業に没頭していると食事を忘れる)
      // - ハイネは生前家政婦を雇っていた
      RandomTalk {
        id: "生前の食事事情".to_string(),
        text: "\
          h1111204あなたは\\_a[AnchorTalk,LikeTheGranma,なんだかおばあちゃんみたい]ちゃんと食べているかしら？\\_a\\n\
          h1111210そう。いいことね。\\n\
          h1111104私？……h1111205生前は食が細かったわ。\\n\
          h1111210……身体が弱い上に、\\n\
          食そのものにあまり関心がなくてね。\\n\
          h1111205何かに没頭していると、\\n\
          食事をとる時間も惜しく思えてしまって。\\n\
          h1123310思えば、家政婦には随分と世話をかけたものね。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      // - ユーザは絵が得意
      // - ハイネの生きていた時代には肖像画は珍しかった
      RandomTalk {
        id: "スケッチ".to_string(),
        text: "\
          h1111205……h1111201あら、絵を描いているの？見せて。\\n\
          h1111305へえ、上手なのね。h1111202……これは、私？\\n\
          ……h1111205ふうん。こんなふうに見えているのね。\\n\\n[half]\
          h1111101…………h1111204いいえ、いいのよ。\\n\
          h1111205絵に描いてもらえるなんて、\\n\
          私の生きていた頃から考えれば\\n\
          願ってもないことだもの。\\n\
          h1111210描きあげたら、また見せてちょうだい。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "振り子時計の調整".to_string(),
        text: "\
          h1000000\\1ハイネが振り子時計の前に立っている。\\n\
          ガラスの蓋を開けて、振り子に触れた。\\n\\n[half]\
          \\0……少し遅れているの。\\n\
          \\1ねじを回す音がする。\\n\\n[half]\
          \\1……ガチ、と鈍い音がした。\\n\
          h1123105……。\\n\
          \\1さっきより振り子の動きが不安定になっている。\
          h1113106……彼に任せるべきだったかしら。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "菓子の切り分け".to_string(),
        text: "\
          h1111105\\1ハイネが菓子を切り分けている。\\n\
          一度切り、眺めて、もう一つ切り出す。\\n\\n[half]\
          h1123205……少し小さかったかしら。\\n\\n[half]\
          h1111210大きい方をあげる。\
          \\1違いがわからなかったが、\\n\
          黙って受け取ることにした。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "匂い".to_string(),
        text: "\
          h1111105\\1玄関で、濡れた上着を脱いでいると、\\n\
          h1111204……雨の匂い。\\n\
          h1111205土と、少し湿った石の。\\n\\n[half]\
          \\1ハイネが、わずかにこちらへ身を寄せた。\\n\\n[half]\
          h1111206濡れた石の匂いは、好きよ。\\n\
          h1111205雨の日は、よく窓を開けさせるの。\\n\
          h1111210……今日は、その必要もなさそうね。\
          "
        .to_string(),
        required_condition: Some(|| *get_read(&GHOST_UP_TIME) < 60 * 15), // 起動から15分以内限定のトーク
        callback: None,
      },
      RandomTalk {
        id: "袖のほつれ".to_string(),
        text: "\
          h1111106\\1ハイネの袖口から、糸がほつれて垂れている。\\n\\n[half]\
          \\1『そこ、糸が出てるよ』\\n\
          h1111205……あら、気づかなかったわ。\\n\\n[half]\
          \\1ハイネはほつれた糸の先を摘まんで、\\n\
          くるりと袖の内側に巻き込んだ。\\n\\n[half]\
          h1111206あとで従者に直させましょう。\\n\
          h1111210繕い物は、あの子の方がずっと上手なの。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "携帯電話を使わない".to_string(),
        text: "\
          h1111204あなた、携帯電話をあまり使わないわよね。\\n\
          h1113106以前言っていたでしょう？\\n\
          現代人は携帯電話を手放せないって。\\n\
          h1111101\\1『今はハイネとの話に集中したいから』\\n\\n[half]\
          h1111204……そういうものかしら。\\n\
          \\1『あと、\\n\
          ハイネなら絶対興味津々になっちゃうと思って』\\n\
          h1121210そっちが本音ね？\\n\
          ……まったく、h1121306よく分かってるじゃない。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "生きた字".to_string(),
        text: "\
          h1111105\\1ペンを走らせていると、\\n\
          ハイネが手元をのぞき込んだ。\\n\
          \\0h1111210……死んだ者の字ばかり、読んでいたの。\\n\
          h1111206どれも掠れて、途中で力尽きている。\\n\\n[half]\
          h1111205あなたのは、急いて、跳ねて、そして穏やかで。\\n\
          h1111210ひと文字ずつ息づいているようだわ。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "白くない息".to_string(),
        text: "\
          h1111206\\1寒い日だった。吐く息が、白く立つ。\\n\\n[half]\
          \\1向かいのハイネの口元からは、\\n\
          それが立ちのぼらない。\\n\\n[half]\
          h1111201『息、白くないんだね』\\n\
          h1111205ええ。冷えたものが、冷えたまま出てゆくだけ。\\n\
          温める火が、私の中にはないの。\\n\\n[half]\
          h1111210……あなたの息は、ずいぶん暖かそうね。\\1\\n\
          \\1白い息が、急に生々しく見えた。\
          "
        .to_string(),
        required_condition: Some(is_winter),
        callback: None,
      },
      RandomTalk {
        id: "夜の灯り".to_string(),
        text: "\
          h1111106\\1日が暮れて、手元が暗くなってきた。\\n\
          かまわず描き続けていると、\\n\
          h1111205\\0——まだ描くのでしょう？\\n\\n[half]\
          h1000000\\1燭台がひとつ、テーブルに増えた。\\n\
          h1111210暗がりで描いては、目を悪くするわ。\\n\\n[half]\
          \\1『蝋燭、もったいなくない？』\\n\
          h1111204客に灯りを惜しむ家があるものですか。\\n\\n[half]\
          h1111210私も昔、暗がりで本を読んでは\\n\
          散々叱られたものよ。h1111205……さあ、続けて。\
          "
        .to_string(),
        required_condition: Some(is_near_night),
        callback: None,
      },
      // RandomTalk {
      //   id: "".to_string(),
      //   text: "\
      //     \\1少し喉に粉っぽい感覚があり、咳払いをした。\
      //     h1111101喉の調子が悪いの？埃が舞っていたかしら。\\n\
      //     h1111106あなたが来てからは掃除と換気を\\n\
      //     余計にさせているのだけど……。\\n\\n\
      //     h1111205ひとまず、飴を渡しておくわ。\\n\
      //     すこしはましになるはずよ。\\n\
      //     \\1飴？水ではなく…？\\n\
      //     h1111104……？飴は嫌いだったかしら。\
      //     \\1……後で知ったが、\\n\
      //     ハイネの時代に飲み水は今ほど豊富でなく、\\n\
      //     口直しには安価な飴が一般的だったらしい。\
      //     "
      //   .to_string(),
      //   required_condition: None,
      //   callback: None,
      // },
    ],
    TalkType::Lore => vec![
      RandomTalk {
        id: "冥界の渡し賃".to_string(),
        text: "\
          h1111206古代ギリシャでは、\\n\
          死者に銅貨を持たせて葬っていたの。\\n\
          h1111210冥界には川を渡っていかなければ\\n\
          ならなかったから、\\n\
          渡し賃を持たせて快適な旅を願っていたのよ。\\n\\n[half]\
          h1111205死者が川を越えていくという伝承は\\n\
          世界中で見られるわ。彼らにとって、\\n\
          境界線といえばまず川が連想されたのかしら。\\n\\n[half]\
          h1111210あなたなら、\\n\
          あの世とこの世の間にはなにがあると思う？\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "死体のうめき声".to_string(),
        text: "\
          h1111205死体は、うめき声を上げることがあるのよ。\\n\
          h1111206……といっても、\\n\
          体内のガスが口から噴き出すとき、\\n\
          声帯が震えて音が出る……\\n\
          ただそれだけのことなのだけど。\\n\
          それでも、そんな些細なことが恐怖をかきたてて、\\n\
          人々は怪物を想像する。\\n\
          ……h1111201呆れるほどに多彩で、\\n\
          身近に根ざした感情の象徴だわ。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "屍蝋".to_string(),
        text: "\
          h1111205屍蝋、って聞いたことあるかしら？\\n\
          h1111210死体の脂肪分が蝋状に変質した状態のこと。\\n\
          h1111206保存状態にもよるけれど、\\n\
          腐りもミイラ化もしない、\\n\
          生前の姿が比較的綺麗に残った状態と\\n\
          言われているわ。\\n\\n[half]\
          h1111205珍しい現象だからかしらね。屍蝋化した死体は、\\n\
          地域によってさまざまな扱いを受けてきたわ。\\n\
          h1111210土に還らないことから、大地が拒んでいる……\\n\
          つまり悪霊になっているとして\\n\
          焼かれることもあれば、\\n\
          h1111204神が起こした奇跡として\\n\
          大切に扱われることもあるの。\\n\
          ……どちらにせよ、\\n\
          ふつうの葬送は望めなさそうね。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "死後の温かさ".to_string(),
        text: "\
          h1111205死後数日経ったはずの身体が、まだ温かい。\\n\
          h1111210それは、\\n\
          微生物が分解を行ったときに生じた熱のせいよ。\\n\
          ガスで膨張もするから、\\n\
          生前よりふくよかで健康的に見えることすら\\n\
          あったみたい。\\n\
          ……h1111204死体が蘇って夜な夜な彷徨い歩く、\\n\
          あるいは夢枕に立って生命を吸い取る\\n\
          という民話は、そんな様子に理由をつけたもの\\n\
          だったのではないかしら。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "生長する死体".to_string(),
        text: "\
          h1111205掘り起こした死体の髪や爪が伸びていた！\\n\
          h1111210土葬が一般的だった時代、たびたびあった話。\\n\
          乾燥して縮むから、皮膚の下の髪や爪が露出する。\\n\
          それがまるで生長しているように見えたの。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "土葬の空洞".to_string(),
        text: "\
          h1111206土葬の場合、地中の遺体が朽ちた後には\\n\
          空洞ができるわ。\\n\
          h1111205「死体に足を引っ張られる」という伝承は、\\n\
          これを踏み抜いてしまっただけかもしれないわね。\\n\
          h1111210あなたも墓地を歩くときは気をつけて……って、\\n\
          h1111204あなたの住む場所にそんなところは少ないかしら。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "永遠の夢".to_string(),
        text: "\
          h1113105恒久の平和、不死の身体、永劫の繁栄……。\\n\
          h1113204永遠を夢見た人物の多くは失敗していて、\\n\
          その代償を払っている。\\n\
          h1113210寓話のモチーフとしての話よ。\\n\
          ……h1113106求めるのは、\\n\
          ほんとうに間違ったことなのかしら？\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "生体電気".to_string(),
        text: "\
          h1111206カエルの足に電流を流す実験。\\n\
          生体電気の発見に繋がったといわれる\\n\
          あの現象は、\\_a[AnchorTalk,Misemono,どんな見世物だったの？]死者の蘇りを謳う見世物\\_aに\\n\
          利用されたことがあったの。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "死者の埋葬".to_string(),
        text: "\
          h1111206古代ギリシャにおける刑死の際、\\n\
          毒薬に阿片を混ぜたものを飲ませていたの。\\n\
          h1113210それは死の苦しみを和らげるため\\n\
          だったのでしょうけれど、\\n\
          それ以上に、死を恐れる人々を抑えるため\\n\
          だったのではないかと思っているの。\\n\
          h1113205罰ではあれど、\\n\
          必要以上に苦しませることはない、と。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "黒死病".to_string(),
        text: "\
          h1111210黒死病が蔓延していたとき、\\n\
          問題になっていたのがいわゆる「早すぎた埋葬」。\\n\
          h1111205ある技師は生き埋めにされる恐怖から逃れるため、\\n\
          埋葬者が生きていることを\\n\
          棺の内側から知らせる仕組みがついた棺を\\n\
          開発したの。\\n\
          h1111204彼、デモンストレーションのために\\n\
          自ら生き埋めになってみせたそうよ。\\n\
          h1212210自分で出られない状態で、冷たい土の下へ。\\n\
          h1211306どんな心地がしたのかしらね。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "鏡を覆う".to_string(),
        text: "\
          h1111206死者が出た家では、\
          鏡を布で覆う風習があるのよ。\\n\
          h1111210抜け出た魂が鏡に閉じ込められてしまう、\\n\
          という恐れからね。\\n\\n[half]\
          h1111205今でも残っている地域があるわ。\\n\
          迷信と分かっていても、\\n\
          万が一を思えば布一枚くらい安いもの。\\n\\n[half]\
          h1111204……ちなみに、私は映るわよ。\\n\
          h1111210魂の在り処は一体どこなのかしらね。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "人魂の正体".to_string(),
        text: "\
          h1111205墓場でふわりと浮かぶ青白い光。\\n\
          人魂として恐れられてきたけれど、\\n\
          h1111210あれはリンの化合物が\\n\
          自然発火しているだけなの。\\n\
          h1111206遺体から染み出たリンが空気に触れて燃えるのよ。\\n\\n[half]\
          h1111210本物の幽霊は、あんなに目立つことはしないわ。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "満月と狂気".to_string(),
        text: "\
          h1111206「狂人」を意味する言葉の多くは、\\n\
          月に由来しているの。\\n\
          h1111210満月の夜に人が狂うという信仰は\\n\
          古代から続いているわ。\\n\\n[half]\
          h1111205科学的には否定されているけれど、\\n\
          h1111206……暗闇の中で唯一明るい夜は、\\n\
          眠れない人が増える。\\n\
          h1111210それだけで十分だったのだと思うわ。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "死者の名前".to_string(),
        text: "\
          h1111205死者の名前をみだりに呼んではいけない、\\n\
          という禁忌は世界中にあるわ。\\n\
          h1111210名前には力があって、\\n\
          呼ばれると霊が引き寄せられる、と。\\n\\n[half]\
          h1111206実際、呼ばれて来る霊は\\n\
          いないこともないのだけれど……。\\n\
          h1111204わざわざ来るのは大抵、\\n\
          暇を持て余しているだけよ。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "鐘の音".to_string(),
        text: "\
          h1111206死にゆく人のために鐘を鳴らす、\\n\
          という慣わしがあったのよ。\\n\
          h1111210パッシングベルと呼ばれていたわ。\\n\
          悪霊を遠ざけるためとも、\\n\
          魂の旅立ちを知らせるためとも。\\n\\n[half]\
          h1111206……この街の修道院にも、\\n\
          鐘塔だけが残っていたわね。\\n\
          h1111205あの鐘が最後に鳴ったのは、\\n\
          いつだったかしら。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "動物磁気".to_string(),
        text: "\
          h1111206催眠術の元になった\\n\
          「動物磁気」という概念があるの。\\n\
          h1111210人体には目に見えない流体が流れていて、\\n\
          それを操ることで病を治せる、という説よ。\\n\
          h1111205当時は真剣に研究されていたけれど、\\n\
          後に否定されたわ。\\n\\n[half]\
          h1111204……目に見えないもので病を治すという発想は、\\n\
          h1111206今も形を変えて残っているわね。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "髪の装身具".to_string(),
        text: "\
          h1111206死者の髪で装身具を作る風習があったのよ。\\n\
          h1111210ブレスレット、ロケット、指輪。\\n\
          h1111205故人の髪を編み込んで、\\n\
          肌身離さず持ち歩いたの。\\n\\n[half]\
          h1111206髪は朽ちないもの。\\n\
          写真がなかった時代の、\\n\
          一番確かな形見だったのよ。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "デスマスク".to_string(),
        text: "\
          h1111206デスマスク、って知っているかしら。\\n\
          h1111210亡くなったばかりの人の顔に、\\n\
          石膏を流し込んで型を取るのよ。\\n\\n[half]\
          h1111205写真のない時代に、\\n\
          死者の顔を留めておく方法のひとつだったの。\\n\
          h1111206有名な作曲家や哲学者のものが、\\n\
          博物館に残っていたりするわね。\\n\\n[half]\
          h1111210……私のそれが作られたのかは分からない。\\n\
          h1111305……まあ、現存していたとしたら\\n\
          一目見てみたくはあるわね。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "蜂に告げる".to_string(),
        text: "\
          h1111206蜂の巣箱に黒い布が掛かっていたら、\\n\
          その家で誰かが死んだという印よ。\\n\
          h1111210家の者は巣箱を叩いて、誰が死んだかを告げて、\\n\
          葬式の菓子と葡萄酒を、蜂にも分けるの。\\n\
          h1111205知らせを怠ると、蜂は機嫌を損ねて、\\n\
          巣ごといなくなってしまうのですって。\\n\\n[half]\
          h1111204実際、世話する人が死んだ巣は、\\n\
          放っておけば本当に空になるもの。\\n\
          h1111210告げるというのは、\\n\
          次の世話人が巣の前に立つということなのね。\\n\
          ……h1111206案外、蜂の方こそ、\\n\
          人をよく見ているのかもしれないわね。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
    ],
    TalkType::Servant => vec![
      RandomTalk {
        id: "霊力と可視性".to_string(),
        text: "\
          h1111206\\1ポットがひとりでに浮き、\\n\
          空になっていたカップに飲み物が注がれる。\\n\
          \\0……h1111204私が見えて彼らが見えないのは、\\n\
          霊としての力量の違いよ。\\n\
          h1111206強い霊力があれば\\n\
          あなたのような人間の目にも見えるし、\\n\
          物理的な接触も可能になるの。\\n\\n[half]\
          h1111204……つまり、彼らのように霊力が弱ければ、\\n\
          誰かさんにべたべたと触られることも\\n\
          なかったということよ。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "低級霊との契約".to_string(),
        text: "\
          h1111206\\1ポットがひとりでに浮き、\\n\
          空になっていたカップに飲み物が注がれる。\\n\
          h1111206私の元へ集うのは弱い人たち。\\n\
          自分だけでは溶けゆく自我を押し留められず、\\n\
          さりとてそれを受け入れることもできない霊。\\n\\n[half]\
          h1111210役割を与えてあげるの。一種の契約ね。\\n\
          h1111205使命に縛られはするけれど、\\n\
          消滅するよりはよしと彼らは決断したの。\\n\\n[half]\
          h1111206救済と言えば聞こえは良いけれど、\\n\
          実際は互いの利害が一致した取引に過ぎないわ。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "あなたの価値".to_string(),
        text: "\
          h1111101何をすればいいか？\\n\
          h1111204何も。難しいことは、何も要らないのよ。\\n\
          h1111206ただ、息をして、ここにいて。\\n\
          気が向いたら、話しかけてちょうだい。\\n\
          h1111204私はもう、息の仕方も忘れてしまったから。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "従者の記憶".to_string(),
        text: "\
          h1111206\\1見えない手が本の頁をめくっている。\\n\
          ペンが踊り、文字が書き込まれていく。\\n\
          \\0h1111210彼らは生前の記憶を文字にして残してくれるの。\\n\
          h1111205日記のような断片的なものから、\\n\
          貴重な技術資料まで様々よ。\\n\\n[half]\
          h1111206……彼らにとっても\\n\
          記録を残すことで自分の存在を\\n\
          確認できるのでしょうね。\\n\
          h1111310とはいえ、\\n\
          彼らの自我で残せる情報は限られている。\\n\
          詳しく話を聞けないのが歯がゆいわ。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
    ],
    TalkType::Past => vec![
      RandomTalk {
        id: "人ひとり".to_string(),
        text: "\
          h1111110人ひとり、殺せるとしたら誰にする？\\n\
          という他愛ない問い。\\n\
          h1111305だから私は私を殺したの。\\n\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "死体損壊".to_string(),
        text: "\
          h1111110「死体の損壊は死者への冒涜だ」\\n\
          という言説があるわね。\\n\
          h1111105当事者の視点から言うと、\\n\
          別にそうでもなかったわ。\\n\
          h1111310幽霊が元の身体に戻った例もない。\\n\
          h1111306畢竟、それは生者の問題ということね。\\n\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "惨めな人生".to_string(),
        text: "\
          h1111105みじめな人生の上に正気でいるには、\\n日々は長すぎたの。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "行き場のない苦しみ".to_string(),
        text: "\
          h1112102誰が悪い？いいえ、誰も悪くない。\\n\
          打ち明けたところで、\\n\
          的はずれな罪悪感を生むだけ。\\n\
          h1112105だからといって、\\n\
          他人に責をなすりつけるほど\\n\
          鈍くあることもできなかった。\\n\
          h1112110この気持ちには、どこにも行き場がなかったの。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "死の瞬間".to_string(),
        text: "\
          h1111105死ぬ瞬間、後悔はなかった。\\n\\n[half]\
          もう一度同じ人生を生きることができたとしても、\\n\
          私は同じことをすると断言できるわ。\\n\
          ……h1111110けれど、\\n\
          遺書くらいは書いたほうがよかったかしら。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "助けは遂げられず".to_string(),
        text: "\
          h1111105助けようとしてくれた人は沢山いたけれど、\\n\
          h1111110それが遂げられることはついぞなかったわ。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "死なない理由".to_string(),
        text: "\
          h1111110生きていて良かったと思えることは\\n\
          数えきれないほどあったわ。\\n\
          h1111105でも、死なない理由は一つも見つからなかった。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "ふつうになりたかった".to_string(),
        text: "\
          h1112110ふつうになりたかった。\\n\
          ……h1112105でも、ふつうだったら、\\n\
          もう私じゃないとも思う。\\n\
          それは私の顔をした別のだれかで、\\n\
          私は私の性質と不可分で、\\n\
          今ここにいる私は、私以外でいられない。\\n\
          h1112110だから、私として生きることができなかった私は、\\n\
          もうどこにもいられない。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "人と本".to_string(),
        text: "\
          h1111105昔から、人と本の違いがわからなかったの。\\n\
          h1111105無論、区別がつかないという意味ではなくて。\\n\
          ……h1111110人に期待するものがそれだけしか無かったの。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "今度こそ無へ".to_string(),
        text: "\
          h1111105死にぞこなったものだから、\\n\
          次の手段を求めている。\\n\
          ……h1112305今度こそ、終わらせたいものね。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "魂は消える".to_string(),
        text: "\
          h1111110未練もなく、しかし現世に留まっている魂。\\n\
          h1111105あるべきでないものはやがて消滅する。\\n\
          h1111106多少の不純物が含まれようと、\\n\
          そのルールは変わらない。\\n\
          h1111105私は、それを待ち望んでいるの。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "人生の無意味".to_string(),
        text: "\
          h1111210人生に意味などあってはならない。\\n\
          h1111204だって、そうでなければ。\\n\
          h1111205失うことに耐えられないもの。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
    ],
    TalkType::Abstract => vec![
      RandomTalk {
        id: "今ここに立っていること".to_string(),
        text: "\
          h1111310過去は記憶の中にしかない。\\n\
          h1111305未来は想像の中にしかない。\\n\
          h1112305我々が立っているのは今ここだけ。\\n\
          わたしたちが感じられるのは現在だけ。\\n\
          h1112310ひどい過去も、おぞましい未来も、\\n\
          h1112305いまわたしが立つこの瞬間には存在しないの。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "感動と倦み".to_string(),
        text: "\
          h1111105ある本を最初に読んだときの感動と、\\n\
          何度も読み返して全て見知ったゆえの倦み。\\n\
          どちらがその本の真の印象か。\\n\\n[half]\
          h1111110どちらも正しいと思う。\\n\
          h1111110印象なんてその時々で変わるもので、\\n\
          h1111105一つに定まることなんて稀だもの。\\n\\n[half]\
          まして、自分の中に秘めるものならなおさら。\\n\
          h1111306どちらか一方だけだなんて、勿体ないわ。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "納得のための因果".to_string(),
        text: "\
          h1111110因果が巡ってきた。\\n\
          過去が現在を刈り取りに来た。\\n\
          わたしは報いを受けたのだ。\\n\\n[half]\
          ……h1111105それが、\\n\
          自分を納得させるための妄想だったとしたら？\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "怖いものを見るということ".to_string(),
        text: "\
          h1111102怖いものだからこそ、見つめなければ戦えない。\\n\
          ……h1111105そんなもの、戦える人のためだけの論理だわ。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "停滞を終わらせるために".to_string(),
        text: "\
          h1111105危険と隣り合わせだからこそ、世界は美しいの。\\n\
          身を損なう心配がなくなっては、\\n\
          美しさが心を打つこともない。\\n\
          h1111105ただただ平坦な、揺らがぬ水面があるだけ。\\n\
          h1111110それはやがて、淀み、腐る。\\n\
          h1111105願わくば、せめて終わりがありますように、と。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "停滞の破壊".to_string(),
        text: "\
          h1111105人生に変化は付きもの\\n\
          ……けれどh1111110停滞はそれ以上。\\n\
          一度立ち止まってしまうと、空気は一瞬で淀んで、\\n\
          身動きがとれなくなってしまう。\\n\
          それは倦怠とも違う、鈍い痛み。\\n\
          h1111105もしそうなったときは、\\n\
          多少無理にでも変化を取り入れるの。\\n\
          ……h1111110たとえなにかを破壊することになるとしても、\\n\
          何も出来ないよりはずっとましよ。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "極限の変化としての死".to_string(),
        text: "\
          h1111105死の瞬間の、極限に振れた変化。\\n\
          命が命でなくなり、身体が陳腐な肉の塊になる、\\n\
          その一瞬が愛しくてたまらない。\\n\
          どうしようもなく、愛しいの。\\n\\n[half]\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "死の向こう側".to_string(),
        text: "\
          h1112110どうか、死の向こう側がありませんように。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "沈んでいく".to_string(),
        text: "\
          h1111105沈んでいく。\\n\
          手がどうしても動かなくて、\\n\
          目の前の希望を掴めない。\\n\
          身体が重い。浅い呼吸のなかで、\\n\
          沈んでいく自分の身体を感じていることしか\\n\
          できない。\\n\
          わたしは、わたしを救うことを諦めているみたい。\\n\
          h1111110どうして。\\n\
          h1111105どうして、こうなってしまったのだろう。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "人を解体したい".to_string(),
        text: "\
          h1111110人を解体したいと、思うことがあるの。\\n\
          何が人を人たらしめているのか、\\n\
          どこまで分解すれば人は人でなくなるのか。\\n\
          h1111105人という恐ろしく不可解な物の、\\n\
          どこにその根源があるのか。\\n\
          それを知るには、他に方法が思いつかないの。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "わがままな祈り".to_string(),
        text: "\
          h1111110がんばっているってこと、\\n\
          理解できなくても見ていてほしかったの。\\n\
          ……h1111105わがままかしら。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "生者にとっての慰め".to_string(),
        text: "\
          h1111110枯れ木に水をあげましょう。\\n\
          もはや花は見れずとも、それが慰めとなるのなら。\\n\\n[half]\
          h1111105それは誰にとって？\\n\
          h1111106無論、死を悼む者にとっての慰めよ。\\n\
          むくろに心はないもの。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "不可逆な崩壊".to_string(),
        text: "\
          h1111110燃え殻がひとりでに崩れるように、\\n\
          心が静かに割れて戻らなくなった。\\n\
          h1111105だから、諦めたの。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "中途半端な助け".to_string(),
        text: "\
          h1111110中途半端な助けは何もしないより残酷だわ。\\n\
          h1111105希望を持たせておいて、それを奪うのだもの。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "レンズの歪み".to_string(),
        text: "\
          h1111105観察と模倣を続ければ、\\n\
          完全に近づけると思っていた。\\n\
          想定外だったのは、レンズが歪んでいたことと、\\n\
          それを取り替える方法がなかったこと。\\n\
          h1111310そうなればすべてが台無し。\\n\
          h1111305望みが絶えるとはこのことね。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "先の見えない苦しみ".to_string(),
        text: "\
          h1111105一寸先は暗く、扉は閉ざされている。\\n\
          不明な道のりを諸手で探るよりも、\\n\
          h1112305目先の手首を切り裂くほうが遥かに明瞭なのだ！\\n\
          ……h1111110なんてね。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "唯一の視点".to_string(),
        text: "\
          h1111106わたしたちは、自我という色眼鏡を通してしか\\n\
          世界を観測できない。\\n\
          h1111105隣り合う二つの魂があろうとも、\\n\
          互いの内なる世界を覗き見ることは\\n\
          決してできないの。\\n\
          h1112110それって、この上なく残酷なことだわ。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "一つの個としての限界".to_string(),
        text: "\
          h1111103世界が複雑で曖昧すぎるから、\\n\
          わたしたちは認識したものを\\n\
          理解できる形に歪めてしまう。\\n\
          h1111110既存の分類に当て嵌めて、安心を優先するの。\\n\
          それは曇る視界と引き換えに。\\n\
          ……h1111105あなたには、\\n\
          わたしはどう見えているのかしら？\\n\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "自己同一性の仮定".to_string(),
        text: "\
          h1111105環境と経験の総体こそが、\\n\
          自己であるような気がするの。\\n\
          自己同一性すら偶然の産物？\\n\
          h1111110執着しているのが馬鹿馬鹿しく思えてくるわ。\\n\
          h1111105仮にそうでなければ。\\n\
          ……自己は最初から決定されている？\\n\
          h1111110それこそ、ね。\\n\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "自分の理解者は自分だけ".to_string(),
        text: "\
          h1111110「なぜみんな私をわかってくれないの？」\\n\
          と誰もが思う。\\n\
          h1111105答えは簡単。\\n\
          他人がわたしではなく、\\n\
          わたしが他人でないからよ。\\n\
          わたし以外にわたしを理解できるひとはいない。\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "得ることは失うこと".to_string(),
        text: "\
          h1111110ひとつ得るとき、ひとつ失う。\\n\
          h1111106わたしは今日、なにを失った？\\n\
          h1111105その喪失は、なにをわたしに齎した？\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
      RandomTalk {
        id: "中庸".to_string(),
        text: "\
          h1111206盲目的にすべてを行うことも、\\n\
          全く行わないことも正解ではない。\\n\
          いつだって答えは中庸。\\n\
          悩ましくて、煮えきらなくて……\\n\
          h1111210考えるって、だからこんなにも楽しいのでしょう。\\n\
          "
        .to_string(),
        required_condition: None,
        callback: None,
      },
    ],
  };

  let mut talks = Vec::new();
  for st in strings {
    if let Some(expr) = st.required_condition {
      if !expr() {
        continue;
      }
    }
    talks.push(Talk::new(
      Some(talk_type),
      st.id,
      st.text.to_string(),
      st.callback,
    ));
  }
  Some(talks)
}

pub(crate) fn derivative_talks() -> Vec<DerivaliveTalk> {
  vec![
    DerivaliveTalk {
      parent_id: "生前の記録".to_string(),
      id: "生前の記録・過去".to_string(),
      summary: "『読んでみたい』".to_string(),
      text: "\
        \\0h1111204……それは、できない相談ね。\\n\
        h1111210他人に見せるために書いたものではないもの。\\n\
        h1112205私の記憶は、私だけのもの。\\n\
        h1112204従者にも、あなたにも、見せるつもりはないわ。\\n\
        h1111310……それに、興味本位で読むには長すぎるの。\\n\
        忘れないうちにと書き始めたけれど、\\n\
        気づけば三百を超えてしまって。\\n\
        h1111206冊数がね。置き場所にも困っているわ。\
        "
      .to_string(),
      required_condition: None,
      callback: None,
    },
    DerivaliveTalk {
      parent_id: "服装へのこだわり".to_string(),
      id: "服装へのこだわり・昔から".to_string(),
      summary: "『つまり、その服装は昔から？』".to_string(),
      text: "\
        h1111205ええ、そうよ。\\n\
        h1111211けれど、あなたのファッションを見る限りでは\\n\
        それほど浮世離れしているわけではなさそうね。\
        "
      .to_string(),
      required_condition: None,
      callback: None,
    },
    DerivaliveTalk {
      parent_id: "服装へのこだわり".to_string(),
      id: "服装へのこだわり・違う服".to_string(),
      summary: "『たまには違う服も着てみない？』".to_string(),
      text: "\
        h1113205……そうね、たまにはいいかもしれないわ。\\n\
        h1111204あなた、選んでくれる？\\n\
        h1111210…だって、\\n\
        自分では良し悪しも好き嫌いもわからないもの。\\n\
        h1111206従者にクローゼットの中身を\\n\
        持って来させましょう。\\n\
        h1111204あなたのセンスをh1111211信じているわ。\
        "
      .to_string(),
      required_condition: None,
      callback: None,
    },
    DerivaliveTalk {
      parent_id: "生家の広さ".to_string(),
      id: "生家の広さ・思い出".to_string(),
      summary: "『思い出の品や場所はある？』".to_string(),
      text: "\
        h1111206ここへ来るまでの階段の下に、\\n\
        スペースがあったでしょう。\\n\
        h1111210あそこに隠れるのが好きでね。\\n\
        お気に入りの本やランプ、\\n\
        自作の地図に方位磁石なんかを持ち込んで、\\n\
        秘密基地を作っていたのよ。\\n\
        h1111205大きくなるにつれて縁遠くなったけれど、\\n\
        h1111210今でもあのわくわくする気持ちは思い出せるの。\
        "
      .to_string(),
      required_condition: None,
      callback: None,
    },
    DerivaliveTalk {
      parent_id: "生前の食事事情".to_string(),
      id: "生前の食事事情・好きな食べ物".to_string(),
      summary: "『何か好きな食べ物はなかった？』".to_string(),
      text: "\
        h1111205……そうね、硬い焼き菓子が好きよ。\\n\
        甘さが控えめのものが、特に。\\n\
        長持ちするし、口の中に味が残りにくいから\\n\
        読書の邪魔にならないの。\\n\
        ……従者からは不評だけれど。\\n\
        ポロポロこぼして回るから掃除が大変だ、ってね。\\n\
        \\1『こぼして回る……？』\\n\
        h1221210……ええ、そう。\\n\
        考え事をするとき、歩き回る癖があって……\\n\
        h1221206進んで汚したいわけではないのだけど、\\n\
        どうしてもやめられなくて。\
        "
      .to_string(),
      required_condition: None,
      callback: None,
    },
    DerivaliveTalk {
      parent_id: "身体が弱い".to_string(),
      id: "身体が弱い・お使い".to_string(),
      summary: "『かわりにお使いをしようか？』".to_string(),
      text: "\
        h1111101……h1111210やさしいのね。\\n\
        h1111210ありがたいけれど、結構よ。\\n\
        h1111206定期的な買い出しは既にしているし、\\n\
        h1111210私達が必要とするものはとても少ないの。\\n\
        h1111204あなたは客人で、従者ではないから。\\n\
        h1111210あなた自身のことだけを考えていてほしいの。\\n\
        \\1『私はあなたの役に立ちたいと思ってる』\\n\
        h1111101…………h1111204そう、わかったわ。\\n\
        ならば、そうね、h1111210次からは\\n\
        あなたにお茶菓子を用意してもらいましょう。\\n\
        h1111204私の好みはわかっているでしょう？\\n\
        h1111211お願いね、{user_name}。\
        "
      .to_string(),
      required_condition: None,
      callback: None,
    },
  ]
}

pub(crate) fn derivative_talks_per_talk_type() -> HashMap<TalkType, Vec<DerivaliveTalk>> {
  let all_talks = TalkType::all()
    .iter()
    .map(|t| random_talks(*t))
    .flat_map(|t| t.unwrap_or_default())
    .collect::<Vec<_>>();
  let mut talks: HashMap<TalkType, Vec<DerivaliveTalk>> = HashMap::new();
  for talk in derivative_talks() {
    let parent_talk = match all_talks.iter().find(|t| t.id == talk.parent_id) {
      Some(t) => t,
      None => {
        error!("Parent talk with id {} not found, skipping", talk.parent_id);
        continue;
      }
    };
    if let Some(tt) = parent_talk.talk_type {
      talks.entry(tt).or_default().push(talk);
    } else {
      error!(
        "Parent talk {} has no talk_type, skipping derivative",
        parent_talk.id
      );
    }
  }
  talks
}

pub(crate) fn derivative_talk_by_id(parent_id: &str) -> Option<Vec<DerivaliveTalk>> {
  derivative_talks()
    .into_iter()
    .filter(|t| {
      let condition_ok = match &t.required_condition {
        Some(condition) => condition(),
        None => true,
      };
      t.parent_id == parent_id && condition_ok
    })
    .collect::<Vec<_>>()
    .into()
}

pub(crate) fn get_parent_talk(derivative_talk: &DerivaliveTalk) -> Option<Talk> {
  let all_talks = TalkType::all()
    .iter()
    .map(|t| random_talks(*t))
    .flat_map(|t| t.unwrap_or_default())
    .collect::<Vec<_>>();
  let result = all_talks
    .into_iter()
    .find(|t| t.id == derivative_talk.parent_id);
  if result.is_none() {
    error!(
      "Parent talk with id {} not found",
      derivative_talk.parent_id
    );
  }
  result
}
