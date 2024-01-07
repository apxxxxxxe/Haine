use crate::events::common::*;
use crate::variables::get_global_vars;
use core::fmt::{Display, Formatter};
use once_cell::sync::Lazy;
use rand::prelude::*;
use shiorust::message::{parts::HeaderName, Request, Response};

#[derive(Clone)]
pub struct Talk {
  pub talk_type: Option<TalkType>,
  pub text: String,
}

impl Display for Talk {
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    write!(f, "{}", self.text)
  }
}

impl Talk {
  pub fn new(talk_type: Option<TalkType>, text: String) -> Self {
    Self { talk_type, text }
  }

  pub fn from_vec(texts: Vec<String>) -> Vec<Self> {
    texts.into_iter().map(|t| Self::new(None, t)).collect()
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
      Self::Lore => "ロアの話題",
      Self::Past => "ハイネの過去についての話題",
      Self::Abstract => "抽象的な話題",
      Self::WithYou => "あなたについての話題",
    };
    write!(f, "{}", s)
  }
}

impl TalkType {
  fn all() -> Vec<Self> {
    vec![
      Self::SelfIntroduce,
      Self::Lore,
      Self::Past,
      Self::Abstract,
      Self::WithYou,
    ]
  }

  pub fn all_talks() -> Vec<Talk> {
    let mut v = Vec::new();
    for t in Self::all() {
      v.extend(t.talks());
    }
    v
  }

  fn talks(&self) -> Vec<Talk> {
    let strings = match self {
      Self::SelfIntroduce => vec![
        "\
        h1111205ある本を最初に読んだときの感動と、何度も読み返して全て見知ったゆえの倦み。\\n\
        どちらがその本の真の印象かしら。\\n\\n\
        h1111209私はどちらも正しいと思うの。……卑怯だと思う？\\n\
        印象なんてその時々で変わるもので、h1111205一つに定まることなんて稀だもの。\\n\\n\
        正反対の感想を同時に抱いたっていいのよ。\\n\
        どちらか一方だけだなんて、勿体ないもの。\
        ".to_string(),

        "\
        h1111203たまに、街に出ることもあるのよ。\\n\
        私が持っているだけの時間を費やすには、ここの蔵書だけでは足りないから。\\n\
        ……h1111209大丈夫よ。館を出れば、私に気付く人はいない。\\n\
        h1111205私の存在の根は、どうしたってここにあるの。\
        "
        .to_string(),

        "\
        h1111205幽霊は生前の想い……好みや恨みに執着するの。\\n\
        h1111209想い人がいればその人に。恨みがあればその相手に。\\n\
        h1111203逆に、死後新たな執着が生まれることはほとんどないわ。\\n\
        だから幽霊同士、h1111206ましてや人と幽霊の間に恋愛が生まれることは皆無といっていいでしょう。\\n\
        h1111304……なに、その顔は。h1111309あいにく、私は生きていた頃から恋愛とは無縁よ。\\n\
        "
        .to_string(),

        "\
        h1111206\\1ポットがひとりでに浮き、空になっていたカップに飲み物が注がれる。\\n\
        h1111206私の元へ集うのは弱い人たち。\\n\
        自分だけでは溶けゆく自我を押し留められず、さりとてそれを受け入れることもできない霊。\\n\
        h1111209役割を与えてあげるの。一種の契約ね。\\n\
        h1111205使命に縛られはするけれど、自分を保てるわ。\\n\
        ".to_string(),

        "\
        h1111203カンテルベリオには、霊が集まりやすい土壌があるの。\\n\
        h1111206正確には、死の意識が。\\n\
        ……h1111209あなたがここに来たのも偶然ではないのかもね。\\n\
        ".to_string(),

        "\
        h1111209ここは私の生家なの。実際は別荘なのだけど。\\n\
        h1111205そして、ここで死んだ。\\n\
        h1111209だから私はここの主となった。\\n\
        ……h1121304静かに眠りたかったのに、ね？\\n\
        h1121305貧乏くじを引いたものだわ。\
        ".to_string(),
      ],
      Self::Lore => vec![
        "\
        h1111205食事の時間になっても部屋から出てこない家族。\\n\
        扉を開けてみると、彼女は足の一部を残して焼死していた。\\n\
        ……h1111206人体発火現象は、世界中で見られるわ。\\n\
        h1111209多くの場合火気はなく、発火の原理は不明。\\n\
        h1111206さらに、いくらかの延焼はあれど周囲には被害が及ばない。\\n\
        h1111209まったく不思議な話よね。h1111204あなたはどう考える？\
        "
        .to_string(),
        "\
        h1111206古代ギリシャでは、刑死の際は毒薬に阿片を混ぜたものを飲ませていたの。\\n\
        h1113209それは死の苦しみを和らげるためだったのかもしれないけれど、\\n\
        それ以上に、死を恐れる人々を抑えるためだったのかもしれないわね。\\n\
        h1113205罰ではあれど必要以上に苦しませることはない、と。\
        "
        .to_string(),
        "\
        h1111209幽霊、霊体、死後の存在。人類の科学は、そういったものにまだ答えを出していない。\\n\
        h1111205存在する、しないの議論は、h1112205まあ、私たちには必要ないわね。\\n\
        h1111209……いつかその時が来るのかしら。霊体を観測し、干渉し……消滅させる方法。\\n\
        h1111205私、期待しているの。\
        "
        .to_string(),
        "\
        h1111209黒死病が蔓延していたとき、問題になっていたのがいわゆる「早すぎた埋葬」。\\n\
        h1111205ある技師は生き埋めにされる恐怖から逃れるため、\\n\
        埋葬者が生きていることを棺の内側から知らせる仕組みがついた棺を開発したの。\\n\
        h1111204彼、デモンストレーションのために自ら生き埋めになってみせたそうよ。\\n\
        h1212209自分で出られない状態で、冷たい土の下へ。\\n\
        ……h1212205どんな心地がしたのかしらね。\
        "
        .to_string(),
      ],
      Self::Past => vec![
        "\
        h1111205みじめな人生の上に正気でいるには、\\n日々は長すぎたの。\
        "
        .to_string(),
        "\
        h1112202誰が悪い？いいえ、誰も悪くない。\\n\
        打ち明けたところで、的はずれな罪悪感を生むだけ。\\n\
        h1112205だからといって、他人に責をなすりつけるほど鈍くあることもできなかった。\\n\
        h1112209この気持ちには、どこにも行き場がなかったの。\
        "
        .to_string(),
        "\
        h1111205私に救いは訪れなかった。\\n\
        想いは、今もずっと、私の中にある。\\n\
        あなたが、私を救える人だったら良かh1111101……。\\n\
        ……h1111109……。いえ、死んだ後で報われようだなんて。\\n\
        h1121205…………h1121305ごめんなさい。\
        "
        .to_string(),
        "\
        h1111205死ぬ瞬間、後悔はなかった。\\n\\n\
        もう一度同じ人生を生きることができたとしても、私は同じことをすると断言できるわ。\\n\
        ……h1121209ただ、遺書くらいは書いたほうがよかったかしら。\
        "
        .to_string(),
        "\
        h1111205助けようとしてくれた人は沢山いたけれど、\\n\
        h1121209それが遂げられることはついぞなかったわ。\
        "
        .to_string(),
        "\
        h1111209生きていて良かったと思えることは数えきれないほどあったわ。\\n\
        h1111205でも、死なない理由は一つも見つからなかった。\
        "
        .to_string(),
        "\
        h1111105あの人は、私を守ってくれた。\\n\
        でも、私を救えはしなかった。\\n\
        理解と共感は、違う。h1112105……違うのよ。\
        "
        .to_string(),
        "\
        h1122209ふつうになりたかった。\\n\
        ……h1122205でも、ふつうだったら、もう私じゃないとも思う。\\n\
        それは私の顔をした別のだれかで、\\n\
        私は私の性質と不可分で、\\n\
        今ここにいる私は、私以外でいられない。\\n\
        h1122209だから、私として生きることができなかった私は、もうどこにもいられない。\
        "
        .to_string(),
        "\
        h1111205昔から、人と本の違いがわからなかったの。\\n\
        h1121204もちろん、区別がつかないという意味ではなくて。\\n\
        ……h1111209人に期待するものがそれだけしか無かったの。\
        "
        .to_string(),
        "\
        h1111205死にぞこなったものだから、次の手段を求めている。\\n\
        ……h1112305今度こそ、終わらせたいの。\\n\
        今度こそ、無へ。\
        ".to_string(),
        "\
        h1111109未練もなく、しかし現世に留まっている魂。\\n\
        h1111105あるべきでないものはやがて消滅する。\\n\
        h1111206多少の不純物が含まれようと、そのルールは変わらない。\\n\
        h1111205私は、それを待ち望んでいるの。
        ".to_string(),
      ],
      Self::Abstract => vec![
        "\
        h1111209因果が巡ってきた。\\n\
        過去が現在を刈り取りに来た。\\n\
        私は報いを受けたのだ。\\n\\n\
        ……h1111205それが、自分を納得させるための妄想だったとしたら？\
        "
        .to_string(),
        "\
        h1111201怖いものだからこそ、見つめなければ戦えない。\\n\
        ……h1121205そんなもの、戦える人のためだけの論理だわ。\
        "
        .to_string(),
        format!(
          "\
        h1111205危険と隣り合わせだからこそ、世界は美しいの。\\n\
        身を損なう心配がなくなっては、美しさが心を打つこともない。\\n\
        h1121205ただただ平坦な、揺らがぬ水面があるだけ。\\n\
        h1121209それはやがて、淀み、腐る。\\n\
        h1111205願わくば、せめて終わりがありますように。\
        "
        ),
        "\
        h1111105人生に変化は付きもの……けれどh1111109停滞はそれ以上。\\n\
        一度立ち止まってしまうと、空気は一瞬で淀んで、身動きがとれなくなってしまう。\\n\
        それは倦怠とも違う、鈍い痛み。\\n\
        h1111201あなた。h1111205もしそうなったときは、多少無理にでも変化を取り入れるほうがいいわ。\\n\
        ……h1111209たとえなにかを破壊することになるとしても、何も出来ないよりはずっとましよ。\
        "
        .to_string(),
        "\
        h1111205死の瞬間の、極限に振れた変化。\\n\
        命が命でなくなり、身体が陳腐な肉の塊になる、その一瞬が愛しくてたまらない。\\n\
        どうしようもなく、愛しいの。\\n\\n\
        h1111209……幻滅するでしょう。あなたの目には、贔屓目にも綺麗には写らない。\\n\
        h1111309……でもね。倫理は私を救わない。\\n\
        あなたの心を私が握る必要も、ない。\\n\
        h1111304それを諦められるくらい、私は、これを諦められないのだから。\
        "
        .to_string(),
        "\
        h1112109どうか、死の向こう側がありませんように。\
        "
        .to_string(),
        "\
        h1111105沈んでいく。\\n\
        手がどうしても動かなくて、目の前の希望を掴めない。\\n\
        身体が重い。浅い呼吸のなかで、沈んでいく自分の身体を感じていることしかできない。\\n\
        私は、私を救うことを諦めているみたい。\\n\
        h1111109どうして。\\n\
        h1121205どうして、こうなってしまったのだろう。\
        "
        .to_string(),
        "\
        h1111109人を解体したいと、思うことがあるの。\\n\
        何が人を人たらしめているのか、どこまで分解すれば人は人でなくなるのか。\\n\
        h1111105人という恐ろしく不可解な物の、どこにその根源があるのか。\\n\
        それを知るには、他に方法が思いつかないの。\
        "
        .to_string(),
        "\
        h1111209がんばっているってこと、\\n\
        理解できなくても見ていてほしかったの。\\n\
        ……h1121205わがままかしら。\
        "
        .to_string(),
        "\
        h1111209枯れ木に水をあげましょう。\\n\
        もはや花は見れずとも、それが慰めとなるのなら。\\n\
        \\n\
        h1111205それは誰にとって？\\n\
        h1111206もちろん、死を悼む者にとっての慰めよ。\\n\
        むくろに心はないもの。\\n\
        "
        .to_string(),
        "\
        h1111209燃え殻がひとりでに崩れるように、心が静かに割れて戻らなくなった。\\n\
        h1111205だから、諦めたの。\
        "
        .to_string(),
        "\
        h1111209中途半端な助けは何もしないより残酷だわ。\\n\
        h1111205希望を持たせておいて、それを奪うのだもの。\
        "
        .to_string(),
        "\
        h1111205観察と模倣を続ければ、完全に近づけると思っていた。\\n\
        想定外だったのは、レンズが歪んでいたことと、それを取り替える方法がなかったこと。\\n\
        \\n\
        h1111209それに気づいた時の、あのときの私は、完全に最も近かったわ。\
        ".to_string(),
        "\
        h1111205一寸先は暗く、扉は閉ざされている。\\n\
        不明な道程(みちのり)を手で探るよりも、\\n\
        h1112305目先の手首を切り裂くほうがはるかに明瞭なのだ！\
        ".to_string(),
      ],
      Self::WithYou => vec![
        "\
        h1111209今は手軽に写真が撮れていいわね。\\n\
        h1111205印象的な光景を、いつでも手元に残しておける。\\n\
        ……h1111201あら、私？h1121204光栄だけれど、残念ながら写真には写らないわ。\
        ".to_string(),

        "\
        h1111204……なにか不安なの？\\n\
        それとも、何が不安かもわからない？\\n\
        h1111209……紙に書き出してみるのはどうかしら。\\n\
        h1111205いくらか整理がつくかもしれない。\
        ".to_string(),

        "\
        h1111209茶葉や茶菓子はあなたが持ってきてね。\\n\
        h1111206ここにある分は限られているし、私一人では補充する手段がないから。\\n\
        h1111209あなたが食べられるようなものは、私には用意できないの。\
        ".to_string(),

        "\
        h1111204あなたは、ちゃんと食べているかしら？\\n\
        h1111209そう。いいことね。\\n\
        h1111104私？……h1111205生前は食が細かったわ。\\n\
        h1111209……というより、食そのものにあまり関心がなかったみたい。\\n\
        ……h1123309思えば、使用人には随分と世話をかけたわね。\
        ".to_string(),

        format!("\
        h1111206霧が、濃いでしょう。\\n\
        ただの霧ではないの。むしろ、性質としては私たちに近い。\\n\
        h1111209ただの霊である私がここまで力を持っているのも、\\n\
        この地に根付いているもののおかげ。\\n\\n\
        h1111205次も、霧の濃い日にいらっしゃい。\\n\
        そのほうが、身体が楽なの。\
        "),

        "\
        h1111209あなたたちが歩いている姿を、いつも窓から見ているの。\\n\
        h1111204いつも何かをして、どこかへ向かっている。\\n\
        h1111207羨ましいわ。h1111207私は\\_a[Fastened,どういうこと？]見ていることしかできない\\_aから、なおさら。\
        ".to_string(),

        "\
        h1111201あなたの趣味……ゴスファッション、と言うんだったかしら。\\n\
        h1111207素敵ね。よく似合っているわ。\\n\
        h1111101……初めて言われた？……h1111204あなたの周りは見る目がないのね。\\n\
        ……h1111206色眼鏡で見られたとして、気にする必要はないわ。\\n\
        自分に嘘をつかないことが、いちばん大切だから。\
        ".to_string(),

        format!("\
        h1111201ある程度外見を変えることもできるの。h1111207……こんなふうに。\\n\\n\
        h1000000{}\\n\
        \\0髪も、身長も、年齢すら、私たちには関係ないの。\\n\
        h1111209\\1瞬きしたとき、彼女はまた元の姿に戻っていた。\
        h1111204あまり大きく変化すると自己認識が揺らいでしまうから、基本的には最も自分らしい姿をすることが多いわ。\\n\
        h1111207こういう戯れもたまにはいいでしょう？\
        ",
        user_talk("……髪が伸びてる！","彼女の姿が揺らいだかと思うと、その姿は一瞬で変わった。", true)),

        "\
        h1111209寄る辺ない幽霊はいつか消える。\
        それが10年後なのか、100年後なのか、それとも明日なのか。\\n\
        それは分からないけれど、その日は必ず来る。\\n\
        h1111205だからあなた、いつか消える幽霊にかまけるなんて、時間の無駄なのよ。\\n\
        ……h1111309いつ来なくなっても、私は気にしないわ。\
        ".to_string(),

        "\
        h1111109\\1床を、極彩色の虫が這っている。\\n\
        h1111106……h1111105。\\n\\n\
        \\1ハイネはそれを一瞥して、すぐに視線を戻した。\\n\
        気にならないのだろうか。\\n\\n\
        ……そういえば、本で読んだことがある。\\n\
        フィクションの怪物の多くが不潔な外見をしているように、\\n\
        人ならざる者たちは毒虫や汚物に対する嫌悪感をほとんど持たないらしい。\\n\
        あれは小説の設定だったが……彼女もそうなのだろうか。\\n\\n\
        h1111109……h1111201ねえ。\\n\
        h1111204あなた、虫を捕まえるのが得意だったりしないかしら。\\n\
        \\1……。\
        ".to_string(),

        "\
        h1111204あなた、口数が少ないのね。\\n\
        h1111201いえ、いいのよ。h1111205そう、どちらでもいい。\\n\
        ".to_string(),

        "\
        h1111206私たちは、自我という色眼鏡を通してしか世界を観測できない。\\n\
        h1111204あなたは目の前にいるのに、\\n\
        あなたが見る世界を私が知ることはできないの。\\n\
        h1112209それって、この上なく残酷なことだわ。\
        ".to_string(),
      ],
    };
    strings
      .iter()
      .map(|s| Talk::new(Some(*self), s.clone()))
      .collect()
  }
}

pub fn random_talks_analysis() -> String {
  format!(
    "\\_q\
    TALKS_SELF_INTRODUCE: {}\\n\
    TALKS_LORE: {}\\n\
    TALKS_PAST: {}\\n\
    TALKS_ABSTRACT: {}\\n\
    TALKS_WITH_YOU: {}\\n\
    ---\\n\
    TOTAL: {}\
    ",
    TalkType::SelfIntroduce.talks().len(),
    TalkType::Lore.talks().len(),
    TalkType::Past.talks().len(),
    TalkType::Abstract.talks().len(),
    TalkType::WithYou.talks().len(),
    TalkType::all_talks().len(),
  )
}

#[allow(dead_code)]
static BOTSU: Lazy<Vec<String>> = Lazy::new(|| {
  vec![
    "\
    h1111205……\\n\\n\
    \\1本を読む彼女をスケッチしている。\\n\\n\
    彼女は写真に写らないという。原理は不明。彼女の存在が科学的に解明されるときは来るのだろうか。まあ、それ自体にあまり興味はないけれど……。\\n\
    ……ともかく、彼女の姿を何かに残しておきたくなって描きはじめたのだ。\\n\
    \\0……h1111201あら、絵を描いているの？見せて。\\n\
    h1111202……これは、私？……h1111205ふうん。こんなふうに見えているのね。\\n\\n\
    h1111101…………h1111204いいえ、いいのよ。h1111205絵に描いてもらえるなんて、嬉しいわ。\
    ".to_string(),

    "\
    h1111205見慣れたはずの場所にいながら、いつもと違う道に迷いこんだことはある？\\n\
    \\n\
    h1111204もしそうなったら、「\\_a[Yomotsuhegui,ヨモツヘグイって？]ヨモツヘグイ\\_a」……\
    つまり、食べ物には常に注意しなさい。\\n\
    h1111205一度だけなら、は許されない。それがすべてを変えてしまうような落とし穴がこの世にはあるの。\
    ".to_string(),

    format!("\
    h1111205悲しい。ここに縛られていることが、ではない。私が見ることのできない世界のこと。\\n\
    ……h1111209何も、知ることができないの。老人の、幼子の、男の、女の、見る世界を、\\n\
    そのすべてを私がこの身で知ることはかなわない。h1111205決して、……h1111209決して。\\n\
    それが、悲しくて、悔しくて、気が狂いそうになる。\\n\
    h1122305だって、せっかくこの世に生まれたのに。こんなにも自由なのに。……この手で、何をすることもできるはずなのに！\\n\\n\\_w[1200]\
    {}\
    \\0……h1111205あなたに言っても詮無いことだわ。忘れてちょうだい。\
    ",
    user_talk("ハイネ……。","思わず、声が漏れる。", false)),
  ]
});

pub fn version(_req: &Request) -> Response {
  new_response_with_value(String::from(env!("CARGO_PKG_VERSION")), false)
}

pub fn on_ai_talk(_req: &Request) -> Response {
  let vars = get_global_vars();

  // 没入度を上げる
  let current_immersive_degrees = vars.volatility.immersive_degrees();
  vars
    .volatility
    .set_immersive_degrees(std::cmp::min(current_immersive_degrees + 2, 100));

  vars
    .volatility
    .set_last_random_talk_time(vars.volatility.ghost_up_time());
  debug!(
    "{} < {}: {}",
    vars.volatility.idle_seconds(),
    vars.volatility.idle_threshold(),
    vars.volatility.idle_seconds() < vars.volatility.idle_threshold(),
  );

  let rnd = rand::thread_rng().gen_range(0..=100);
  let immersive: &str;
  let talks = if rnd < vars.volatility.immersive_degrees() {
    // 没入度が高いときのトーク
    immersive = "高";
    let mut v = vec![];
    v.extend(TalkType::Abstract.talks());
    v.extend(TalkType::Past.talks());
    v
  } else {
    // 没入度が低いときのトーク
    immersive = "低";
    let mut v = vec![];
    v.extend(TalkType::SelfIntroduce.talks());
    v.extend(TalkType::Lore.talks());
    v.extend(TalkType::WithYou.talks());
    v
  };

  let choosed_talk = choose_one(
    &talks,
    vars.volatility.idle_seconds() < vars.volatility.idle_threshold(),
  )
  .unwrap();

  let mut res = new_response_with_value(choosed_talk.text, true);
  res.headers.insert_by_header_name(
    HeaderName::from("Marker"),
    format!(
      "{} (没入度{})",
      choosed_talk.talk_type.unwrap(),
      immersive,
    ),
  );
  res
}

pub fn on_anchor_select_ex(req: &Request) -> Response {
  let refs = get_references(req);
  let id = refs[1].to_string();
  let user_dialog = refs.get(2).unwrap_or(&"").to_string();

  let mut m = String::from("\\C");
  m += "\\0\\n\\_q─\\w1──\\w1───\\w1───────\\w1────\\w1──\\w1──\\w1─\\w1─\\n\\_w[750]\\_q";
  if !user_dialog.is_empty() {
    m += &format!("\\1『{}』\\_w[500]", user_dialog);
  }
  match id.as_str() {
    "Fastened" => {
      m += "\
      h1111205文字通りの意味よ。\\n\
      私はこの街から出られない。物理的にね。\\n\
      h1111209私の身体はここに縛られている。\\n\
      h1111205きっと、それは消滅する瞬間まで変わらないでしょう。\\n\
      ";
    }
    _ => return new_response_nocontent(),
  }
  new_response_with_value(m, true)
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
    vars.volatility.set_idle_threshold(2);

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
