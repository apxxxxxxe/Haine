use crate::events::common::*;
use crate::get_touch_info;
use crate::roulette::RouletteCell;
use crate::variables::{get_global_vars, EventFlag, IDLE_THRESHOLD};
use core::fmt::{Display, Formatter};
use once_cell::sync::Lazy;
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use shiorust::message::{parts::HeaderName, Request, Response};
use std::collections::HashSet;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

// トーク1回あたりに上昇する没入度
const IMMERSIVE_RATE: u32 = 8;

#[derive(Clone)]
pub struct Talk {
  pub talk_type: Option<TalkType>,
  pub text: String,
  pub id: &'static str,
}

impl RouletteCell for Talk {
  fn key(&self) -> String {
    self.id.to_string()
  }
}

impl Display for Talk {
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    write!(f, "{}", self.text)
  }
}

fn talk_with_punchline(text: String, funny_punchline: String) -> String {
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

pub fn random_talks(talk_type: TalkType) -> Vec<Talk> {
  let strings: Vec<(&str, String, Vec<EventFlag>)> = match talk_type {
      TalkType::SelfIntroduce => vec![

        ("服装へのこだわり", "\
        h1111203服装にはどちらかというと無頓着なの。\\n\
        h1112305一度決めた「いつもの」を守り続けるのが楽で、いつもそうしているわ。\
        ".to_string(), vec![]),

        ("生前の記録", "\
        h1111206生前のこと、記録に残しているの。h1000000……h1111205このノートに。\\n\
        ……h1123305まあ、ずいぶん昔のことよ。\\n\
        自分のものだという実感はもうなくなってしまって、\\n\
        h1123310今読んでも他人の伝記を読んでいるようだわ。\\n\
        ".to_string(), vec![]),

        ("外出できない理由", talk_with_punchline("\
        h1123310霊力の強い霊は、\\n\
        特定の場所に縛られる傾向にあるの。\\n\
        h1113205私もそう。結び付きが強すぎて、この家から離れられないのよ。\\n\
        ".to_string(),
        "\
        h1123210……まあ、外に出たとしても問題は山積みでしょうけど。\\n\
        なにせ、今のファッションについていける自信がないわ。\\n\
        ……h1123204ジーンズはまだ流行ってる？\
        ".to_string()), vec![]),

        ("恋愛観", talk_with_punchline("\
        h1111205幽霊は生前の想い……好みや恨みに執着するの。\\n\
        h1111210想い人がいればその人に、恨みがあればその相手に。\\n\
        h1111203逆に、死後新たな執着が生まれることはほとんどないわ。\\n\
        だから幽霊同士、h1111206ましてや人と幽霊の間に恋愛が生まれることは皆無といっていいでしょう。\\n\
        ".to_string(),
        "h1111304……なに、その顔は。h1111310あいにく、私は生きていた頃から恋愛とは無縁よ。\\n\
        ".to_string()), vec![]),

        ("霊力と可視性", talk_with_punchline("\
        h1111206\\1ポットがひとりでに浮き、空になっていたカップに飲み物が注がれる。\\n\
        \\0……h1111204私が見えて彼らが見えないのは、霊としての力量の違いよ。\\n\
        h1111206強い霊力があればあなたのような人間の目にも見えるし、\\n\
        物理的な接触も可能になるの。\\n\
        ".to_string(),
        "h1111206……つまり、彼らのように霊力が弱ければ、\\n\
        誰かさんにべたべたと触られることもなかったということね。\
        ".to_string()), vec![]),

        ("霊力の多寡", talk_with_punchline("\
        h1111204霊力の多寡は年月や才能、特別な契約の有無などで変わるけれど、\\n\
        最も大きな要因は環境──つまり、その地との関わりの深さによるの。\\n\
        h1111310私のように生家に根付いた霊はいわずもがな。\\n\
        h1111205……まあ、強いからといって良いことばかりでもないの。\\n\
        ".to_string(),
        "\
        h1111203霊にも社会があってね。\\n\
        h1111515ノブレス・オブリージュというわけ。\
        ".to_string()), vec![]),

        ("低級霊との契約", talk_with_punchline("\
        h1111206\\1ポットがひとりでに浮き、空になっていたカップに飲み物が注がれる。\\n\
        h1111206私の元へ集うのは弱い人たち。\\n\
        自分だけでは溶けゆく自我を押し留められず、さりとてそれを受け入れることもできない霊。\\n\
        h1111210役割を与えてあげるの。一種の契約ね。\\n\
        h1111205使命に縛られはするけれど、消滅するよりはよしと彼らは決断したの。\
        ".to_string(),
        "\
        ".to_string()), vec![]),

        ("カンテルベリオという土壌", "\
        h1111203カンテルベリオには、霊……正確には、\\n\
        死の意識が集まりやすい土壌があるの。\\n\
        ……h1111210あなたがここに来たのも、\\n\
        偶然ではないのかもしれないわね。\
        ".to_string(), vec![]),

        ("生家の広さ", talk_with_punchline("\
        h1111210ここは私の生家なの。実際は別荘なのだけど。\\n\
        h1111206知っての通り、従者がいなければ掃除が行き届かないほど広いの。\\n\
        ".to_string(),
        "h1111205……まあ、勝手知ったる場所なのは不幸中の幸い、といえなくもないかしらね。\
        ".to_string()), vec![]),

        ("幽霊たちの役割", "\
        h1111203従者……と、私が呼ぶ幽霊たち。\\n\
        h1111210私の与えた役割を全うしてくれるものは多くいるわ。\\n\
        h1111205最も多いのは、自分の生前の経験を記録として私に提供してくれる者たち。\\n\
        h1111210一つとして同じものはない。読んでいて退屈しないわ。\\n\
        ……h1113204少し形は違えど、あなたもその一人ね。\\n\
        h1113211期待しているわ、{user_name}。\
        ".to_string(), vec![]),

        ("幽霊たちの自由", talk_with_punchline("\
        h1111206私は彼らと直接話すことはできないの。\\n\
        霊力の差があまりにも大きい場合、\\n\
        h1111210会話や接触を少し行うだけで、弱い方の霊は力を奪われる。\\n\
        ".to_string(),
        "\
        h1111701……h1111204いえ、私はやったことがなくて、伝聞なのだけど。\\n\
        h1121206……他人の魂を玩具になんてしないわよ。\\n\
        h1121301勘違いしているようだけど、私にそんな嗜好はないわ。\
        ".to_string()), vec![]),

      ],
      TalkType::WithYou => vec![

        ("紙の薄さ", "\
        h1111204\\1メモ帳を取り出す。\\n\
        ハイネの語る話は情報量が多いものばかりだったから、\\n\
        何かに書き留めて整理したかったのだ。\\n\
        h1141201……あら、それが現代のノート？\\n\
        \\1思いがけず、ハイネが興味を示した。\\n\
        メモ帳を差し出す。\\n\
        h1113205すごい。小さいし、すごく薄いわ。\\n\
        罫線も正確だし、紙の質もいい。\\n\
        意匠も美しいわ。印刷はここまでできるようになったのね。\\n\
        \\n\
        ……ええ、もういいわ。ありがとう。\\n\
        \\1ひとしきり眺め、なで回した後、ハイネはメモ帳を返してくれた。\
        h1113204技術の進歩ね。\\n\
        h1113205……十年ひと昔、このままでは置いていかれてしまいそうね。\
        ".to_string(), vec![]),

        ("写真には写らない", "\
        h1111210今は手軽に写真が撮れていいわね。\\n\
        h1111205印象的な光景を、いつでも手元に残しておける。\\n\
        ……h1111201あら、私？h1121204光栄だけれど、残念ながら写真には写らないわ。\
        ".to_string(), vec![]),

        ("霧の力", "\
        h1111206霧が、濃いでしょう。\\n\
        ただの霧ではないの。乾いた霧よ。\\n\
        むしろ、性質としては私たちに近い。\\n\
        h1111210ただの霊である私がここまで力を持っているのも、\\n\
        この地に根付いているもののおかげ。\\n\\n\
        h1111205次も、霧の濃い日にいらっしゃい。\\n\
        そのほうが身体が楽なの。\
        ".to_string(), vec![]),

        ("見ていることしかできない", "\
        h1111210あなたたちが歩いている姿を、\\n\
        いつも窓から見ているの。\\n\
        h1111204いつも何かをして、どこかへ向かっている。\\n\
        h1111211羨ましいわ。\\n\
        h1111211私は\\_a[Fastened,どういうこと？]見ていることしかできない\\_aから、なおさら。\
        ".to_string(), vec![]),

        ("あなたのゴスファッション", "\
        h1111201あなたのその趣味……\\n\
        ゴス・ファッションと言うんだったかしら。\\n\
        h1111202ほら、その首元の十字架……ああ、そのピアスも。\\n\
        h1111205そうでしょう？\\n\
        h1111211素敵ね。よく似合っているわ。\\n\
        h1111101……初めて言われた？h1111204そう。\\n\
        \\n\
        ……h1111206色眼鏡で見られたとして、気にする必要はないわ。\\n\
        自分に嘘をつかないことがいちばん大切だから。\
        ".to_string(), vec![]),

        ("あなたの価値", "\
        h1111101何をすればいいかって？\\n\
        h1111204今しているように、ただ話し相手になればいいのよ。\\n\
        h1111206私には従者がいるけれど、\\n\
        彼らは私と話すようにはできていないから。\\n\
        h1111204あなたの価値は、その自由意志。\\n\
        h1111210ここは予想通りなものばかりで退屈なの。\
        ".to_string(), vec![]),

        ("生前の食事事情", "\
        h1111204あなたは、ちゃんと食べているかしら？\\n\
        h1111210そう。いいことね。\\n\
        h1111104私？……h1111205生前は食が細かったわ。\\n\
        h1111210……というより、食そのものにあまり関心がなくてね。\\n\
        h1111205何かに没頭していると、食事をとる時間も惜しく思えてしまって。\\n\
        ……h1123310思えば、家政婦には随分と世話をかけたわね。\
        ".to_string(), vec![]),

        ("スケッチ", "\
        h1111205\\1本を読む彼女をスケッチしている。\\n\\n\
        彼女は写真に写らないという。\\n\
        それを聞いてから、彼女の姿を何かに残しておきたくなって描きはじめたのだ。\\n\
        \\0……h1111201あら、絵を描いているの？見せて。\\n\
        h1111202……これは、私？……h1111205ふうん。こんなふうに見えているのね。\\n\\n\
        h1111101…………h1111204いいえ、いいのよ。\\n\
        h1111204たしかにそういう除霊の方法もあるけれど、\\n\
        私には効かないから心配はいらないわ。\\n\\
        h1111205それに絵に描いてもらえるなんて、願ってもないことだもの。\\n\
        h1111210描きあげたら、また見せてね。\
        ".to_string(), vec![]),

      ],
      TalkType::Lore => vec![

        ("冥界の渡し賃", "\
        h1111206古代ギリシャでは死者に銅貨を持たせて葬っていたの。\\n\
        h1111210冥界には川を渡っていかなければならなかったから、\\n\
        渡し賃を持たせて快適な旅を願う……ということね。\\n\\n\
        h1111205死者が川を越えていくという伝承は世界中で見られるわ。\\n\
        彼らにとって、境界線といえばまず川が連想されたのかしら。\\n\
        h1111210あなたなら、あの世とこの世の間にはなにがあると思う？\
        ".to_string(), vec![]),

        ("死体のうめき声", "\
        h1111205死体は、うめき声を上げることがあるのよ。\\n\
        h1111206……といっても、体内のガスが口から噴き出すとき、\\n\
        声帯が震えて音が出る……ただそれだけのことなのだけど。\\n\
        それでも、そんな些細なことが恐怖をかきたてて、\\n\
        人々は怪物を想像する。\\n\
        ……h1111201呆れるほどに多彩で、\\n\
        身近に根ざした感情の象徴だわ。\
        ".to_string(), vec![]),

        ("屍蝋", "\
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
        ".to_string(), vec![]),

        ("死後の温かさ", "\
        h1111205死後数日経ったはずの身体が、まだ温かい。\\n\
        h1111210それは微生物が分解を行ったときに生じた熱のせいよ。\\n\
        ガスで膨張もするから、\\n\
        生前よりふくよかで健康的に見えることすらあったみたい。\\n\
        ……h1111204死体が蘇って夜な夜な彷徨い歩く、\\n\
        あるいは夢枕に立って生命を吸い取るという民話は、\\n\
        そんな様子に理由をつけるためのものじゃないかしら。\
        ".to_string(), vec![]),

        ("生長する死体", "\
        h1111205掘り起こした死体の髪や爪が伸びていた！\\n\
        h1111210土葬が一般的だった時代、たびたびあった話。\\n\
        乾燥して縮むから、皮膚の下の髪や爪が露出する。\\n\
        それがまるで生長しているように見えたの。\
        ".to_string(), vec![]),

        ("土葬の空洞", "\
        h1111206土葬の場合、地中の遺体が朽ちるとそこに空洞ができるわ。\\n\
        h1111205「死体に足を引っ張られる」という伝承は、\\n\
        これを踏み抜いてしまっただけかもしれないわね。\\n\
        h1111210あなたも墓地を歩くときは気をつけて……って、\\n\
        h1111204あなたの住む場所にそんなところは少ないかしら。\
        ".to_string(), vec![]),

        ("永遠の夢", "\
        h1113105恒久の平和、不死の身体、永劫の繁栄……。\\n\
        h1113204永遠を夢見た人物の多くは失敗していて、その代償を払っている。\\n\
        ときに命を落としていることもあるわね。\\n\
        ……h1113101求めるのは間違ったことなのかしら？\
        ".to_string(), vec![]),

        ("生体電気", "\
        h1111206カエルの足に電流を流す実験。\\n\
        生体電気の発見に繋がったといわれる\\n\
        あの現象は、\\_a[Misemono,どんな見世物だったの？]死者の蘇りを謳う見世物\\_aに\\n\
        利用されたことがあったの。\
        ".to_string(), vec![]),

        ("死者の埋葬", "\
        h1111206古代ギリシャでは、刑死の際は毒薬に阿片を混ぜたものを飲ませていたの。\\n\
        h1113210それは死の苦しみを和らげるためだったのかもしれないけれど、\\n\
        それ以上に、死を恐れる人々を抑えるためだったのかもしれないわね。\\n\
        h1113205罰ではあれど、必要以上に苦しませることはない、と。\
        ".to_string(), vec![]),

        ("死後の存在", "\
        h1111210幽霊、霊体、死後の存在。人類の科学は、そういったものにまだ答えを出していない。\\n\
        h1111205存在する、しないの議論は、h1112205まあ、私たちには必要ないわね。\\n\
        h1111210……いつかその時が来るのかしら。霊体を観測し、干渉し……あるいは、消滅させる方法。\\n\
        h1111205ふふ。私、期待しているの。\
        ".to_string(), vec![]),

        ("黒死病", "\
        h1111210黒死病が蔓延していたとき、問題になっていたのがいわゆる「早すぎた埋葬」。\\n\
        h1111205ある技師は生き埋めにされる恐怖から逃れるため、\\n\
        埋葬者が生きていることを棺の内側から知らせる仕組みがついた棺を開発したの。\\n\
        h1111204彼、デモンストレーションのために自ら生き埋めになってみせたそうよ。\\n\
        h1212210自分で出られない状態で、冷たい土の下へ。\\n\
        ……h1211506どんな心地がしたのかしらね。\
        ".to_string(), vec![]),
      ],
      TalkType::Past => vec![
        ("惨めな人生", "\
        h1111205みじめな人生の上に正気でいるには、\\n日々は長すぎたの。\
        ".to_string(), vec![]),

        ("行き場のない苦しみ", "\
        h1112202誰が悪い？いいえ、誰も悪くない。\\n\
        打ち明けたところで、的はずれな罪悪感を生むだけ。\\n\
        h1112205だからといって、他人に責をなすりつけるほど鈍くあることもできなかった。\\n\
        h1112210この気持ちには、どこにも行き場がなかったの。\
        ".to_string(), vec![]),

        ("後悔", "\
        h1111205私に救いは訪れなかった。\\n\
        想いは、今もずっと、私の中にある。\\n\
        あなたが、私を救える人だったら良かh1111101……。\\n\
        ……h1111110。いえ、死んだ後で報われようだなんて。\\n\
        h1121205……h1121305悪いわね。\
        ".to_string(), vec![]),

        ("死の瞬間", "\
        h1111205死ぬ瞬間、後悔はなかった。\\n\\n\
        もう一度同じ人生を生きることができたとしても、私は同じことをすると断言できるわ。\\n\
        ……h1121210ただ、遺書くらいは書いたほうがよかったかしら。\
        ".to_string(), vec![]),

        ("助けは遂げられず", "\
        h1111205助けようとしてくれた人は沢山いたけれど、\\n\
        h1121210それが遂げられることはついぞなかったわ。\
        ".to_string(), vec![]),

        ("死なない理由", "\
        h1111210生きていて良かったと思えることは数えきれないほどあったわ。\\n\
        h1111205でも、死なない理由は一つも見つからなかった。\
        ".to_string(), vec![]),

        ("守ってくれた人", "\
        h1111105あの人は、私を守ってくれた。\\n\
        でも、私を救えはしなかった。\\n\
        理解と共感は、違う。h1112105……違うのよ。\
        ".to_string(), vec![]),

        ("ふつうになりたかった", "\
        h1122210ふつうになりたかった。\\n\
        ……h1122205でも、ふつうだったら、もう私じゃないとも思う。\\n\
        それは私の顔をした別のだれかで、\\n\
        私は私の性質と不可分で、\\n\
        今ここにいる私は、私以外でいられない。\\n\
        h1122210だから、私として生きることができなかった私は、もうどこにもいられない。\
        ".to_string(), vec![]),

        ("人と本", "\
        h1111205昔から、人と本の違いがわからなかったの。\\n\
        h1121204もちろん、区別がつかないという意味ではなくて。\\n\
        ……h1111210人に期待するものがそれだけしか無かったの。\
        ".to_string(), vec![]),

        ("分厚い本", {
            let (topic, description) = random_book_topic();
            format!("\
                h1111204……手持ち無沙汰のようね。\\n\
                h1111206なにか本を見繕ってあげましょうか。\\n\
                h1111203……h1111201これはどうかしら。\\n\
                \\1……ずいぶん分厚い本を手渡された。\\n\
                h1111202{}の構成要素について論じられているの。\\n\
                {}についての項が特に興味深いわ。\
                h1111205要点だけなら半日もあれば読み終わると思うから、\\n\
                h1111204終わったら意見を交換しましょう。\
            ",topic,description)
        }, vec![]),

        ("今度こそ無へ", "\
        h1111205死にぞこなったものだから、\\n\
        次の手段を求めている。\\n\
        ……h1112305今度こそ、終わらせたいの。\\n\
        今度こそ、無へ。\
        ".to_string(), vec![]),

        ("魂は消える", "\
        h1111110未練もなく、しかし現世に留まっている魂。\\n\
        h1111105あるべきでないものはやがて消滅する。\\n\
        h1111206多少の不純物が含まれようと、そのルールは変わらない。\\n\
        h1111205私は、それを待ち望んでいるの。\
        ".to_string(), vec![]),

        ("月の満ち欠けのように", "\
        h1111205月の満ち欠けのように、私の心は移り変わる。\\n\
        h1111210理解を得ることは難しかったわ。\\n\
        そんな仕打ちも、納得はできる。自分ですら不可解なのだから。\\n\
        ……h1121306少しでも自分で律することができたなら、\\n\
        こんなに苦しむことはなかっただろうに。\
        ".to_string(), vec![]),

      ],
      TalkType::Abstract => vec![

        ("自己理解、他者理解", "\
        h1111205自分のことを本当に理解しているのは他人、って本当なのかしら。\\n\
        h1111206……私が知らない私がいる。\\n\
        h1112204なんだか不安になってきたわ。\
        ".to_string(), vec![]),

        ("感動と倦み", "\
        h1111205ある本を最初に読んだときの感動と、何度も読み返して全て見知ったゆえの倦み。\\n\
        どちらがその本の真の印象かしら。\\n\\n\
        h1111210私はどちらも正しいと思うの。\\n\
        ……h1111504卑怯だと思った？\\n\
        h1111210印象なんてその時々で変わるもので、h1111205一つに定まることなんて稀だもの。\\n\\n\
        まして、自分の中に秘めるものならなおさら。\\n\
        h1111506どちらか一方だけだなんて、勿体ないわ。\
        ".to_string(), vec![]),

        ("納得のための因果", "\
        h1111210因果が巡ってきた。\\n\
        過去が現在を刈り取りに来た。\\n\
        私は報いを受けたのだ。\\n\\n\
        ……h1111205それが、自分を納得させるための妄想だったとしたら？\
        ".to_string(), vec![]),

        ("怖いものを見るということ", "\
        h1111201怖いものだからこそ、見つめなければ戦えない。\\n\
        ……h1121205そんなもの、戦える人のためだけの論理だわ。\
        ".to_string(), vec![]),

        ("停滞を終わらせるために", "\
        h1111205危険と隣り合わせだからこそ、世界は美しいの。\\n\
        身を損なう心配がなくなっては、美しさが心を打つこともない。\\n\
        h1121205ただただ平坦な、揺らがぬ水面があるだけ。\\n\
        h1121210それはやがて、淀み、腐る。\\n\
        h1111205願わくば、せめて終わりがありますように。\
        ".to_string(), vec![]),

        ("停滞の破壊", "\
        h1111105人生に変化は付きもの……けれどh1111110停滞はそれ以上。\\n\
        一度立ち止まってしまうと、空気は一瞬で淀んで、身動きがとれなくなってしまう。\\n\
        それは倦怠とも違う、鈍い痛み。\\n\
        h1111201あなた、h1111205もしそうなったときは、多少無理にでも変化を取り入れるほうがいいわ。\\n\
        ……h1111210たとえなにかを破壊することになるとしても、何も出来ないよりはずっとましよ。\
        ".to_string(), vec![]),

        ("極限の変化としての死", "\
        h1111205死の瞬間の、極限に振れた変化。\\n\
        命が命でなくなり、身体が陳腐な肉の塊になる、その一瞬が愛しくてたまらない。\\n\
        どうしようもなく、愛しいの。\\n\\n\
        ".to_string(), vec![]),

        ("死の向こう側", "\
        h1112110どうか、死の向こう側がありませんように。\
        ".to_string(), vec![]),

        ("沈んでいく", "\
        h1111105沈んでいく。\\n\
        手がどうしても動かなくて、目の前の希望を掴めない。\\n\
        身体が重い。浅い呼吸のなかで、沈んでいく自分の身体を感じていることしかできない。\\n\
        私は、私を救うことを諦めているみたい。\\n\
        h1111110どうして。\\n\
        h1121205どうして、こうなってしまったのだろう。\
        ".to_string(), vec![]),

        ("人を解体したい", "\
        h1111110人を解体したいと、思うことがあるの。\\n\
        何が人を人たらしめているのか、どこまで分解すれば人は人でなくなるのか。\\n\
        h1111105人という恐ろしく不可解な物の、どこにその根源があるのか。\\n\
        それを知るには、他に方法が思いつかないの。\
        ".to_string(), vec![]),

        ("わがままな祈り", "\
        h1111210がんばっているってこと、\\n\
        理解できなくても見ていてほしかったの。\\n\
        ……h1121205わがままかしら。\
        ".to_string(), vec![]),

        ("生者にとっての慰め", "\
        h1111210枯れ木に水をあげましょう。\\n\
        もはや花は見れずとも、それが慰めとなるのなら。\\n\
        \\n\
        h1111205それは誰にとって？\\n\
        h1111206もちろん、死を悼む者にとっての慰めよ。\\n\
        むくろに心はないもの。\
        ".to_string(), vec![]),

        ("不可逆な崩壊", "\
        h1111210燃え殻がひとりでに崩れるように、心が静かに割れて戻らなくなった。\\n\
        h1111205だから、諦めたの。\
        ".to_string(), vec![]),

        ("中途半端な助け", "\
        h1111210中途半端な助けは何もしないより残酷だわ。\\n\
        h1111205希望を持たせておいて、それを奪うのだもの。\
        ".to_string(), vec![]),

        ("レンズの歪み", "\
        h1111205観察と模倣を続ければ、完全に近づけると思っていた。\\n\
        想定外だったのは、レンズが歪んでいたことと、それを取り替える方法がなかったこと。\\n\
        h1121310そうなればすべて台無し。h1121304諦めるしかなかったわ。\
        ".to_string(), vec![]),

        ("先の見えない苦しみ", "\
        h1111205一寸先は暗く、扉は閉ざされている。\\n\
        不明な道のりを諸手で探るよりも、\\n\
        h1112305目先の手首を切り裂くほうが遥かに明瞭なのだ！\\n\
        ……h1111210なんてね。\
        ".to_string(), vec![]),

        ("唯一の視点", "\
        h1111206私たちは、自我という色眼鏡を通してしか世界を観測できない。\\n\
        h1111204あなたは目の前にいるのに、\\n\
        あなたが見る世界を私が知ることはできないの。\\n\
        h1112210それって、この上なく残酷なことだわ。\
        ".to_string(), vec![]),

        ("一つの個としての限界", "\
        h1111203世界が複雑で曖昧すぎるから、\\n\
        私たちは認識したものを理解できる形に歪めてしまう。\\n\
        h1111210既存の分類に当て嵌めて、安心を優先するの。\\n\
        曇る視界と引き換えにね。\\n\
        ……h1111204あなたには、私はどう見えているのかしら？\\n\
        ".to_string(), vec![]),

        ("自己同一性の仮定", "\
        h1111205環境と経験の総体こそが、\\n\
        自己であるような気がするの。\\n\
        自己同一性すら偶然の産物？\\n\
        h1111210執着しているのが馬鹿馬鹿しく思えてくるわ。\\n\
        h1111205仮にそうでなければ。\\n\
        ……自己は最初から決定されている？\\n\
        h1111210それこそ、ね。\\n\
        ".to_string(), vec![]),

        ("自分の理解者は自分だけ", "\
        h1111210「なぜみんな私をわかってくれないの？」と誰もが思う。\\n\
        h1111205答えは簡単。他人があなたではなく、あなたが他人でないからよ。\\n\
        畢竟、あなた以外にあなたを理解できるひとはいないの。\
        ".to_string(), vec![]),

        ("得ることは失うこと", "\
        h1111210あまねく変化は表裏一体。\\n\
        h1111206何かを得るとき、選択は慎重になさい。\\n\
        h1111205それは失うものをも左右するのだから。\\n\
        ".to_string(), vec![]),
      ],
    };

  let vars = get_global_vars();
  let mut talks = Vec::new();
  for s in strings {
    if s.2.iter().all(|flag| vars.flags().check(flag)) {
      talks.push(Talk::new(Some(talk_type), s.0, s.1.to_string()));
    }
  }
  talks
}

#[allow(dead_code)]
impl Talk {
  pub fn new(talk_type: Option<TalkType>, id: &'static str, text: String) -> Self {
    Self {
      talk_type,
      text,
      id,
    }
  }

  pub fn from_vec_with_type(talk_type: TalkType, texts: Vec<(&'static str, String)>) -> Vec<Self> {
    texts
      .into_iter()
      .map(|t| Self::new(Some(talk_type), t.0, t.1))
      .collect()
  }

  pub fn all_talks() -> Vec<Talk> {
    let mut v = Vec::new();
    for t in TalkType::all() {
      v.extend(random_talks(t));
    }
    v
  }

  pub fn get_unseen_talks(talk_type: TalkType, seen: &HashSet<String>) -> Vec<Talk> {
    random_talks(talk_type)
      .into_iter()
      .filter(|t| !seen.contains(t.id))
      .collect()
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash, EnumIter)]
pub enum TalkType {
  SelfIntroduce,
  Lore,
  Past,
  Abstract,
  WithYou,
}

impl Display for TalkType {
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    let s = match self {
      Self::SelfIntroduce => "ハイネ自身の話題",
      Self::Lore => "ロア/オカルト",
      Self::Past => "ハイネの過去についての話題",
      Self::Abstract => "抽象的な話題",
      Self::WithYou => "あなたについての話題",
    };
    write!(f, "{}", s)
  }
}

impl TalkType {
  pub fn from_u32(n: u32) -> Option<Self> {
    Self::all().into_iter().find(|t| *t as u32 == n)
  }

  pub fn all() -> Vec<Self> {
    Self::iter().collect()
  }
}

pub fn random_talks_analysis() -> String {
  let mut s = String::new();
  let mut sum = 0;
  for talk_type in TalkType::all() {
    let len = random_talks(talk_type).len();
    s.push_str(&format!("{:?}: {}\\n", talk_type, len,));
    sum += len;
  }

  format!(
    "\\_q{}
    ---\\n\
    TOTAL: {}",
    s, sum
  )
}

#[allow(dead_code)]
static BOTSU: Lazy<Vec<(&str, String)>> = Lazy::new(|| {
  vec![

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TalkingPlace {
  LivingRoom,
  Library,
}

impl Display for TalkingPlace {
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    let s = match self {
      Self::LivingRoom => "客間",
      Self::Library => "書斎",
    };
    write!(f, "{}", s)
  }
}

impl TalkingPlace {
  pub fn balloon_surface(&self) -> u32 {
    match self {
      Self::LivingRoom => 0,
      Self::Library => 6,
    }
  }

  pub fn talk_types(&self) -> Vec<TalkType> {
    match self {
      Self::LivingRoom => vec![TalkType::SelfIntroduce, TalkType::Lore, TalkType::WithYou],
      Self::Library => vec![TalkType::Abstract, TalkType::Past],
    }
  }
}

pub static FIRST_RANDOMTALKS: Lazy<Vec<String>> = Lazy::new(|| {
  vec!["\
      h1111206いま、別の者にお茶の準備をさせているわ。\\n\
      \\n\
      h1111201さて。\\n\
      h1111204それで、ここには死ににきたのですってね。\\n\
      \\n\
      h1113210……相当な覚悟がなければ決断できなかったでしょう。\\n\
      h1113205なにがあなたを追い詰めたのかしらね。\\n\
      仕事、学問、病気、怪我、家庭環境……h1113206あるいは、名前もつけられないほど微妙で、それでいて耐えられないなにか。\
      \\c\
      h1111204ああ、話せなんて言わないわ。\\n\
      話したければそうすればいいけれど、まあ、今はそうではないでしょう？\\n\
      \\n\
      それに、それを聞いて、私があなたを救うこともない。\\n\
      私の立場でできることなどたかが知れているし、\\n\
      あなたの生死は、h1111210なんというか、あまり大きな関心事ではないから。\
      \\x\
      ……h1111201あなたに興味がないわけではないのよ。\\n\
      h1111210もしそうならば引き止めたりせずに追い出しているわ。\\n\
      h1111204勘違いしてほしくないのは、\\n\
      あなたの宿痾に特別な興味はないということ。\\n\
      h1111210あなたの食事の好み、\\n\
      やめられない癖、\\n\
      過去に犯した罪、\\n\
      好みの散歩ルート、\\n\
      なんでもいいの。\\n\
      h1111205この倦み腐った脳に刺激を与えてくれるなら、情報に貴賤はないわ。\
      \\x\
      ……h1111210なにせ、私はここを出られない身だから。\\n\
      h1112205ずっと、ずっと、ずっと、退屈だったの。\\n\
      h1111204そんなところにあなたという玩具が\\n\
      転がりこんできたものだから、\\n\
      都合のいい遊び道具として使おうと考えたのよ。\\n\
      \\n\
      ……h1111210ふふ、ここまで言われても怒らないのね。\\n\
      とても、根深い。h1111211ますます気に入ったわ。\
      \\x\
      h1111204ともかく。逝きたくなったら、そうすればいい。\\n\
      必要ならばいくらか手助けもしましょう。\\n\
      ただ、それまでは私の話し相手になってもらうわ。\\n\
      いいわねφ？\\_w[750]{user_name}。\
      ".to_string(),

      "\
      \\1(コンコン)\\_w[1250]\\n\
      h1111203入りなさい。\\n\\1ハイネの後方、客室のドアが開き、なにかが静かに入ってきた。\\n\
      ……カップとポット。ティーセットだ。浮いている。透明な何かに支えられているかのようだ。\\n\
      \\n\
      h1111203契約している霊よ。召使のようなもの。\\n\
      ……h1141101ああ、h1111204あなたには見えないのかしら？低級霊だものね。\\n\
      \\n\
      ……h1111206そう、ならば伝えておく必要があるわね。\\n\
      この部屋には、常に数人の召使がついているの。\\n\
      h1111201……h1111204驚いた？まあ、普段は居ないのとそう変わらないわ。\\n\
      お茶のおかわりが欲しかったら、h1111306そうね……あのあたりに合図を送ればいいわ。\\n\
      \\x\
      h1111204彼らはみな、私と契約している霊たち。\\n\
      私は力が強いから、それで彼らの……まあ、身の安全を保障しているの。\\n\
      代わりに、彼らは私の命令に従う。\\n\
      これ以上なく忠実にね。\\n\
      \\x\
      h1111203霊にとっての契約は、生者のそれとは重みが違うわ。\\n\
      h1111206なにせ、文字通り存在をかけて結ぶものだから。\\n\
      なまなかな誤魔化しはきかず、反故にすることもできない。\\n\
      h1113210肉体は重くはあれど強固な鎧だったのだと、死んだ後からしみじみ思うわ。\
      \\x\
      h1111204……混乱しているかしら。\\n\
      今すぐにすべてを理解する必要はないわ。\\n\
      どうせ時間はたっぷりあるのだから。\\n\
      \\n\
      ほら、お茶を飲んで。\\n\
      \\1勧められるままに、私は深赤色の液体を口に含んだ。\\n\
      ……まろやかな苦味の後に、薬草を想わせる複雑な香りが広がる。\\n\
      深く、コクのある味わいだ。……美味しい。\\n\
      h1111211悪くないでしょう？h1111204それが私のお気に入り。\\n\
      ……h1111211これで、私の好物を1つ知れたわね。\\n\
      h1111204全て、このように。\\n\
      じっくり、互いを知っていけばいいのよ。\
      ".to_string()
  ]
});

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
      let mut res =
        new_response_with_value(text.clone(), TranslateOption::with_shadow_completion());
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
      "\\0\\![bind,ex,流血,0]h1111705ふー…………。\\n\\1ハイネは深く息を吐いた。……落ち着いたようだ。".to_string(),
    ]];
    talk_parts.push(if !vars.flags().check(&EventFlag::FirstHitTalkDone) {
      vars.flags_mut().done(EventFlag::FirstHitTalkDone);
      vec!["……h1111204これで終わり？そう。\\n\
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
            TalkingPlace::Library => vec!["h1111204あなた、書斎は初めてね。\\n\
            \\1……客間より少し狭い程度の間取りに、所狭しと本棚が設置されている。\\n\
            窓すら本棚に覆われていて、ハイネは蝋燭の灯りで本を読んでいるようだった。\\n\
            h1111210ここは私の私室でもあるの。\\n\
            h1111204……あなたは、本を読むのは好き？\\n\
            h1111306私は好きよ。巨人の肩に乗って遠くが見える。\\n\
            h1111305あるいは、ここではないどこかへ、遠くへ行ける。\
            h1111204あなたも自由に読み、そして考えなさい。\\n\
            h1111310ここはそういう場所よ。\
            "
            .to_string()],
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
    format!("\\0\\![set,balloonnum,{}]{}", comment, choosed_talk.text),
    TranslateOption::with_shadow_completion(),
  )
}

pub fn register_talk_collection(talk: &Talk) {
  let mut talk_collection = get_global_vars().talk_collection_mut();
  match talk_collection.get_mut(&talk.talk_type.unwrap()) {
    Some(t) => {
      let key = talk.id.to_string();
      if !t.contains(&key) {
        t.insert(key);
      }
    }
    None => {
      talk_collection.insert(
        talk.talk_type.unwrap(),
        HashSet::from_iter(vec![talk.id.to_string()]),
      );
    }
  }
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
