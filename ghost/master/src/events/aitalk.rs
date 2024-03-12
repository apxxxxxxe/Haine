use crate::events::common::*;
use crate::roulette::RouletteCell;
use crate::variables::{get_global_vars, EventFlag, IDLE_THRESHOLD};
use core::fmt::{Display, Formatter};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use shiorust::message::{parts::HeaderName, Request, Response};
use std::collections::HashSet;

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

impl RouletteCell for String {
  fn key(&self) -> String {
    self.clone()
  }
}

impl Display for Talk {
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    write!(f, "{}", self.text)
  }
}

pub fn random_talks(talk_type: TalkType) -> Vec<Talk> {
  let strings = match talk_type {
      TalkType::SelfIntroduce => vec![

        ("生前の記録", "\
        h1111206生前のこと、記録に残しているの。h1000000……h1111205このノートね。\\n\
        ……h1123305まあ、ずいぶん昔のことよ。実感はもうなくなってしまったの。\\n\
        h1123309今読んでも、他人の伝記を読んでいるようだわ。\\n\
        ……h1111205それにしてもこのノート、昔は紙がこんなに厚かったのね。\\n\
        h1113206今のはもう少し薄いと聞いているけれど、そうなのかしら。\
        ".to_string()),

        ("外出できない理由", "\
        h1111104外出したいかって？h1121304ええ、それはもう。\\n\
        ……h1123309霊力の強い霊は、特定の場所に縛られる傾向にあるの。\\n\
        h1113205私もそう。結び付きが強すぎて、この家から離れられないのよ。\\n\
        h1123209……まあ、外に出たら、今のファッションについていける自信がないわ。\\n\
        ……h1123204ジーンズはまだ流行ってる？\
        ".to_string()),

        ("恋愛観", "\
        h1111205幽霊は生前の想い……好みや恨みに執着するの。\\n\
        h1111209想い人がいればその人に、恨みがあればその相手に。\\n\
        h1111203逆に、死後新たな執着が生まれることはほとんどないわ。\\n\
        だから幽霊同士、h1111206ましてや人と幽霊の間に恋愛が生まれることは皆無といっていいでしょう。\\n\
        h1111304……なに、その顔は。h1111309あいにく、私は生きていた頃から恋愛とは無縁よ。\\n\
        ".to_string()),

        ("霊力と可視性", "\
        h1111206\\1ポットがひとりでに浮き、空になっていたカップに飲み物が注がれる。\\n\
        \\0……h1111204私が見えて彼らが見えないのは、霊としての力量の違いよ。\\n\
        h1111206私たちは霊力と呼んでいるのだけど。\\n\
        物理的な質量を持つほどに強い霊力があれば、\\n\
        あなたのように霊感のない人間の目にも見えるの。\
        ".to_string()),

        ("霊力の多寡", "\
        h1111204霊力の多寡は……年月や才能、特別な契約の有無などで変わるけれど、\\n\
        最も大きな要因は環境──つまり、その地との関わりの深さによるの。\\n\
        h1111309私のように生家に根付いた霊はいわずもがな。\\n\
        h1111205……まあ、強いからといって良いことばかりでもないの。\\n\
        h1111203霊にも社会があってね。h1111506ノブレス・オブリージュというわけ。\
        ".to_string()),

        ("低級霊との契約", "\
        h1111206\\1ポットがひとりでに浮き、空になっていたカップに飲み物が注がれる。\\n\
        h1111206私の元へ集うのは弱い人たち。\\n\
        自分だけでは溶けゆく自我を押し留められず、さりとてそれを受け入れることもできない霊。\\n\
        h1111209役割を与えてあげるの。一種の契約ね。\\n\
        h1111205使命に縛られはするけれど、消滅するよりはよしと彼らは決断したの。\
        ".to_string()),

        ("カンテルベリオという土壌", "\
        h1111203カンテルベリオには霊……正確には、\\n\
        死の意識が集まりやすい土壌があるの。\\n\
        ……h1111209あなたがここに来たのも偶然ではないのかもしれないわね。\
        ".to_string()),

        ("生家、そして主", "\
        h1111209ここは私の生家なの。実際は別荘なのだけど。\\n\
        h1111205そして、ここで死んだ。\\n\
        \\n\
        h1111209だから私は、ここの主とならなければならなかったの。\\n\
        懇願されてね。h1111206断ろうにもここを離れることもできないし。\\n\
        ……h1121304静かに眠りたかったのに、h1121305貧乏くじを引いたものだわ。\
        ".to_string()),

        ("幽霊たちの役割", "\
        h1111203従者……と、私が呼ぶ幽霊たち。\\n\
        h1111209私の与えた役割を全うしてくれるものは多くいるわ。\\n\
        h1111205最も多いのは、自分の生前の経験を記録として私に提供してくれる者たち。\\n\
        h1111209一つとして同じものはない。読んでいて退屈しないわ。\\n\
        ……h1113204少し形は違えど、あなたもその一人ね。\\n\
        h1113207期待しているわ、{user_name}。\
        ".to_string()),

        ("幽霊たちの自由", "\
        h1111206私は彼らと直接話すことはできないの。\\n\
        霊力の差があまりにも大きい場合、\\n\
        h1111209会話や接触を少し行うだけで、弱い方の霊は力を奪われる。\\n\
        間接的に指示を出すことはできるけれど、\\n\
        h1111205何か伝えるなら仲立ちをする者が必要なのよ。\
        ".to_string()),

      ],
      TalkType::Lore => vec![

        ("死者の埋葬", "\
        h1111206古代ギリシャでは、刑死の際は毒薬に阿片を混ぜたものを飲ませていたの。\\n\
        h1113209それは死の苦しみを和らげるためだったのかもしれないけれど、\\n\
        それ以上に、死を恐れる人々を抑えるためだったのかもしれないわね。\\n\
        h1113205罰ではあれど、必要以上に苦しませることはない、と。\
        ".to_string()),

        ("死後の存在", "\
        h1111209幽霊、霊体、死後の存在。人類の科学は、そういったものにまだ答えを出していない。\\n\
        h1111205存在する、しないの議論は、h1112205まあ、私たちには必要ないわね。\\n\
        h1111209……いつかその時が来るのかしら。霊体を観測し、干渉し……あるいは、消滅させる方法。\\n\
        h1111205ふふ。私、期待しているの。\
        ".to_string()),

        ("黒死病", "\
        h1111209黒死病が蔓延していたとき、問題になっていたのがいわゆる「早すぎた埋葬」。\\n\
        h1111205ある技師は生き埋めにされる恐怖から逃れるため、\\n\
        埋葬者が生きていることを棺の内側から知らせる仕組みがついた棺を開発したの。\\n\
        h1111204彼、デモンストレーションのために自ら生き埋めになってみせたそうよ。\\n\
        h1212209自分で出られない状態で、冷たい土の下へ。\\n\
        ……h1212205どんな心地がしたのかしらね。\
        ".to_string()),
      ],
      TalkType::Past => vec![
        ("惨めな人生", "\
        h1111205みじめな人生の上に正気でいるには、\\n日々は長すぎたの。\
        ".to_string()),

        ("行き場のない苦しみ", "\
        h1112202誰が悪い？いいえ、誰も悪くない。\\n\
        打ち明けたところで、的はずれな罪悪感を生むだけ。\\n\
        h1112205だからといって、他人に責をなすりつけるほど鈍くあることもできなかった。\\n\
        h1112209この気持ちには、どこにも行き場がなかったの。\
        ".to_string()),

        ("後悔", "\
        h1111205私に救いは訪れなかった。\\n\
        想いは、今もずっと、私の中にある。\\n\
        あなたが、私を救える人だったら良かh1111101……。\\n\
        ……h1111109。いえ、死んだ後で報われようだなんて。\\n\
        h1121205……h1121305悪いわね。\
        ".to_string()),

        ("死の瞬間", "\
        h1111205死ぬ瞬間、後悔はなかった。\\n\\n\
        もう一度同じ人生を生きることができたとしても、私は同じことをすると断言できるわ。\\n\
        ……h1121209ただ、遺書くらいは書いたほうがよかったかしら。\
        ".to_string()),

        ("助けは遂げられず", "\
        h1111205助けようとしてくれた人は沢山いたけれど、\\n\
        h1121209それが遂げられることはついぞなかったわ。\
        ".to_string()),

        ("死なない理由", "\
        h1111209生きていて良かったと思えることは数えきれないほどあったわ。\\n\
        h1111205でも、死なない理由は一つも見つからなかった。\
        ".to_string()),

        ("守ってくれた人", "\
        h1111105あの人は、私を守ってくれた。\\n\
        でも、私を救えはしなかった。\\n\
        理解と共感は、違う。h1112105……違うのよ。\
        ".to_string()),

        ("ふつうになりたかった", "\
        h1122209ふつうになりたかった。\\n\
        ……h1122205でも、ふつうだったら、もう私じゃないとも思う。\\n\
        それは私の顔をした別のだれかで、\\n\
        私は私の性質と不可分で、\\n\
        今ここにいる私は、私以外でいられない。\\n\
        h1122209だから、私として生きることができなかった私は、もうどこにもいられない。\
        ".to_string()),

        ("人と本", "\
        h1111205昔から、人と本の違いがわからなかったの。\\n\
        h1121204もちろん、区別がつかないという意味ではなくて。\\n\
        ……h1111209人に期待するものがそれだけしか無かったの。\
        ".to_string()),

        ("今度こそ無へ", "\
        h1111205死にぞこなったものだから、次の手段を求めている。\\n\
        ……h1112305今度こそ、終わらせたいの。\\n\
        今度こそ、無へ。\
        ".to_string()),

        ("魂は消える", "\
        h1111109未練もなく、しかし現世に留まっている魂。\\n\
        h1111105あるべきでないものはやがて消滅する。\\n\
        h1111206多少の不純物が含まれようと、そのルールは変わらない。\\n\
        h1111205私は、それを待ち望んでいるの。\
        ".to_string()),

        ("月の満ち欠けのように", "\
        h1111205月の満ち欠けのように、私の心は移り変わる。\\n\
        h1111209理解を得ることは難しかったわ。\\n\
        そんな仕打ちも、納得はできる。自分ですら不可解なのだから。\\n\
        ……h1121306少しでも自分で律することができたなら、\\n\
        こんなに苦しむことはなかっただろうに。\
        ".to_string()),

      ],
      TalkType::Abstract => vec![

        ("感動と倦み", "\
        h1111205ある本を最初に読んだときの感動と、何度も読み返して全て見知ったゆえの倦み。\\n\
        どちらがその本の真の印象かしら。\\n\\n\
        h1111209私はどちらも正しいと思うの。\\n\
        ……h1111504卑怯だと思った？\\n\
        h1111209印象なんてその時々で変わるもので、h1111205一つに定まることなんて稀だもの。\\n\\n\
        まして、自分の中に秘めるものならなおさら。\\n\
        h1111506どちらか一方だけだなんて、勿体ないわ。\
        ".to_string()),

        ("納得のための因果", "\
        h1111209因果が巡ってきた。\\n\
        過去が現在を刈り取りに来た。\\n\
        私は報いを受けたのだ。\\n\\n\
        ……h1111205それが、自分を納得させるための妄想だったとしたら？\
        ".to_string()),

        ("怖いものを見るということ", "\
        h1111201怖いものだからこそ、見つめなければ戦えない。\\n\
        ……h1121205そんなもの、戦える人のためだけの論理だわ。\
        ".to_string()),

        ("停滞を終わらせるために", "\
        h1111205危険と隣り合わせだからこそ、世界は美しいの。\\n\
        身を損なう心配がなくなっては、美しさが心を打つこともない。\\n\
        h1121205ただただ平坦な、揺らがぬ水面があるだけ。\\n\
        h1121209それはやがて、淀み、腐る。\\n\
        h1111205願わくば、せめて終わりがありますように。\
        ".to_string()),

        ("停滞の破壊", "\
        h1111105人生に変化は付きもの……けれどh1111109停滞はそれ以上。\\n\
        一度立ち止まってしまうと、空気は一瞬で淀んで、身動きがとれなくなってしまう。\\n\
        それは倦怠とも違う、鈍い痛み。\\n\
        h1111201あなた。h1111205もしそうなったときは、多少無理にでも変化を取り入れるほうがいいわ。\\n\
        ……h1111209たとえなにかを破壊することになるとしても、何も出来ないよりはずっとましよ。\
        ".to_string()),

        ("極限の変化としての死", "\
        h1111205死の瞬間の、極限に振れた変化。\\n\
        命が命でなくなり、身体が陳腐な肉の塊になる、その一瞬が愛しくてたまらない。\\n\
        どうしようもなく、愛しいの。\\n\\n\
        h1111209……幻滅するでしょう。あなたの目には、贔屓目にも綺麗には写らない。\\n\
        h1111309……でもね。倫理は私を救わない。\\n\
        あなたの心を私が握る必要も、ない。\\n\
        h1111304それを諦められるくらい、私は、これを諦められないのだから。\
        ".to_string()),

        ("死の向こう側", "\
        h1112109どうか、死の向こう側がありませんように。\
        ".to_string()),

        ("沈んでいく", "\
        h1111105沈んでいく。\\n\
        手がどうしても動かなくて、目の前の希望を掴めない。\\n\
        身体が重い。浅い呼吸のなかで、沈んでいく自分の身体を感じていることしかできない。\\n\
        私は、私を救うことを諦めているみたい。\\n\
        h1111109どうして。\\n\
        h1121205どうして、こうなってしまったのだろう。\
        ".to_string()),

        ("人を解体したい", "\
        h1111109人を解体したいと、思うことがあるの。\\n\
        何が人を人たらしめているのか、どこまで分解すれば人は人でなくなるのか。\\n\
        h1111105人という恐ろしく不可解な物の、どこにその根源があるのか。\\n\
        それを知るには、他に方法が思いつかないの。\
        ".to_string()),

        ("わがままな祈り", "\
        h1111209がんばっているってこと、\\n\
        理解できなくても見ていてほしかったの。\\n\
        ……h1121205わがままかしら。\
        ".to_string()),

        ("生者にとっての慰め", "\
        h1111209枯れ木に水をあげましょう。\\n\
        もはや花は見れずとも、それが慰めとなるのなら。\\n\
        \\n\
        h1111205それは誰にとって？\\n\
        h1111206もちろん、死を悼む者にとっての慰めよ。\\n\
        むくろに心はないもの。\
        ".to_string()),

        ("不可逆な崩壊", "\
        h1111209燃え殻がひとりでに崩れるように、心が静かに割れて戻らなくなった。\\n\
        h1111205だから、諦めたの。\
        ".to_string()),

        ("中途半端な助け", "\
        h1111209中途半端な助けは何もしないより残酷だわ。\\n\
        h1111205希望を持たせておいて、それを奪うのだもの。\
        ".to_string()),

        ("レンズの歪み", "\
        h1111205観察と模倣を続ければ、完全に近づけると思っていた。\\n\
        想定外だったのは、レンズが歪んでいたことと、それを取り替える方法がなかったこと。\\n\
        h1121309そうなればすべて台無し。h1121304諦めるしかなかったわ。\
        ".to_string()),

        ("先の見えない苦しみ", "\
        h1111205一寸先は暗く、扉は閉ざされている。\\n\
        不明な道のりを諸手で探るよりも、\\n\
        h1112305目先の手首を切り裂くほうが遥かに明瞭なのだ！\\n\
        ……h1111209なんてね。\
        ".to_string()),

        ("唯一の視点", "\
        h1111206私たちは、自我という色眼鏡を通してしか世界を観測できない。\\n\
        h1111204あなたは目の前にいるのに、\\n\
        あなたが見る世界を私が知ることはできないの。\\n\
        h1112209それって、この上なく残酷なことだわ。\
        ".to_string()),

        ("一つの個としての限界", "\
        h1111203世界が複雑で曖昧すぎるから、\\n\
        私たちは認識したものを理解できる形に歪めてしまう。\\n\
        h1111209既存の分類に当て嵌めて、安心を優先するの。\\n\
        曇る視界と引き換えにね。\\n\
        ……h1111204あなたには、私はどう見えているのかしら？\\n\
        ".to_string()),

        ("自己同一性の仮定", "\
        h1111205環境と経験の総体こそが自己であるような気がするの。\\n\
        仮にそうだとすれば、自己同一性すら偶然の産物ということになる。\\n\
        h1111209執着しているのが馬鹿馬鹿しく思えてくるわ。\\n\
        h1111205仮にそうでなければ。\\n\
        ……自己は最初から決定されている？\\n\
        h1111209それこそ、ね。\\n\
        ".to_string()),

        ("自分の理解者は自分だけ", "\
        h1111209「なぜみんな私をわかってくれないの？」と誰もが思う。\\n\
        h1111205答えは簡単。他人があなたではなく、あなたが他人でないからよ。\\n\
        畢竟、あなた以外にあなたを理解できるひとはいないの。\
        ".to_string()),

      ],
      TalkType::WithYou => vec![

        ("写真には写らない", "\
        h1111209今は手軽に写真が撮れていいわね。\\n\
        h1111205印象的な光景を、いつでも手元に残しておける。\\n\
        ……h1111201あら、私？h1121204光栄だけれど、残念ながら写真には写らないわ。\
        ".to_string()),

        ("不安を書き出す", "\
        h1111204……なにか不安なの？\\n\
        それとも、何が不安かもわからない？\\n\
        h1111209……紙に書き出してみるのはどうかしら。\\n\
        h1111205いくらか整理がつくかもしれない。\
        ".to_string()),

        ("茶菓子は用意してね", "\
        h1111209茶葉や茶菓子はあなたが持ってきてね。\\n\
        h1111206ここにある分は限られているし、私一人では補充する手段がないから。\\n\
        h1111209あなたが食べられるようなものは、私には用意できないの。\
        ".to_string()),

        ("霧の力", "\
        h1111206霧が、濃いでしょう。\\n\
        ただの霧ではないの。乾いた霧よ。\\n\
        むしろ、性質としては私たちに近い。\\n\
        h1111209ただの霊である私がここまで力を持っているのも、\\n\
        この地に根付いているもののおかげ。\\n\\n\
        h1111205次も、霧の濃い日にいらっしゃい。\\n\
        そのほうが、身体が楽なの。\
        ".to_string()),

        ("見ていることしかできない", "\
        h1111209あなたたちが歩いている姿を、いつも窓から見ているの。\\n\
        h1111204いつも何かをして、どこかへ向かっている。\\n\
        h1111207羨ましいわ。h1111207私は\\_a[Fastened,どういうこと？]見ていることしかできない\\_aから、なおさら。\
        ".to_string()),

        ("あなたのゴスファッション", "\
        h1111201あなたの趣味……ゴスファッション、と言うんだったかしら。\\n\
        h1111207素敵ね。よく似合っているわ。\\n\
        h1111101……初めて言われた？\\n\
        h1111204あなたの周りの人たちは見る目がないのね。\\n\
        ……h1111206色眼鏡で見られたとして、気にする必要はないわ。\\n\
        自分に嘘をつかないことがいちばん大切だから。\
        ".to_string()),

        ("あなたの価値", "\
        h1111101何をすればいいかって？\\n\
        h1111204今しているように、ただ話し相手になればいいのよ。\\n\
        h1111206私には従者がいるけれど、\\n\
        彼らは私と話すようにはできていないから。\\n\
        h1111204あなたの価値は、その自由意志。\\n\
        h1111209ここは予想通りなものばかりで退屈なの。\
        ".to_string()),

        ("生前の食事事情", "\
        h1111204あなたは、ちゃんと食べているかしら？\\n\
        h1111209そう。いいことね。\\n\
        h1111104私？……h1111205生前は食が細かったわ。\\n\
        h1111209……というより、食そのものにあまり関心がなかったみたい。\\n\
        ……h1123309思えば、家政婦には随分と世話をかけたわね。\
        ".to_string()),

        ("スケッチ", "\
        h1111205……\\n\\n\
        \\1本を読む彼女をスケッチしている。\\n\\n\
        彼女は写真に写らないという。\\n\
        それを聞いてから、彼女の姿を何かに残しておきたくなって描きはじめたのだ。\\n\
        \\0……h1111201あら、絵を描いているの？見せて。\\n\
        h1111202……これは、私？……h1111205ふうん。こんなふうに見えているのね。\\n\\n\
        h1111101…………h1111204いいえ、いいのよ。h1111205絵に描いてもらえるなんて、願ってもないことだわ。\
        ".to_string()),

      ],
    };
  strings
    .iter()
    .map(|s| Talk::new(Some(talk_type), s.0, s.1.to_string()))
    .collect()
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
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
  pub fn from_u32(n: u32) -> Self {
    match n {
      0 => Self::SelfIntroduce,
      1 => Self::Lore,
      2 => Self::Past,
      3 => Self::Abstract,
      4 => Self::WithYou,
      _ => unreachable!(),
    }
  }

  pub fn to_u32(self) -> u32 {
    match self {
      Self::SelfIntroduce => 0,
      Self::Lore => 1,
      Self::Past => 2,
      Self::Abstract => 3,
      Self::WithYou => 4,
    }
  }

  pub fn all() -> Vec<Self> {
    vec![
      Self::SelfIntroduce,
      Self::Lore,
      Self::Past,
      Self::Abstract,
      Self::WithYou,
    ]
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
static BOTSU: Lazy<Vec<String>> = Lazy::new(|| {
  vec![

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
    h1111205食事の時間になっても部屋から出てこない家族。\\n\
    扉を開けてみると、彼女は足の一部を残して焼死していた。\\n\
    ……h1111206人体発火現象は、世界中で見られるわ。\\n\
    h1111209多くの場合火気はなく、発火の原理は不明。\\n\
    h1111206さらに、いくらかの延焼はあれど周囲には被害が及ばない。\\n\
    h1111209まったく不思議な話よね。h1111204あなたはどう考える？\
    ".to_string(),

    "\
    h1111209寄る辺ない幽霊はいつか消える。\
    それが10年後なのか、100年後なのか、それとも明日なのか。\\n\
    それは分からないけれど、その日は必ず来る。\\n\
    h1111205だからあなた、いつか消える幽霊にかまけるなんて、時間の無駄なのよ。\\n\
    ……h1111309いつ来なくなっても、私は気にしないわ。\
    ".to_string(),

    "\
    h1111204あなた、口数が少ないのね。\\n\
    h1111201いえ、いいのよ。h1111205そう、どちらでもいい。\\n\
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
    h1111109……h1111201あなた。\\n\
    h1111204虫を捕まえるのが得意だったりしないかしら。\\n\
    \\1……。\
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

pub fn on_ai_talk(_req: &Request) -> Response {
  let vars = get_global_vars();
  let if_consume_talk_bias = vars.volatility.idle_seconds() < IDLE_THRESHOLD;

  vars
    .volatility
    .set_last_random_talk_time(vars.volatility.ghost_up_time());

  // 没入度を上げる
  let immersive_degrees = std::cmp::min(vars.volatility.immersive_degrees() + IMMERSIVE_RATE, 100);

  if immersive_degrees >= 100 {
    let (previous_talking_place, current_talking_place) = match vars.volatility.talking_place() {
      TalkingPlace::LivingRoom => (TalkingPlace::LivingRoom, TalkingPlace::Library),
      TalkingPlace::Library => (TalkingPlace::Library, TalkingPlace::LivingRoom),
    };

    let messages: Vec<String> = {
      let parts: Vec<Vec<String>> = if !vars.flags().get(EventFlag::FirstPlaceChange) {
        vars.flags_mut().set(EventFlag::FirstPlaceChange, true);
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
            h1111204……あなたは、本を読むのは好き？\\n\
            h1111306私は好きよ。巨人の肩に乗って遠くが見える。\\n\
            h1111305あるいは、ここではないどこかへ、遠くへ行ける。\
            h1111204あなたも自由に読みなさい。h1111309ここはそういう場所よ。\
            "
            .to_string()],
            TalkingPlace::LivingRoom => vec!["これが表示されることはないはず".to_string()],
          },
        ]
      } else {
        vec![
          vec![format!(
            "\\0\\b[{}]h1000000……。\\1また、ハイネが姿を消してしまった。\\n\
            \\0\\b[{}]\\1前回のように{}を探しに行くと、彼女はそこにいた。\\n\
          ",
            previous_talking_place.balloon_surface(),
            current_talking_place.balloon_surface(),
            current_talking_place
          )],
          match current_talking_place {
            TalkingPlace::Library => vec!["\
            h1111209さて、仕切り直しましょう。\\n\
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
      TranslateOption::simple_translate(),
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

  let text = format!(
    "\\b[{}]{}",
    vars.volatility.talking_place().balloon_surface(),
    choosed_talk.text
  );

  let mut res = new_response_with_value(text, TranslateOption::with_shadow_completion());
  res.headers.insert_by_header_name(
    HeaderName::from("Marker"),
    format!("{}: {}", choosed_talk.talk_type.unwrap(), choosed_talk.id,),
  );
  res
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
  m += "\\0\\n\\_q─\\w1──\\w1───\\w1─────\\w1────\\w1──\\w1──\\w1─\\w1─\\n\\_w[750]\\_q";
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
