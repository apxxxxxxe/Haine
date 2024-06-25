use crate::events::common::*;
use crate::events::talk::{Talk, TalkType};
use crate::events::TalkingPlace;
use crate::variables::{get_global_vars, EventFlag, GlobalVariables};
use rand::prelude::*;

pub const TALK_ID_SERVANT_INTRO: &str = "従者について：イントロ";
pub const TALK_UNLOCK_COUNT_SERVANT: u64 = 5;
pub const TALK_ID_LORE_INTRO: &str = "死に対する興味：イントロ";
pub const TALK_UNLOCK_COUNT_LORE: u64 = 10;

// 私/主: 50代の身綺麗な男
// 僕/主様: 30代のおとなしい男
// わたし/主さま: 20代の活発な女
// ぼく/ご主人さま: 10代の男の子
pub const RANDOMTALK_COMMENTS: [&str; 18] = [
  "霧が濃い。",
  "彼女の声は低いがよく通る。",
  "彼女の赤い瞳の奥の思考は伺い知れない。",
  "「主に誉れあれ。」",
  "「主は客人をたいそうお気に入りのようだ。」",
  "「古木のように主は佇む。」",
  "「常に主様に心からの賛辞を。」",
  "「主様には僕達も知らない秘密が多い。」",
  "「主様の思考は大樹のように広がっている。」",
  "「主さまの美貌、わたしたちの誰もがうっとりしてるわ。」",
  "「主さまはわりと我儘よ。そんなところも好きだけど。」",
  "「主さまは勘違いされがちだけど優しいひとよ。」",
  "「ぼく、かけっこならご主人さまに勝てるよ。」",
  "「ご主人さま、たまに元気がないんだ。」",
  "「ご主人さまはいつも笑顔でいてほしいな。」",
  "「館近くのパン屋は絶品だった。」",
  "「街角の喫茶店は素晴らしいコーヒーを出していた。」",
  "「街の端にある花屋は色とりどりの花で溢れていた。」",
];

pub fn talk_with_punchline(text: String, funny_punchline: String) -> String {
  text + "\\n" + &funny_punchline
}

// esperantoで本のジャンルを記述
// それっぽければよし
static BOOK_TOPICS: [(&str, &str); 3] = [
  ("funkcia lingvo", "referenca travidebleco"), // 関数型言語, 参照透過性
  ("metafiziko", "ontologio"),                  // 形而上学, 存在論
  ("harmonio", "konsonanco"),                   // 調和, 一致
];

fn random_book_topic() -> (&'static str, &'static str) {
  *BOOK_TOPICS.choose(&mut rand::thread_rng()).unwrap()
}

struct RandomTalk {
  id: &'static str,
  text: String,
  required_condition: Option<fn(&mut GlobalVariables) -> bool>,
  callback: Option<fn()>,
}

pub fn random_talks(talk_type: TalkType) -> Vec<Talk> {
  let strings: Vec<RandomTalk> = match talk_type {
      TalkType::SelfIntroduce => vec![

        RandomTalk {
        id: TALK_ID_LORE_INTRO,
          text: format!("\
            h1111201死について。深く考えることはある？\\n\
            h1111206……あなたには聞くまでもないようね。\\n\
            h1111205私もそう。\\n\
            生きていたころから、なぜ生きるのか、死ぬとはどういうことかをずっと考えていたわ。\\n\
            いくつか不思議な話を知っているの。\\n\
            話の種に、いくつか語ってみましょうか。{}\
            ",
            if !get_global_vars().flags().check(&EventFlag::TalkTypeUnlock(TalkType::Lore)) {
              render_achievement_message(TalkType::Lore)
            } else {
              "".to_string()
            },
            ),
            // FIXME: 開放条件見直し
            required_condition: Some(|vars| vars.cumulative_talk_count() >= TALK_UNLOCK_COUNT_LORE),
            callback: Some(|| {
              get_global_vars().flags_mut().done(EventFlag::TalkTypeUnlock(TalkType::Lore));
            }),
        },

        RandomTalk {
          id: TALK_ID_SERVANT_INTRO,
          text: format!("\
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
            if !get_global_vars().flags().check(&EventFlag::TalkTypeUnlock(TalkType::Servant)) {
              let user_name = if let Some(name) = get_global_vars().user_name() {
                name.to_string()
              } else {
                "お客".to_string()
              };
              format!("\\![set,balloonnum,おや、本当だ。よろしくね、{}さん。]", user_name)
            } else {
              "".to_string()
            },
            if !get_global_vars().flags().check(&EventFlag::TalkTypeUnlock(TalkType::Servant)) {
              render_achievement_message(TalkType::Servant)
            } else {
              "".to_string()
            },
          ),
          // FIXME: 開放条件見直し
          required_condition: Some(|vars| vars.cumulative_talk_count() >= TALK_UNLOCK_COUNT_SERVANT),
          callback: Some(|| {
            get_global_vars().flags_mut().done(EventFlag::TalkTypeUnlock(TalkType::Servant));
          }),
        },

        RandomTalk {
          id: "科学への興味",
          text: "\
          h1111209生きていた頃、科学に興味を持っていたわ。\\n\
          h1111206物質の構造や、宇宙の謎、生命の起源。\\n\
          h1111205それは今でも変わらない。\\n\
          h1111305どんな新しい発見があるのか、いつも楽しみにしているのよ。\
          ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "服装へのこだわり",
          text: "\
          h1111203服装にはどちらかというと無頓着なの。\\n\
          h1112305一度決めた「いつもの」を守り続けるだけ。\\n\
          h1112304そうすれば、余計なことを考えなくて良くなるわ。\\n\
          h1111210そもそも私たちは着替える必要もないし、ね。\
          ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "生前の記録",
          text: "\
          h1111206生前のこと、記録に残しているの。\\n\
          ……h1123305まあ、まる1世紀も昔のことよ。\\n\
          自分のものだという実感はもうなくなってしまって、\\n\
          h1123310今読んでも他人の伝記を読んでいるようだわ。\\n\
          ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "外出できない理由",
          text: talk_with_punchline("\
            h1123310霊力の強い霊は、\\n\
            特定の場所に縛られる傾向にあるの。\\n\
            h1113205私もそう。結び付きが強すぎて、この家から離れられないのよ。\\n\
            ".to_string(),
            "\
            h1111203たまに、それができる幽霊もいるわ。\\n\
            街から街を渡り歩ける彼らの話は貴重よ。\\n\
            h1111306それができるのは彼らの自我の強さゆえ。\\n\
            h1111310こう言ってはなんだけど、偏屈な者が多いのよ。\
            ".to_string()),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "恋愛観",
          text: talk_with_punchline("\
            h1111205幽霊は生前の想い……好みや恨みに執着するの。\\n\
            h1111210想い人がいればその人に、恨みがあればその相手に。\\n\
            h1111203逆に、死後新たな執着が生まれることは\\n\
            ほとんどないわ。\\n\
            だから幽霊同士、h1111206ましてや人と幽霊の間に恋愛が生まれることは皆無といっていいでしょう。\\n\
            ".to_string(),
            "h1111304……なに、その顔は。h1111310あいにく、私は生きていた頃から恋愛とは無縁よ。\\n\
            ".to_string()),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "霊力の多寡",
          text: "\
            h1111204霊力の多寡は年月や才能、特別な契約の有無などで変わるけれど、\\n\
            最も大きな要因は環境──つまり、その地との関わりの深さによるの。\\n\
            h1111310私のように生家に根付いた霊はいわずもがな。\\n\
            h1111205……まあ、強いからといって良いことばかりでもないわ。\\n\
            h1111203霊にも社会があるの。h1111206上位者の義務というものもね。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "カンテルベリオという土壌",
          text: "\
            h1111203カンテルベリオには、霊……正確には、\\n\
            死の意識が集まりやすい土壌があるの。\\n\
            ……h1111210あなたがここに来たのも、\\n\
            偶然ではないのかもしれないわね。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "生家の広さ",
          text: "\
            h1111210ここは私の生家なの。実際は別荘なのだけど。\\n\
            h1111206知っての通り、従者がいなければ掃除が行き届かないほど広いの。\\n\
            h1111205……まあ、勝手知ったる場所なのは不幸中の幸い、といえなくもないかしらね。\\n\
            h1111210くつろいで暮らすのにこれ以上の場所はないわ。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

      ],
      TalkType::WithYou => vec![

        RandomTalk {
          id: "写真には写らない",
          text: "\
            h1111210今は手軽に写真が撮れていいわね。\\n\
            h1111205印象的な光景を、いつでも手元に残しておける。\\n\
            ……h1111201あら、私？h1121204光栄だけれど、残念ながら写真には写らないわ。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "霧の力",
          text: "\
            h1111206霧が、濃いでしょう。\\n\
            ただの霧ではないの。乾いた霧よ。\\n\
            むしろ、性質としては私たちに近い。\\n\
            h1111210ただの霊である私がここまで力を持っているのも、\\n\
            この地に根付いているもののおかげ。\\n\\n\
            h1111205次も、霧の濃い日にいらっしゃい。\\n\
            そのほうが身体が楽なの。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "見ていることしかできない",
          text: "\
            h1111210あなたたちが歩いている姿を、\\n\
            いつも窓から見ているの。\\n\
            h1111204いつも何かをして、どこかへ向かっている。\\n\
            h1111211羨ましいわ。\\n\
            h1111211私は\\_a[Fastened,どういうこと？]見ていることしかできない\\_aから、なおさら。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "あなたのゴスファッション",
          text: "\
            h1111201あなたのその趣味……\\n\
            ゴス・ファッションと言うんだったかしら。\\n\
            h1111202ほら、その首元の十字架……ああ、そのピアスも。\\n\
            h1111205そうでしょう？\\n\
            h1111211素敵ね。よく似合っているわ。\\n\
            h1111101……初めて言われた？h1111204そう。\\n\
            \\n\
            ……h1111206色眼鏡で見られたとして、気にする必要はないわ。\\n\
            自分に嘘をつかないことがいちばん大切だから。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "生前の食事事情",
          text: "\
            h1111204あなたは、ちゃんと食べているかしら？\\n\
            h1111210そう。いいことね。\\n\
            h1111104私？……h1111205生前は食が細かったわ。\\n\
            h1111210……というより、食そのものにあまり関心がなくてね。\\n\
            h1111205何かに没頭していると、食事をとる時間も惜しく思えてしまって。\\n\
            ……h1123310思えば、家政婦には随分と世話をかけたわね。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "スケッチ",
          text: "\
            h1111205……h1111201あら、絵を描いているの？見せて。\\n\
            h1111305あら、上手なのね。h1111202……これは、私？\\n\
            ……h1111205ふうん。こんなふうに見えているのね。\\n\\n\
            h1111101…………h1111204いいえ、いいのよ。\\n\
            h1111204たしかにそういう除霊の方法もあるけれど、\\n\
            私には効かないから心配はいらないわ。\\n\
            h1111205それに絵に描いてもらえるなんて、願ってもないことだもの。\\n\
            h1111210描きあげたら、また見せてね。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },
      ],
      TalkType::Lore => vec![

        RandomTalk {
          id: "冥界の渡し賃",
          text: "\
            h1111206古代ギリシャでは死者に銅貨を持たせて葬っていたの。\\n\
            h1111210冥界には川を渡っていかなければならなかったから、\\n\
            渡し賃を持たせて快適な旅を願う……ということね。\\n\\n\
            h1111205死者が川を越えていくという伝承は世界中で見られるわ。\\n\
            彼らにとって、境界線といえばまず川が連想されたのかしら。\\n\
            h1111210あなたなら、あの世とこの世の間にはなにがあると思う？\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "死体のうめき声",
          text: "\
            h1111205死体は、うめき声を上げることがあるのよ。\\n\
            h1111206……といっても、体内のガスが口から噴き出すとき、\\n\
            声帯が震えて音が出る……ただそれだけのことなのだけど。\\n\
            それでも、そんな些細なことが恐怖をかきたてて、\\n\
            人々は怪物を想像する。\\n\
            ……h1111201呆れるほどに多彩で、\\n\
            身近に根ざした感情の象徴だわ。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "屍蝋",
          text: "\
            h1111205屍蝋、って聞いたことあるかしら？\\n\
            h1111210死体の脂肪分が蝋状に変質した状態のこと。\\n\
            h1111206保存状態にもよるけれど、腐りもミイラ化もしない、\\n\
            生前の姿が比較的綺麗に残った状態と言われているわ。\\n\\n\
            h1111205珍しい現象だからかしらね。屍蝋化した死体は、\\n\
            地域によってさまざまな扱いを受けてきたわ。\\n\
            h1111210土に還らないことから、大地が拒んでいる……\\n\
            つまり悪霊になっているとして焼かれることもあれば、\\n\
            h1111204神が起こした奇跡として大切に扱われることもあるの。\\n\
            ……どちらにせよ、ふつうの葬送は望めなさそうね。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "死後の温かさ",
          text: "\
            h1111205死後数日経ったはずの身体が、まだ温かい。\\n\
            h1111210それは微生物が分解を行ったときに生じた熱のせいよ。\\n\
            ガスで膨張もするから、\\n\
            生前よりふくよかで健康的に見えることすらあったみたい。\\n\
            ……h1111204死体が蘇って夜な夜な彷徨い歩く、\\n\
            あるいは夢枕に立って生命を吸い取るという民話は、\\n\
            そんな様子に理由をつけるためのものじゃないかしら。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "生長する死体",
          text: "\
            h1111205掘り起こした死体の髪や爪が伸びていた！\\n\
            h1111210土葬が一般的だった時代、たびたびあった話。\\n\
            乾燥して縮むから、皮膚の下の髪や爪が露出する。\\n\
            それがまるで生長しているように見えたの。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "土葬の空洞",
          text: "\
            h1111206土葬の場合、地中の遺体が朽ちるとそこに空洞ができるわ。\\n\
            h1111205「死体に足を引っ張られる」という伝承は、\\n\
            これを踏み抜いてしまっただけかもしれないわね。\\n\
            h1111210あなたも墓地を歩くときは気をつけて……って、\\n\
            h1111204あなたの住む場所にそんなところは少ないかしら。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "永遠の夢",
          text: "\
            h1113105恒久の平和、不死の身体、永劫の繁栄……。\\n\
            h1113204永遠を夢見た人物の多くは失敗していて、その代償を払っている。\\n\
            h1113210寓話のモチーフとしての話よ。\\n\
            ……h1113106求めるのは、ほんとうに間違ったことなのかしら？\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "生体電気",
          text: "\
            h1111206カエルの足に電流を流す実験。\\n\
            生体電気の発見に繋がったといわれる\\n\
            あの現象は、\\_a[Misemono,どんな見世物だったの？]死者の蘇りを謳う見世物\\_aに\\n\
            利用されたことがあったの。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "死者の埋葬",
          text: "\
            h1111206古代ギリシャでは、刑死の際は毒薬に阿片を混ぜたものを飲ませていたの。\\n\
            h1113210それは死の苦しみを和らげるためだったのかもしれないけれど、\\n\
            それ以上に、死を恐れる人々を抑えるためだったのかもしれないわね。\\n\
            h1113205罰ではあれど、必要以上に苦しませることはない、と。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "死後の存在",
          text: "\
            h1111210幽霊、霊体、死後の存在。人類の科学は、そういったものにまだ答えを出していない。\\n\
            h1111205存在する、しないの議論は、h1112205まあ、私たちには必要ないわね。\\n\
            h1111210……いつかその時が来るのかしら。霊体を観測し、干渉し……あるいは、消滅させる方法。\\n\
            h1111205ふふ。私、期待しているの。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "黒死病",
          text: "\
            h1111210黒死病が蔓延していたとき、問題になっていたのがいわゆる「早すぎた埋葬」。\\n\
            h1111205ある技師は生き埋めにされる恐怖から逃れるため、\\n\
            埋葬者が生きていることを棺の内側から知らせる仕組みがついた棺を開発したの。\\n\
            h1111204彼、デモンストレーションのために自ら生き埋めになってみせたそうよ。\\n\
            h1212210自分で出られない状態で、冷たい土の下へ。\\n\
            ……h1211506どんな心地がしたのかしらね。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

      ],
      TalkType::Servant => vec![
        RandomTalk {
          id: "霊力と可視性",
          text: talk_with_punchline("\
            h1111206\\1ポットがひとりでに浮き、空になっていたカップに飲み物が注がれる。\\n\
            \\0……h1111204私が見えて彼らが見えないのは、霊としての力量の違いよ。\\n\
            h1111206強い霊力があればあなたのような人間の目にも見えるし、\\n\
            物理的な接触も可能になるの。\\n\
            ".to_string(),
            "h1111206……つまり、彼らのように霊力が弱ければ、\\n\
            誰かさんにべたべたと触られることもなかったということね。\
            ".to_string()),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "低級霊との契約",
          text: talk_with_punchline("\
            h1111206\\1ポットがひとりでに浮き、空になっていたカップに飲み物が注がれる。\\n\
            h1111206私の元へ集うのは弱い人たち。\\n\
            自分だけでは溶けゆく自我を押し留められず、さりとてそれを受け入れることもできない霊。\\n\
            h1111210役割を与えてあげるの。一種の契約ね。\\n\
            h1111205使命に縛られはするけれど、消滅するよりはよしと彼らは決断したの。\
            ".to_string(),
            "\
            ".to_string()),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "幽霊たちの役割",
          text: "\
            h1111203従者……と、私が呼ぶ幽霊たち。\\n\
            h1111210私の与えた役割を全うしてくれるものは多くいるわ。\\n\
            h1111205最も多いのは、自分の生前の経験を記録として私に提供してくれる者たち。\\n\
            h1111210紙に念写できる程度の力を分け与えているの。\\n\
            h1111206彼らの記録に、一つとして同じものはない。\\n\
            h1111210読んでいて退屈しないわ。\\n\
            ……h1113204そういえば、少し形は違えど、あなたもその一人ね。\\n\
            h1113211期待しているわ、{user_name}。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "幽霊たちの自由",
          text: talk_with_punchline("\
            h1111206私は彼らと直接話すことはできないの。\\n\
            霊力の差があまりにも大きい場合、\\n\
            h1111210会話や接触を少し行うだけで、弱い方の霊は力を奪われる。\\n\
            ".to_string(),
            "\
            h1111701……h1111204いえ、私はやったことがなくて、伝聞なのだけど。\\n\
            h1121206……他人の魂を玩具になんてしないわよ。\\n\
            h1121301勘違いしているようだけど、私にそんな嗜好はないわ。\
            ".to_string()),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "あなたの価値",
          text: "\
            h1111101何をすればいいかって？\\n\
            h1111204今しているように、ただ話し相手になればいいのよ。\\n\
            h1111206私には従者がいるけれど、\\n\
            彼らは私と話すようにはできていないから。\\n\
            h1111204あなたの価値は、その自由意志。\\n\
            h1111210ここは予想通りなものばかりで退屈なの。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

      ],
      TalkType::Past => vec![

        RandomTalk {
          id: "人ひとり",
          text: "\
            h1111210人ひとり、殺せるとしたら誰にする？という他愛ない問い。\\n\
            h1111305だから私は私を殺したの。\\n\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "死体損壊",
          text: "\
            h1111210「死体の損壊は死者への冒涜だ」\\n\
            という言説があるわね。\\n\
            h1111205当事者の視点から言うと、別にそうでもなかったわ。\\n\
            h1111310幽霊が元の身体に戻った例もない。\\n\
            h1111306畢竟、それは生者の問題ということね。\\n\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "惨めな人生",
          text: "\
            h1111205みじめな人生の上に正気でいるには、\\n日々は長すぎたの。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "行き場のない苦しみ",
          text: "\
            h1112202誰が悪い？いいえ、誰も悪くない。\\n\
            打ち明けたところで、的はずれな罪悪感を生むだけ。\\n\
            h1112205だからといって、他人に責をなすりつけるほど鈍くあることもできなかった。\\n\
            h1112210この気持ちには、どこにも行き場がなかったの。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "死の瞬間",
          text: "\
            h1111205死ぬ瞬間、後悔はなかった。\\n\\n\
            もう一度同じ人生を生きることができたとしても、私は同じことをすると断言できるわ。\\n\
            ……h1121210ただ、遺書くらいは書いたほうがよかったかしら。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "助けは遂げられず",
          text: "\
            h1111205助けようとしてくれた人は沢山いたけれど、\\n\
            h1121210それが遂げられることはついぞなかったわ。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "死なない理由",
          text: "\
            h1111210生きていて良かったと思えることは数えきれないほどあったわ。\\n\
            h1111205でも、死なない理由は一つも見つからなかった。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "ふつうになりたかった",
          text: "\
            h1122210ふつうになりたかった。\\n\
            ……h1122205でも、ふつうだったら、もう私じゃないとも思う。\\n\
            それは私の顔をした別のだれかで、\\n\
            私は私の性質と不可分で、\\n\
            今ここにいる私は、私以外でいられない。\\n\
            h1122210だから、私として生きることができなかった私は、もうどこにもいられない。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "人と本",
          text: "\
            h1111205昔から、人と本の違いがわからなかったの。\\n\
            h1121204もちろん、区別がつかないという意味ではなくて。\\n\
            ……h1111210人に期待するものがそれだけしか無かったの。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "分厚い本",
          text: {
            let topic = random_book_topic();
            format!("\
            h1111204……手持ち無沙汰のようね。\\n\
            h1111206なにか本を見繕ってあげましょうか。\\n\
            h1111203……h1111201これはどうかしら。\\n\
            \\1……ずいぶん分厚い本を手渡された。\\n\
            h1111202{}の構成要素について論じられているの。\\n\
            {}についての項が特に興味深いわ。\
            h1111205要点だけなら半日もあれば読み終わると思うから、\\n\
            h1111204終わったら意見を交換しましょう。\
            ", topic.0, topic.1)
          },
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "今度こそ無へ",
          text: "\
            h1111205死にぞこなったものだから、\\n\
            次の手段を求めている。\\n\
            ……h1112305今度こそ、終わらせたいの。\\n\
            今度こそ、無へ。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "魂は消える",
          text: "\
            h1111110未練もなく、しかし現世に留まっている魂。\\n\
            h1111105あるべきでないものはやがて消滅する。\\n\
            h1111206多少の不純物が含まれようと、そのルールは変わらない。\\n\
            h1111205私は、それを待ち望んでいるの。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

      ],
      TalkType::Abstract => vec![

        RandomTalk {
          id: "ハイネの死",
          text: {
            let vars = get_global_vars();
            let achieved_talk_types = [TalkType::Past];
            achieved_talk_types.iter().for_each(|t| {
              vars.flags_mut().done(EventFlag::TalkTypeUnlock(*t));
            });
            let achievements_messages = achieved_talk_types
              .iter()
              .map(|t| render_achievement_message(*t))
              .collect::<Vec<_>>();
            format!("\
              h1111210…………幽霊にとって、自身の死の記憶はある種のタブーなの。\\n\
              誰もが持つがゆえの共通認識。自身の死は恥部なのよ。\\n\
              私も、彼らのそれには深く踏み込まない。\\n\
              けれど、あなたは生者だから。\\n\
              \\n\
              ……私の死因は、自殺よ。\\n\
              この家で死に、そしてここに縛り付けられたの。\\n\
              {}", 
              achievements_messages.join("\\n")
            )
          },
          required_condition: Some(|vars| vars.total_boot_count() >= 3),
          callback: None,
        },

        RandomTalk {
          id: "今ここに立っていること",
          text: "\
            h1111310過去は記憶の中にしかない。\\n\
            h1111305未来は想像の中にしかない。\\n\
            h1112305我々が立っているのは今ここだけ。\\n\
            私たちが感じられるのは現在だけ。\\n\
            h1112310ひどい過去も、おぞましい未来も、\\n\
            h1112305いま私が立つこの瞬間には存在しないの。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "自己理解、他者理解",
          text: "\
            h1111205自分のことを本当に理解しているのは他人、って本当なのかしら。\\n\
            h1111206……私が知らない私がいる。\\n\
            h1112204なんだか不安になってきたわ。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "感動と倦み",
          text: "\
            h1111205ある本を最初に読んだときの感動と、何度も読み返して全て見知ったゆえの倦み。\\n\
            どちらがその本の真の印象かしら。\\n\\n\
            h1111210私はどちらも正しいと思うの。\\n\
            ……h1111504卑怯だと思った？\\n\
            h1111210印象なんてその時々で変わるもので、h1111205一つに定まることなんて稀だもの。\\n\\n\
            まして、自分の中に秘めるものならなおさら。\\n\
            h1111506どちらか一方だけだなんて、勿体ないわ。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "納得のための因果",
          text: "\
            h1111210因果が巡ってきた。\\n\
            過去が現在を刈り取りに来た。\\n\
            私は報いを受けたのだ。\\n\\n\
            ……h1111205それが、自分を納得させるための妄想だったとしたら？\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "怖いものを見るということ",
          text: "\
            h1111201怖いものだからこそ、見つめなければ戦えない。\\n\
            ……h1121205そんなもの、戦える人のためだけの論理だわ。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "停滞を終わらせるために",
          text: "\
            h1111205危険と隣り合わせだからこそ、世界は美しいの。\\n\
            身を損なう心配がなくなっては、美しさが心を打つこともない。\\n\
            h1121205ただただ平坦な、揺らがぬ水面があるだけ。\\n\
            h1121210それはやがて、淀み、腐る。\\n\
            h1111205願わくば、せめて終わりがありますように、と。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "停滞の破壊",
          text: "\
            h1111105人生に変化は付きもの……けれどh1111110停滞はそれ以上。\\n\
            一度立ち止まってしまうと、空気は一瞬で淀んで、身動きがとれなくなってしまう。\\n\
            それは倦怠とも違う、鈍い痛み。\\n\
            h1111201あなた、h1111205もしそうなったときは、多少無理にでも変化を取り入れるほうがいいわ。\\n\
            ……h1111210たとえなにかを破壊することになるとしても、何も出来ないよりはずっとましよ。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "極限の変化としての死",
          text: "\
            h1111205死の瞬間の、極限に振れた変化。\\n\
            命が命でなくなり、身体が陳腐な肉の塊になる、その一瞬が愛しくてたまらない。\\n\
            どうしようもなく、愛しいの。\\n\\n\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "死の向こう側",
          text: "\
            h1112110どうか、死の向こう側がありませんように。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "沈んでいく",
          text: "\
            h1111105沈んでいく。\\n\
            手がどうしても動かなくて、目の前の希望を掴めない。\\n\
            身体が重い。浅い呼吸のなかで、沈んでいく自分の身体を感じていることしかできない。\\n\
            私は、私を救うことを諦めているみたい。\\n\
            h1111110どうして。\\n\
            h1121205どうして、こうなってしまったのだろう。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "人を解体したい",
          text: "\
            h1111110人を解体したいと、思うことがあるの。\\n\
            何が人を人たらしめているのか、どこまで分解すれば人は人でなくなるのか。\\n\
            h1111105人という恐ろしく不可解な物の、どこにその根源があるのか。\\n\
            それを知るには、他に方法が思いつかないの。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "わがままな祈り",
          text: "\
            h1111210がんばっているってこと、\\n\
            理解できなくても見ていてほしかったの。\\n\
            ……h1121205わがままかしら。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "生者にとっての慰め",
          text: "\
            h1111210枯れ木に水をあげましょう。\\n\
            もはや花は見れずとも、それが慰めとなるのなら。\\n\
            \\n\
            h1111205それは誰にとって？\\n\
            h1111206もちろん、死を悼む者にとっての慰めよ。\\n\
            むくろに心はないもの。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "不可逆な崩壊",
          text: "\
            h1111210燃え殻がひとりでに崩れるように、心が静かに割れて戻らなくなった。\\n\
            h1111205だから、諦めたの。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "中途半端な助け",
          text: "\
            h1111210中途半端な助けは何もしないより残酷だわ。\\n\
            h1111205希望を持たせておいて、それを奪うのだもの。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "レンズの歪み",
          text: "\
            h1111205観察と模倣を続ければ、完全に近づけると思っていた。\\n\
            想定外だったのは、レンズが歪んでいたことと、それを取り替える方法がなかったこと。\\n\
            h1121310そうなればすべて台無し。h1121304諦めるしかなかったわ。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "先の見えない苦しみ",
          text: "\
            h1111205一寸先は暗く、扉は閉ざされている。\\n\
            不明な道のりを諸手で探るよりも、\\n\
            h1112305目先の手首を切り裂くほうが遥かに明瞭なのだ！\\n\
            ……h1111210なんてね。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "唯一の視点",
          text: "\
            h1111206私たちは、自我という色眼鏡を通してしか世界を観測できない。\\n\
            h1111204あなたは目の前にいるのに、\\n\
            あなたが見る世界を私が知ることはできないの。\\n\
            h1112210それって、この上なく残酷なことだわ。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "一つの個としての限界",
          text: "\
            h1111203世界が複雑で曖昧すぎるから、\\n\
            私たちは認識したものを理解できる形に歪めてしまう。\\n\
            h1111210既存の分類に当て嵌めて、安心を優先するの。\\n\
            曇る視界と引き換えにね。\\n\
            ……h1111204あなたには、私はどう見えているのかしら？\\n\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "自己同一性の仮定",
          text: "\
            h1111205環境と経験の総体こそが、\\n\
            自己であるような気がするの。\\n\
            自己同一性すら偶然の産物？\\n\
            h1111210執着しているのが馬鹿馬鹿しく思えてくるわ。\\n\
            h1111205仮にそうでなければ。\\n\
            ……自己は最初から決定されている？\\n\
            h1111210それこそ、ね。\\n\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "自分の理解者は自分だけ",
          text: "\
            h1111210「なぜみんな私をわかってくれないの？」と誰もが思う。\\n\
            h1111205答えは簡単。他人があなたではなく、あなたが他人でないからよ。\\n\
            畢竟、あなた以外にあなたを理解できるひとはいないの。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "得ることは失うこと",
          text: "\
            h1111210あまねく変化は表裏一体。\\n\
            h1111206何かを得るとき、選択は慎重になさい。\\n\
            h1111205それは失うものをも左右するのだから。\\n\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

      ],
    };

  let vars = get_global_vars();
  let mut talks = Vec::new();
  for s in strings {
    if let Some(expr) = s.required_condition {
      if !expr(vars) {
        continue;
      }
    }
    talks.push(Talk::new(
      Some(talk_type),
      s.id,
      s.text.to_string(),
      s.callback,
    ));
  }
  talks
}

#[allow(dead_code)]
static BOTSU: Lazy<Vec<(&str, String)>> = Lazy::new(|| {
  vec![

    // RandomTalk {
    //   id: "月の満ち欠けのように",
    //   text: "\
    //     h1111205月の満ち欠けのように、私の心は移り変わる。\\n\
    //     h1111210理解を得ることは難しかったわ。\\n\
    //     そんな仕打ちも、納得はできる。自分ですら不可解なのだから。\\n\
    //     ……h1121306少しでも自分で律することができたなら、\\n\
    //     こんなに苦しむことはなかっただろうに。\
    //     ".to_string(),
    //   required_flags: None,
    //   callback: None,
    // },

    ("茶菓子は用意してね", "\
    h1111210茶葉や茶菓子はあなたが持ってきてね。\\n\
    h1111206ここにある分は限られているし、私一人では補充する手段がないから。\\n\
    h1111210あなたが食べられるようなものは、私には用意できないの。\
    ".to_string()),

    ("不安を書き出す", "\
    h1111204……なにか不安なの？\\n\
    それとも、何が不安かもわからない？\\n\
    h1111210……紙に書き出してみるのはどうかしら。\\n\
    h1111205いくらか整理がつくかもしれない。\
    ".to_string()),

    ("", format!("\
    h1111201ある程度外見を変えることもできるの。h1111211……こんなふうに。\\n\\n\
    h1000000{}\\n\
    \\0髪も、身長も、年齢すら、私たちには関係ないの。\\n\
    h1111210\\1瞬きしたとき、彼女はまた元の姿に戻っていた。\
    h1111204あまり大きく変化すると自己認識が揺らいでしまうから、基本的には最も自分らしい姿をすることが多いわ。\\n\
    h1111211こういう戯れもたまにはいいでしょう？\
    ",
    user_talk("……髪が伸びてる！","彼女の姿が揺らいだかと思うと、その姿は一瞬で変わった。", true))),

    ("", "\
    h1111205食事の時間になっても部屋から出てこない家族。\\n\
    扉を開けてみると、彼女は足の一部を残して焼死していた。\\n\
    ……h1111206人体発火現象は、世界中で見られるわ。\\n\
    h1111210多くの場合火気はなく、発火の原理は不明。\\n\
    h1111206さらに、いくらかの延焼はあれど周囲には被害が及ばない。\\n\
    h1111210まったく不思議な話よね。h1111204あなたはどう考える？\
    ".to_string()),

    ("", "\
    h1111210寄る辺ない幽霊はいつか消える。\
    それが10年後なのか、100年後なのか、それとも明日なのか。\\n\
    それは分からないけれど、その日は必ず来る。\\n\
    h1111205だからあなた、いつか消える幽霊にかまけるなんて、時間の無駄なのよ。\\n\
    ……h1111310いつ来なくなっても、私は気にしないわ。\
    ".to_string()),

    ("", "\
    h1111204あなた、口数が少ないのね。\\n\
    h1111201いえ、いいのよ。h1111205そう、どちらでもいい。\\n\
    ".to_string()),

    ("", "\
    h1111110\\1床を、極彩色の虫が這っている。\\n\
    h1111106……h1111105。\\n\\n\
    \\1ハイネはそれを一瞥して、すぐに視線を戻した。\\n\
    気にならないのだろうか。\\n\\n\
    ……そういえば、本で読んだことがある。\\n\
    フィクションの怪物の多くが不潔な外見をしているように、\\n\
    人ならざる者たちは毒虫や汚物に対する嫌悪感をほとんど持たないらしい。\\n\
    あれは小説の設定だったが……彼女もそうなのだろうか。\\n\\n\
    h1111110……h1111201あなた。\\n\
    h1111204虫を捕まえるのが得意だったりしないかしら。\\n\
    \\1……。\
    ".to_string()),

    ("", "\
    h1111205見慣れたはずの場所にいながら、いつもと違う道に迷いこんだことはある？\\n\
    \\n\
    h1111204もしそうなったら、「\\_a[Yomotsuhegui,ヨモツヘグイって？]ヨモツヘグイ\\_a」……\
    つまり、食べ物には常に注意しなさい。\\n\
    h1111205一度だけなら、は許されない。それがすべてを変えてしまうような落とし穴がこの世にはあるの。\
    ".to_string()),

    ("", format!("\
    h1111205悲しい。ここに縛られていることが、ではない。私が見ることのできない世界のこと。\\n\
    ……h1111210何も、知ることができないの。老人の、幼子の、男の、女の、見る世界を、\\n\
    そのすべてを私がこの身で知ることはかなわない。h1111205決して、……h1111210決して。\\n\
    それが、悲しくて、悔しくて、気が狂いそうになる。\\n\
    h1122305だって、せっかくこの世に生まれたのに。こんなにも自由なのに。……この手で、何をすることもできるはずなのに！\\n\\n\\_w[1200]\
    {}\
    \\0……h1111205あなたに言っても詮無いことだわ。忘れてちょうだい。\
    ",
    user_talk("ハイネ……。","思わず、声が漏れる。", false))),

  ]
});

pub fn finishing_aroused_talks() -> Vec<String> {
  let vars = get_global_vars();
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
  all_combo(&talk_parts)
}

pub fn changing_place_talks(
  previous_talking_place: &TalkingPlace,
  current_talking_place: &TalkingPlace,
) -> Vec<String> {
  let vars = get_global_vars();
  let parts: Vec<Vec<String>> = if !vars.flags().check(&EventFlag::FirstPlaceChange) {
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
          let achieved_talk_types = [TalkType::Abstract];
          achieved_talk_types.iter().for_each(|t| {
            vars.flags_mut().done(EventFlag::TalkTypeUnlock(*t));
          });
          let achievements_messages = achieved_talk_types
            .iter()
            .map(|t| render_achievement_message(*t))
            .collect::<Vec<_>>();
          vec![format!(
            "h1111204あなた、書斎は初めてね。\\n\
            \\1……客間より少し狭い程度の間取りに、所狭しと本棚が設置されている。\\n\
            窓すら本棚に覆われていて、ハイネは蝋燭の灯りで本を読んでいるようだった。\\n\
            h1111210ここは私の私室でもあるの。\\n\
            h1111204……あなたは、本を読むのは好き？\\n\
            h1111306私は好きよ。巨人の肩に乗って遠くが見える。\\n\
            h1111305あるいは、ここではないどこかへ、遠くへ行ける。\
            h1111204あなたも自由に読み、そして考えなさい。\\n\
            h1111310ここはそういう場所よ。{}\
            ",
            achievements_messages.join("\\n")
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
}

pub const IMMERSION_INTRODUCTION_TALK: &str = "\
  \\1……気付けば、辺りが暗くなっていた。\\n\
  そんなに長く話していただろうか。腕時計を見ると、まだ昼間だった。\\n\
  \\0少し話し込んでいたようね。\\n\
  \\1ハイネは呟き、しばらく手をつけていなかった\\0\\![bind,ex,没入度用,0]\\1カップを傾けた。\\n\
  h1111210\\1……？あたりが再び明るくなっている。\\n\
  h1111205明かりは、私の霊力で灯しているの。\\n\
  特別な灯。そちらに注意を払えなくなると、すぐに消えてしまうのよ。\\n\
  h1121210我ながら不便だけれど、従者に頼むにも難しい仕事でね。\\n\
  h1121204悪いけれど、そういうものだと思ってちょうだい。\\n\
  ";
