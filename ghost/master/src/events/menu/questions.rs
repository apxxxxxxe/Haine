use crate::check_error;
use crate::events::talk::BranchTalk;
use crate::system::error::ShioriError;
use crate::system::response::*;
use shiorust::message::{Request, Response};

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
      Question::WHAT_DO_YOU_DO_WHEN_YOU_ARE_ALONE => "ひとりのときは何をして過ごしてる？".to_string(),
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
      _ => {
        error!("Unknown question theme: {:?}", self);
        String::new()
      }
    }
  }

  fn to_script(self) -> String {
    format!("\\![*]\\__q[OnTalkAnswer,{}]{}\\__q", self.0, self.theme())
  }

  /// 回答トークの定義。本文と掘り下げ選択肢（→その後のトーク）をここに一体で書く。
  /// 掘り下げを足すときは BranchTalk::leaf を BranchTalk::node に替えて choices を並べる。
  /// 「回答後に質問一覧へ戻る」動作は with_menu_return が終端ノードへ一括付与するので書かない。
  pub(crate) fn branch_talk(&self) -> BranchTalk {
    match *self {
      Question::FEELING_OF_DEATH => BranchTalk::leaf(
        "\
        h1111104\\1『幽霊ということは、一度死んだんだよね？\\n\
        どんな感じだった？何か思うことはある？』\
        h1111204いいえ、何も。\\n\
        h1111205私の求める変化はそこには無いし、\\n\
        何より私はまだ死ねていない。\\n\
        自我を手放してこその死でしょう？\\n\
        h1111210だから、これからよ。\\n\
        ",
      ),
      Question::FATIGUE_OF_LIFE => BranchTalk::leaf(
        "\
        \\1『生きるのは苦しい。どうしていいかわからない』\\n\
        h1111205そう、そうね。\\n\
        …………h1111204悪いけれど、\\n\
        私はその答えを持っていない。\\n\
        \\n\
        h1111204あなたが満足できるまで話を聞くわ。\\n\
        h1111210それから、どうするかを自分で決めなさい。\
        ",
      ),
      Question::HOW_TALL_ARE_YOU => BranchTalk::leaf(
        "\
        \\1『身長はどれくらい？』\\n\
        h1111204おおよそ175cmね。\\n\
        ……h1111210おおよそ、と言ったのは、\\n\
        生前の身長だから。\\n\
        h1111206今の身長は、測っても無駄なの。\\n\
        霊体は常に揺らめいていて、大\\n\
        きさが変動し続ける。\\n\
        h1111310……まあ、\\n\
        平均的にはそのくらいだと思ってちょうだい。\
        ",
      ),
      Question::HOW_WEIGHT_ARE_YOU => BranchTalk::leaf(
        "\
        \\1『体重は？』\\n\
        h1111201……霊体に重さはないわ。\\n\
        h1111204……知りたいのはそういうことではないって？\\n\
        h1111210まあ、そうでしょうね。\\n\
        h1111205……55kgだったかしら。もう定かではないけれど。\
        ",
      ),
      Question::HOW_MUCH_IS_YOUR_BWH => BranchTalk::leaf(
        "\
        \\1『スリーサイズを教えて』\\n\
        h1111601…………h1111201さっきから随分と果断ね。\\n\
        h1111204怒られるかもとか考えないのかしら。\\n\
        h1111205……79・56・81。\\n\
        ……h1111210知ってどうするのか知らないけれど。\
        ",
      ),
      Question::HOW_OLD_ARE_YOU => BranchTalk::leaf(
        "\
        \\1『何歳？』\\n\
        h1141604……h1111204女性に年齢を聞くなんて。\\n\
        ……h1111205死んだ時は26よ。\\n\
        死んでからは……h1111511教えてあげない。\\n\
        ",
      ),
      Question::HOW_TO_GET_TEALEAVES => BranchTalk::leaf(
        "\
        \\1『お茶はどこから手に入れているの？』\\n\
        h1111206行商人がいるのよ。私と同じ、実体を持つ霊。\\n\
        h1111210それでいて場所に囚われない、稀有な存在よ。\\n\
        それに定期的なお使いを頼んでいるの。\\n\
        良い茶葉を扱う店に、買い物を。\\n\
        \\n\
        h1111205勿論、対価も払わなければならない。\\n\
        それは自由に動ける代わりに、\\n\
        長い休眠を必要とするの。\\n\
        h1111210取引をする者たちはあれが無防備な間、\\n\
        身の安全を保障する契約なのよ。\
        ",
      ),
      Question::DO_SERVENTS_HAVE_NAMES => BranchTalk::leaf(
        "\
        \\1『従者たちに名前はあるの？』\\n\
        h1111210ええ、もちろん。\\n\
        でも、教えることはできないわ。\\n\
        \\n\
        h1111206霊にとって、\\n\
        自分が何者であるかは文字通り死活問題なの。\\n\
        肉の器がない分、\\n\
        簡単に存在が揺らいでしまうから。\\n\
        h1111210必要なときは偽名や通り名を名乗り、\\n\
        真の名前は契約する相手にしか明かさないのよ。\\n\
        \\n\
        ……h1111204「寂しい」って思った？……h1111211ふふ。\\n\
        h1111204あなたに教えた私の名前は偽名ではないわ。\\n\
        私は低級霊ではないから、多少は構わないの。\\n\
        h1111210生者の時間を奪うことへの、せめてもの礼儀よ。\
        ",
      ),
      Question::CALL_YOU_MASTER => BranchTalk::leaf(
        "\
        \\1『ご主人様』\\n\
        h1111101……h1111210ふふ、従者の仲間入りがしたいの？\\n\
        \\n\
        h1111304いいえ、あなたは客人よ。\\n\
        あなたにとって私は、ただのハイネ。\\n\
        \\n\
        h1111205なにかに身を委ねるのは簡単だけれどね。\\n\
        自分の手綱は自分で握るものよ。\\n\
        h1111210自分の意志でここにいる。\\n\
        そういうあなたで、いてちょうだい。\
        ",
      ),
      Question::WHAT_DO_YOU_DO_WHEN_YOU_ARE_ALONE => BranchTalk::leaf(
        "\
        \\1『ひとりのときは何をして過ごしてる？』\\n\
        h1111105……ひとりのときというと、仕事がないときね。\\n\
        h1111204大抵は書斎で本を読むか、\\n\
        私室でお茶を飲んで休むか、\\n\
        h1111206ああ、\\n\
        ここで従者たちの相手をすることもあるわね。\\n\
        彼らも仕事の息抜きを欲しているから、\\n\
        定期的にね。\\n\
        ……h1111105これはひとりのときではないか。\\n\
        h1111204まあ、好きなように過ごしているわ。\
        ",
      ),
      Question::CAN_I_STAY_TONIGHT => BranchTalk::leaf(
        "\
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
        h1111206あなたには、帰る場所と、続いていく日々がある。\\n\
        それを手放させるわけにはいかないもの。\\n\
        \\n\
        h1111204部屋にはあとで案内させるわ。\\n\
        h1111210今夜だけは、ゆっくりしてちょうだい。\
        ",
      ),
      Question::IS_THERE_A_PLACE_TO_VISIT => BranchTalk::leaf(
        "\
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
        ",
      ),
      Question::YOU_ARE_CUTE => BranchTalk::leaf(
        "\
        \\1『かわいい』\\n\
        h1111101……h1111204「可愛げがない」ではなくて？\\n\
        h1111210ふふ、わかっているわ。\\n\
        h1111205生前はそういう言葉をよく聞いたのよ。\\n\
        \\n\
        h1113210それにしても慣れないものね。\\n\
        h1113204参考までに、私のどこを\\n\
        「かわいい」と感じたのか聞いても良いかしら？\
        ",
      ),
      Question::YOU_ARE_BEAUTIFUL => BranchTalk::leaf(
        "\
        \\1『美人』\\n\
        h1111210……まあ、\\n\
        「かわいい」よりは言われ慣れているわね。\\n\
        h1111206着飾るのも、\\n\
        それを見せるのもあまり興味がないうえ、\\n\
        不必要に言い寄られることもあったものだから……\\n\
        h1111205あまり好ましいとは思わないのだけれど。\\n\
        h1111210……それでも、\\n\
        こうしてあなたを喜ばせられているのならば、\\n\
        それはきっと良いことなのでしょうね。\
        ",
      ),
      Question::I_AM_HUNGRY => BranchTalk::leaf(
        "\
        \\1『お腹が空いた』\\n\
        h1111104あら、もうそんな時間？\\n\
        h1111206……悪いけれど、ここには食事の用意はないの。\\n\
        h1111210私たちの食事はもっぱら娯楽として行うもので、\\n\
        それも茶菓子程度だから。\\n\
        \\n\
        だから、あなた自身で\\n\
        外から持って来てもらうことになるのだけど……\\n\
        h1111205…………その、\\n\
        できればここで食べて見せてほしいわ。\\n\
        h1111210……娯楽だけでない、\\n\
        生きる糧としての食事を眺めていたいの。\
        ",
      ),
      Question::CAN_I_TALK_TO_YOUR_SERVANTS => BranchTalk::leaf(
        "\
        \\1『従者たちと話してもいい？』\\n\
        h1111101それは……h1111104許容しかねるわ。\\n\
        h1111110彼らは外部からの影響に弱いの。\\n\
        h1113206挨拶程度なら構わないけれど、\\n\
        それ以上…ただの雑談だとしても、\\n\
        あなたから漏れる悪意なき偏見が、彼らの存在を\\n\
        不可逆に再定義してしまうかもしれない。\\n\
        h1113210生者から死者へ言葉を送る\\n\
        という行為がもつ意味は、\\n\
        あなたが想像するより遥かに重いの。\\n\
        この、互いの声が漏れ聞こえている状況が限界点。\\n\
        h1111204分かってちょうだいね。\
        ",
      ),
      Question::CALL_YOU_HAINE_1 => BranchTalk::leaf(
        "\
        \\1『ハイネ』\\n\
        h1111201ええ、何？\
        ",
      ),
      Question::CALL_YOU_HAINE_2 => BranchTalk::leaf(
        "\
        \\1『ハイネさん』\\n\
        h1111204……どうしたの、かしこまって。\\n\
        ",
      ),
      Question::CALL_YOU_HAINE_3 => BranchTalk::leaf(
        "\
        \\1『ハイネちゃん』\\n\
        h1111210うん……h1111201うん？\
        ",
      ),
      Question::WHEN_DO_YOU_WAKE_UP => BranchTalk::leaf(
        "\
        \\1『ふだん何時に寝起きしてる？』\\n\
        h1111204……質問に答えるなら、\\n\
        数日起きて、数日寝ているわ。\\n\
        h1111206私も彼らも、必要ならば何日でも活動し続ける。\\n\
        h1111210霊は基本的に眠りを必要としないから、\\n\
        起きていようと思えば\\n\
        いくらでも起きていられるのよ。\\n\
        \\n\
        h1121211とはいえ、それでは倦んでしまうから。\\n\
        h1111210必要に応じて…というか、\\n\
        起きている必要のないときは眠るの。\\n\
        \\n\
        h1111204……最近は、ある人間のおかげで\\n\
        起きている理由ができているけれど、ね。\\n\
        ",
      ),
      Question::WHY_IS_YOUR_BODY_COLD => BranchTalk::leaf(
        "\
        \\1『どうして体温が低い？』\\n\
        h1113205一言で言えば、血が通っていないからでしょうね。\\n\
        生物に体温があるのは、代謝……\\n\
        生存のための化学反応に温度が必要だから。\\n\
        h1113204血液を回し、酸素を通わせ、\\n\
        栄養をエネルギーに、そして熱に変える。\\n\
        h1113206一方で……\\n\
        私達がどのような原理で存在しているのかは\\n\
        未解明だけれど、\\n\
        温度を必要としない在り方なのでしょう。\\n\
        h1113204まあ、そのせいであなたには\\n\
        冷たい思いをさせてしまうけれど。\\n\
        h1113205……私の手をろうそくで炙れば、\\n\
        少しは温かくなるかしら？\\n\
        痛覚もさほどh1113101……h1121210冗談よ。そんな顔しないで。\
        ",
      ),
      Question::AM_I_BOTHERING_YOU => BranchTalk::leaf(
        "\
        \\1『迷惑じゃない？』\\n\
        h1111204……今更よ、そんなこと。\\n\
        h1111210ここは私の館。誰を置くかは、私が決めるの。\\n\
        h1111204出ていけと、一度でも言ったかしら。\\n\
        \\n\
        \\_w[1200]h1111210さあ、くだらないことを考えるのはおしまい。\\n\
        h1111204いつものように、\\n\
        あなたの話を聞かせてちょうだい。\
        ",
      ),
      Question::CALL_YOU_MOTHER => BranchTalk::leaf(
        "\
        \\1『お母さん』\\n\
        h1111101……h1111304聞き間違いかしら？\\n\
        h1111210先生のことを間違えてそう呼んでしまう\\n\
        という笑い話はよく聞くけれど。\\n\
        h1111204まさか、\\n\
        私を母親と間違えたわけではないでしょう？\\n\
        h1111210私にそんな素質などないものね。\\n\
        ",
      ),
      Question::CALL_YOU_SISTER => BranchTalk::leaf(
        "\
        \\1『お姉ちゃん』\\n\
        h1111204……h1111210きょうだいにしては、歳が離れているわね。\\n\
        そういう戯れの気分なのかしら？\\n\
        h1111204{user_name}ちゃん。\
        ",
      ),
      Question::WHAT_IS_YOUR_FAVORITE_SNACK => BranchTalk::leaf(
        "\
        \\1『好きなお茶菓子は何？』\\n\
        h1111205そうね……硬く焼き締めた菓子が好きなの。\\n\
        h1111206ビスケットやラスクのような、\\n\
        日持ちがして、片手でつまめるもの。\\n\
        \\n\
        h1111204生前の私は寝食を忘れて本を読むことが多くてね。\\n\
        h1111210食事の時間になっても席を立たず、\\n\
        家政婦を困らせていたのよ。\\n\
        \\n\
        h1111206見かねた彼女が、\\n\
        作業をしながらでも食べられるように\\n\
        硬く焼いた菓子を用意してくれたの。\\n\
        h1111210それが思いのほか美味しくて、\\n\
        読書の合間につまむのが習慣になったわ。\\n\
        \\n\
        h1111205……あれは私の体調を案じてくれた\\n\
        優しい工夫だったのでしょう。\\n\
        h1111206だからこそ、今でもあの味を懐かしく思うのよ。\
        ",
      ),
      Question::I_DREW_YOUR_PORTRAIT => BranchTalk::leaf(
        "\
        \\1『似顔絵を描いた』\\n\
        h1111101あら、私を？\\n\
        h1111204……見せてもらえるかしら？\\n\
        \\n\
        h1111210\\1手に持っていたスケッチブックを見せると、\\n\
        ハイネがゆっくりと手を伸ばして\\n\
        ページをめくった。\\n\
        \\0h1111205……h1111206なるほど。\\n\
        あなたの目には、私はこんな風に映っているのね。\\n\
        \\n\
        h1111210絵の技術もなかなかのものだけど、\\n\
        h1111204それよりも、\\n\
        あなたが私を見つめていた時間を思うと……\\n\
        h1111205少し照れくさいわね。h1111210ありがとう。\\n\
        \\n\
        h1111204大切にしてちょうだい。\\n\
        私にとっても、あなたにとっても、\\n\
        この瞬間の証になるものだから。\
        ",
      ),
      Question::LET_ME_PLAY => BranchTalk::leaf(
        "\
        \\1『遊びに行こう』\\n\
        h1111210……遊び。\\n\
        h1111204その「遊び」とは、どのようなものかしら。\\n\
        h1111206私にとっての娯楽といえば、\\n\
        読書や音楽鑑賞程度だけれど、\\n\
        h1111205生きている人間の「遊び」は、\\n\
        もっと活動的なものでしょう？\\n\
        \\n\
        h1111210……でも、面白そうね。\\n\
        h1111204あなたがどのような遊びを望むのか、\\n\
        聞かせてちょうだい。\\n\
        h1111204この館の中でできることなら、\\n\
        私も一緒に楽しませてもらうわ。\\n\
        h1111206……もしくは、見学させてもらうかしら。\\n\
        h1111210霊体では制約も多いものだから。\
        ",
      ),
      Question::CAN_I_PET_YOU => BranchTalk::leaf(
        "\
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
        ",
      ),
      _ => {
        error!("Unknown question talk: {:?}", self);
        BranchTalk::leaf("")
      }
    }
  }
}

/// 終端ノード（choices が空）の末尾に「\x で閉じて質問一覧に戻る」を付与する。
/// 掘り下げ表示中のノードに付けると \x がバルーンごと選択肢を消してしまうため、終端に限る。
fn with_menu_return(mut branch_talk: BranchTalk) -> BranchTalk {
  if branch_talk.choices.is_empty() {
    branch_talk.text.push_str("\\x\\![raise,OnTalk]");
  } else {
    branch_talk.choices = branch_talk
      .choices
      .into_iter()
      .map(|mut choice| {
        choice.next = with_menu_return(choice.next);
        choice
      })
      .collect();
  }
  branch_talk
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
  new_response_with_value_with_translate(
    with_menu_return(q.branch_talk()).render(),
    TranslateOption::with_shadow_completion(),
  )
}
