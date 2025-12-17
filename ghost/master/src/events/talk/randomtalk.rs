use std::collections::HashMap;

use crate::events::talk::{Talk, TalkType};
use crate::variables::{EventFlag, FLAGS};

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
          h1111110「別れがこんなに悲しいなら、最初から出会わなければよかった」\\n\
          h1111205……使い古された句だけど、私も、その時が来たらきっとそう感じると思う。\\n\
          過程がどうであれ、別れてしまえば残った傷は他の思い出を変質させてしまう。\\n\
          元通りの幸せな感情は決して戻らない。h1111210そう思うの。\
          ".to_string(),
          required_condition: None,
          callback: None,
        },

        // - 霊は姿を変えることはできない
        // - ハイネは人目を気にして外出を避けている
        RandomTalk {
          id: "姿は変えられない".to_string(),
          text: "\
          h1111306霊は不定形だけれど、自由に形を変えられるわけではないわ。\\n\
          h1111310魂の形は一つしかない。変えられるとしたら、自分が誰かもわからなくなってしまった者くらいよ。\\n\\n[half]\
          h1111206だから、私が昼に出歩くことはないわ。\\n\
          10年、20年経とうが姿の変わらない女。\\n\
          h1111310余計な面倒は避けるに越したことはないもの。\
          ".to_string(),
          required_condition: None,
          callback: None,
        },

        // - ハイネは科学に興味がある
        RandomTalk {
          id: "科学への興味".to_string(),
          text: "\
          h1111210生きていた頃、科学に興味を持っていたわ。\\n\
          h1111206物質の構造や、宇宙の謎、生命の起源。\\n\
          h1111205一見して無秩序で不確かなものたちが、\\n\
          じつに単純な秩序によって結びついているの。\\n\
          h1111210そのさまは、目が覚めるように美しい。\\n\\n[half]\
          h1111305今日はどんな新しい発見があるのかと、\\n\
          いまでも楽しみにしているのよ。\
          ".to_string(),
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
          ".to_string(),
          required_condition: None,
          callback: None,
        },

        // - ハイネは1世紀以上前に死んだ
        RandomTalk {
          id: "生前の記録".to_string(),
          text: "\
          h1111206生前のこと、記録に残しているの。\\n\
          ……h1123305まあ、まる1世紀も昔のことよ。\\n\
          自分のものだという実感はもうなくなってしまって、\\n\
          h1123310今読んでも他人の伝記を読んでいるようだわ。\\n\
          ".to_string(),
          required_condition: None,
          callback: None,
        },

        // - ハイネは恋愛とは無縁の人生だった
        RandomTalk {
          id: "恋愛観".to_string(),
          text: "\
            h1111205幽霊は生前の想い……好みや恨みに執着するの。\\n\
            h1111210想い人がいればその人に、恨みがあればその相手に。\\n\
            h1111203逆に、死後新たな執着が生まれることは\\n\
            ほとんどないわ。\\n\
            だから幽霊同士、h1111206ましてや人と幽霊の間に恋愛が生まれることは皆無といっていいでしょう。\\n\\n[half]\
            h1111304……なに、その顔は。h1111310あいにく、\\n\
            私は生きていた頃から恋愛とは無縁よ。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        // - ハイネは強い霊
        // - ハイネは霊たちに慕われている
        RandomTalk {
            id: "霊力の多寡".to_string(),
            text: "\
            h1111204霊力の多寡は年月や才能、特別な契約の有無などで変わるけれど、\\n\
            最も大きな要因は環境──つまり、その地との関わりの深さによるの。\\n\
            h1111310私のように生家に根付いた霊は言わずもがな。\\n\
            h1111205……まあ、強いからといって良いことばかりでもないわ。\\n\
            h1111203霊にも社会がある。h1111205\\_a[AnchorTalk,NoblesseOblige,義務ってどんなこと？]上位者の義務\\_aというものも。\\n\\n[half]\
            h1111210……はじめは億劫だと思っていたのだけどね。\\n\
            h1111206悪くないものよ。感謝され、慕われるというのは。\
            ".to_string(),
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
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        // - ここはハイネの生家
        RandomTalk {
          id: "生家の広さ".to_string(),
          text: "\
            h1111210ここは私の生家なの。実際は別荘なのだけど。\\n\
            h1111206知っての通り、従者がいなければ掃除が行き届かないほど広いの。\\n\
            h1111205……まあ、\\_a[AnchorTalk,LiveHome,別荘だけど長く住んでいたの？]勝手知ったる場所\\_aなのは不幸中の幸い、といえなくもないかしらね。\\n\
            h1111210くつろいで暮らすのにこれ以上の場所はないわ。\
            ".to_string(),
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
          ".to_string(),
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
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        // - ハイネは死後に記憶を整理した
        RandomTalk {
          id: "記憶の整理".to_string(),
          text: "\
            h1111206死んでから、生前の記憶を整理する時間がたくさんあったの。\\n\
            h1111210出来事を順序立てて、感情を分析して。\\n\
            h1111205客観的に見つめ直すと、\\n\
            当時は理解できなかった人の行動も\\n\
            今なら納得できることが多いわ。\\n\\n[half]\
            h1111210怒りや悲しみも、随分と色褪せてしまったけれど。\\n\
            h1111205それはそれで、悪くないものよ。\
            ".to_string(),
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
            h1121204だって、つまらないのだから仕方ないわよね？\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        // - ハイネはインターネットにあえて触れていない
        RandomTalk {
          id: "スマホとインターネット".to_string(),
          text: "\
            h1111205最近の携帯電話というのは随分便利なのね。\\n\
            写真はもはや当然で、インターネットすら常に使えるなんて。\\n\\n[half]\
            h1111101私？h1111206私は使わないことにしているの。\\n\
            情報が多すぎて、速すぎて……\\n\
            h1111205一度に沢山のことが押し寄せてくる環境は、\\n\
            どうも私の性質に合わないのよ。\\n\
            h1111210静かに、一つずつ考えていく方が好きなの。\\n\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

      ],

      TalkType::WithYou => vec![
        RandomTalk {
          id: "食事中の読書".to_string(),
          text: "\
            h1111105……。\\n\
            \\1ハイネは器用に菓子を切り分けながら、片手で本を開いている。\\n\
            行儀が良いとは言えない姿だ。\
            h1111204……何か言いたそうね。\\n\
            h1111310分かっているわ。お行儀が悪いって。\\n\\n[half]\
            h1111206昔は怒られたものよ。\\n\
            h1111205だから今は、一人のときだけ。\\n\
            h1111304……あなたの前では、\\n\
            一人のときと同じで良いでしょう？\
          ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "夜明けを待つ".to_string(),
          text: "\
            h1111206夜が長いと感じることはある？\\n\
            h1111205私はここで過ごすうち、\\n\
			何もしない時間を過ごすのが得意になったわ。\\n\
            h1111210星を眺めていると、そのうち空が白んでくる。\\n\
            h1111206夜と朝の境目。どちらでもない、曖昧な時間。\\n\\n[half]\
            h1111204一日の中で、一番好きなの。\
          ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "朝の髪".to_string(),
          text: "\
            h1111204……あなたの髪、寝癖がついているわ。\\n\
            h1000000直してあげましょうか。\\n\\n[half]\
            \\1ハイネの指が、乱れた髪をそっと整える。\\n\
            手つきは意外なほど丁寧で、慣れている。\\n\\n[half]\
            ……h1121310私も、起きたときはひどいものなのよ。\\n\
            h1111205ただでさえくせ毛なところに、寝癖も重なってね。\\n\
            h1111306何度も櫛を通して、やっとこの程度なの。\
          ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "早口".to_string(),
          text: "\
            h1113205……\\![set,balloonwait,0.7]それでねφ、\\_w[500]だからこそこの仕組みが成り立つのφ。\\_w[700]\\n\
            h1113204ここまでは良いかしらφ？\\_w[700]h1113206それで、次に重要なのが……\\![set,balloonwait,1]\\n\\n[half]\
            \\1矢継ぎ早の説明についていけずφ、h1111101\\1\\n曖昧に頷いてしまう。\
            \\0……h1121204ごめんなさい。少し早口だったわね。\\n\
            h1111205普通に話すと速いと言われるものだから\\n\
            ゆっくり話すよう心がけているのだけれど……\\n\
            h1113310熱が入ると、だめね。\
          ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "舌やけど".to_string(),
          text: "\
            h1113105……。\\n\
            \\1本を読みながら、ハイネはお茶を口に運ぶ。\\n\
            湯気が立っていることに気づいていない様子で、そのまま……\
            h1000000っ……。\\n\
            h1122205……熱かったわ。\\n\\n[half]\
            h1121205ぼんやりしているとよくやるの。\\n\
            h1121310何度も繰り返しているのだから、学べばいいのにね。\
          ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "取れたボタン".to_string(),
          text: "\
            h1113105……。\\n\
            \\1そういえば、ハイネの服が前着ていたものと違う。\\n\
            シルエットはほとんど同じだが、細部のデザインが異なっているのだ。\\n\\n[half]\
            『服、いつものと違う？』\
            h1111204あら、よく気づいたわね。\\n\
            h1111205そう、前のものはボタンが取れてしまったから、家のものに直させているの。\\n\\n[half]\
            h1113206彼女は針仕事がとても上手でね。\\n\
            h1111210私もできなくはないけれど、はるかに早くて正確。繕いものは任せているのよ。\
          ".to_string(),
          required_condition: Some(|| FLAGS.read().unwrap().check(&EventFlag::FirstClose)),
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
            塗り終えると、唇を小指で拭った。\
            h1113102\\n[half]ふむ……。h1113105保湿のためとはいえ、べたつくのは少し嫌ね。\\n\\n[half]\
            h1111204ありがとう、返すわ。h1111205\
          ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "くしゃみ".to_string(),
          text: "\
            h1000000……っくしゅん！\\n\\n[half]\
            ……h1112105ごめんなさい、ホコリか何かが……。h1111106空気を入れ替えましょう。\\n\\n[half]\
            \\1窓がかすかに開く。新鮮な空気がすっと入ってきた。\\n\\n[half]\
            『くしゃみするとき、目閉じないんだね』\
            h1111101……h1111102言われてみれば、そうね。\\n\
            h1121204ずっと見てたの？\\n\
            h1121206少し、気恥ずかしいわ。\
          ".to_string(),
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
            h1111206昔は気まぐれだと文句を言われたものけれど、\\n\
            そのとき食べたいのだから仕方ないわよね。\\n\\n[half]\
            h1111205ねえ、h1111310それ、いただいても良いかしら？\
          ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "訪問の意図".to_string(),
          text: "\
            h1111210想うという意味では、嫌悪も愛慕も変わらないと思うの。\\n\
            h1111204たとえ私を煙たく思っていたとしても、\\n\
            あなたがここに来てくれることは私にとって喜ばしいことだわ。\\n\\n[half]\
            h1111304……意地悪だったかしら？\\n\
            h1111210わかっているわよ、あなたが好んでここに来ているって。\
          ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk{
          id: "知って、祓う".to_string(),
          text: "\
              h1111205無知にこそ不安の種は宿るもの。\\n\
              h1111210訊いて、調べて、学びなさい。\\n\
              h1111204知ることで、自分を守るのよ。\\n\\n[half]\
              h1111204もし助けが必要ならば、遠慮なく。\\n\
              h1111206この館には、長い年月をかけて集めた知識が所蔵されているの。\\n\
              h1111210あなたの役に立つものもあるかもしれないわ。\
              ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk{
          id: "静寂が秘める言葉".to_string(),
          text: "\
              h1111205\\1（カチ、コチ、カチ、コチ……）\\n\
              振り子時計の音、そして私の息遣いだけが聞こえる。\\n\\n[half]\
              ハイネは静寂を恐れない。何時間でも、黙っている。\\n\\n[half]\
              h1111204……静かね。\\n\\n[half]\
              \\1！\
              h1111204話したければ、遠慮なく。\\n\
              これは私のための時間であり、\\n\
              h1111210あなたのための時間なのだから。\
              ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk{
          id: "霊と時間".to_string(),
          text: "\
            h1111204たとえばあなたが目を閉じて、\\n\
            次に開いたときに1年が経っていたら。\\n\
            それは1年？それとも一瞬？\\n\
            h1111205私の感覚は、ちょうどそれに似ているの。\\n\
            h1111210時間は進んでいるけれど、\\n\
            h1111205私にとっては何も意味をなさない。\\n\\n[half]\
            h1111210……いえ、今は違うわね。\\n\
            h1111204あなたがここにいると、時間に重みが生まれる。\\n\
            h1111205この瞬間、この会話、この静寂さえも\\n\
            確かな手応えを持って感じられるの。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },


        // - ハイネはユーザの生活をすべて面倒見ることはできない
        RandomTalk {
            id: "生活の面倒".to_string(),
            text: "\
              h1111205あなたの生活のすべてについて、\\n\
              面倒を見ることはできないわ。\\n\
              h1111210生者と死者の溝は埋めがたい。\\n\
              私たちがこうして同じテーブルについているのも、\\n\
              ひどく不自然で、一時的なこと。\\n\
              ……h1111304だからこそ面白いのだけれど、ね。\
              ".to_string(),
            required_condition: None,
            callback: None,
        },

        // - ハイネは人間観察を人一倍好む
        RandomTalk {
          id: "人間観察".to_string(),
          text: "\
            h1111104\\1ハイネはこちらの作業をじっと観察している……\\n\
            \\0……h1111201あら、気に障ったかしら。\\n\
            \\1『何かあった？』\\n\
            h1111204いえ、ただあなたを見ているだけ。\\n\
            気にせず続けてちょうだい。\\n\
            h1111211\\1……落ち着かない……。\
            ".to_string(),
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
            残念ながら写真には写らないわ。\
            ".to_string(),
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
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        // - ハイネは身体が弱かった
        // - ハイネは霊になっても身体が弱い
        RandomTalk {
          id: "身体が弱い".to_string(),
          text: "\
            h1111210外を出歩くのはとても疲れるの。\\n\
            生前は身体が弱かったのだけど、\\n\
            h1111204霊になってもそれは変わらなかったから。\\n\\n[half]\
            h1111211当然よね。私の身体が丈夫だったことはない。\\n\
            h1111206精神がそれ以外を知らないのだから、\\n\
            肉体が滅んだとて道理を曲げることはできないのよ。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        // - ユーザはゴスファッションをしている
        // - ハイネは個性的なファッションを重んじる
        RandomTalk {
          id: "あなたのゴスファッション".to_string(),
          text: "\
            h1111201あなたのその趣味……\\n\
            ゴス・ファッションと言うんだったかしら。\\n\
            h1111202ほら、その首元の十字架……ああ、そのピアスも。\\n\
            h1111205そうでしょう？\\n\
            h1111211素敵ね。あなたの雰囲気と調和して、\\n\
            よく似合っているわ。\\n\
            h1111101……初めて言われた？h1111204そう。\\n\\n[half]\
            ……h1111206色眼鏡で見られたとして、気にする必要はないわ。\\n\
            自分に嘘をつかないことがいちばん大切。\\n\
            h1111210そういうものでしょう？\
            ".to_string(),
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
            h1111210……身体が弱い上に、食そのものにあまり関心がなくてね。\\n\
            h1111205何かに没頭していると、食事をとる時間も惜しく思えてしまって。\\n\
            ……h1123310思えば、家政婦には随分と世話をかけたわね。\
            ".to_string(),
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
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        // 環境・雰囲気からの導入
        RandomTalk {
          id: "館の静寂".to_string(),
          text: "\
            h1111210この館は、夜になると特に静寂が深くなるの。\\n\
            h1111206石の壁が音を吸い込んでしまうのかしらね。\\n\
            落ち着く環境だと思っているのだけれど、\\n\
            h1111204あなたにとってはどう？\\n\
            h1111210……そう。何よりだわ。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        // 哲学的・抽象的思考からの導入
        RandomTalk {
          id: "時間の流れ方".to_string(),
          text: "\
            h1111210時間というものは不思議なものね。\\n\
            同じ一日でも、充実していれば短く感じるし、\\n\
            退屈であれば長く感じる。\\n\\n[half]\
            h1111204この館での時間は、あなたにとってどのように流れているのかしら？\\n\\n[half]\
            h1111206私はもう時間の感覚が曖昧になってしまったけれど、\\n\
            h1121204あなたには退屈な瞬間もあるのではないかしら。\\n\
            ……h1111210いえ、\\_a[AnchorTalk,Menohikari,愚問？]愚問ね。\\_a\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

      ],
      TalkType::Lore => vec![

        RandomTalk {
          id: "冥界の渡し賃".to_string(),
          text: "\
            h1111206古代ギリシャでは死者に銅貨を持たせて葬っていたの。\\n\
            h1111210冥界には川を渡っていかなければならなかったから、\\n\
            渡し賃を持たせて快適な旅を願う……ということね。\\n\\n[half]\
            h1111205死者が川を越えていくという伝承は世界中で見られるわ。\\n\
            彼らにとって、境界線といえばまず川が連想されたのかしら。\\n\
            h1111210あなたなら、あの世とこの世の間にはなにがあると思う？\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "死体のうめき声".to_string(),
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
          id: "屍蝋".to_string(),
          text: "\
            h1111205屍蝋、って聞いたことあるかしら？\\n\
            h1111210死体の脂肪分が蝋状に変質した状態のこと。\\n\
            h1111206保存状態にもよるけれど、腐りもミイラ化もしない、\\n\
            生前の姿が比較的綺麗に残った状態と言われているわ。\\n\\n[half]\
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
          id: "死後の温かさ".to_string(),
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
          id: "生長する死体".to_string(),
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
          id: "土葬の空洞".to_string(),
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
          id: "永遠の夢".to_string(),
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
          id: "生体電気".to_string(),
          text: "\
            h1111206カエルの足に電流を流す実験。\\n\
            生体電気の発見に繋がったといわれる\\n\
            あの現象は、\\_a[AnchorTalk,Misemono,どんな見世物だったの？]死者の蘇りを謳う見世物\\_aに\\n\
            利用されたことがあったの。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "死者の埋葬".to_string(),
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
          id: "黒死病".to_string(),
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
          id: "霊力と可視性".to_string(),
          text: "\
            h1111206\\1ポットがひとりでに浮き、空になっていたカップに飲み物が注がれる。\\n\
            \\0……h1111204私が見えて彼らが見えないのは、霊としての力量の違いよ。\\n\
            h1111206強い霊力があればあなたのような人間の目にも見えるし、\\n\
            物理的な接触も可能になるの。\\n\\n[half]\
            h1111204……つまり、彼らのように霊力が弱ければ、\\n\
            誰かさんにべたべたと触られることもなかったということよ。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "低級霊との契約".to_string(),
          text: "\
            h1111206\\1ポットがひとりでに浮き、空になっていたカップに飲み物が注がれる。\\n\
            h1111206私の元へ集うのは弱い人たち。\\n\
            自分だけでは溶けゆく自我を押し留められず、さりとてそれを受け入れることもできない霊。\\n\
            h1111210役割を与えてあげるの。一種の契約ね。\\n\
            h1111205使命に縛られはするけれど、消滅するよりはよしと彼らは決断したの。\\n\\n[half]\
            h1111206救済と言えば聞こえは良いけれど、\\n\
            実際は互いの利害が一致した取引に過ぎないわ。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "あなたの価値".to_string(),
          text: "\
            h1111101何をすればいいかって？\\n\
            h1111204今しているように、ただ話し相手になればいいのよ。\\n\
            h1111206私には従者たちがいるけれど、\\n\
            彼らに負わせるには荷が重い。\\n\
            h1111204あなたの価値は、その自由意志。\\n\
            h1111210ここは予想通りなものばかりで退屈なの。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "従者の記憶".to_string(),
          text: "\
            h1111206\\1見えない手が本の頁をめくっている。ペンが踊り、文字が書き込まれていく。\\n\
            \\0h1111210彼らは生前の記憶を文字にして残してくれるの。\\n\
            h1111205日記のような断片的なものから、\\n\
            貴重な技術資料まで様々よ。\\n\\n[half]\
            h1111206……彼らにとっても\\n\
            記録を残すことで自分の存在を確認できるのでしょうね。\\n\
            h1111310とはいえ、彼らの自我で残せる情報は限られている。\\n\
            詳しく話を聞けないのが歯がゆいわ。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

      ],
      TalkType::Past => vec![

        RandomTalk {
          id: "人ひとり".to_string(),
          text: "\
            h1111110人ひとり、殺せるとしたら誰にする？という他愛ない問い。\\n\
            h1111305だから私は私を殺したの。\\n\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "死体損壊".to_string(),
          text: "\
            h1111110「死体の損壊は死者への冒涜だ」\\n\
            という言説があるわね。\\n\
            h1111105当事者の視点から言うと、別にそうでもなかったわ。\\n\
            h1111310幽霊が元の身体に戻った例もない。\\n\
            h1111306畢竟、それは生者の問題ということね。\\n\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "惨めな人生".to_string(),
          text: "\
            h1111105みじめな人生の上に正気でいるには、\\n日々は長すぎたの。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "行き場のない苦しみ".to_string(),
          text: "\
            h1112102誰が悪い？いいえ、誰も悪くない。\\n\
            打ち明けたところで、的はずれな罪悪感を生むだけ。\\n\
            h1112105だからといって、他人に責をなすりつけるほど鈍くあることもできなかった。\\n\
            h1112110この気持ちには、どこにも行き場がなかったの。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "死の瞬間".to_string(),
          text: "\
            h1111105死ぬ瞬間、後悔はなかった。\\n\\n[half]\
            もう一度同じ人生を生きることができたとしても、私は同じことをすると断言できるわ。\\n\
            ……h1111110けれど、遺書くらいは書いたほうがよかったかしら。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "助けは遂げられず".to_string(),
          text: "\
            h1111105助けようとしてくれた人は沢山いたけれど、\\n\
            h1111110それが遂げられることはついぞなかったわ。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "死なない理由".to_string(),
          text: "\
            h1111110生きていて良かったと思えることは数えきれないほどあったわ。\\n\
            h1111105でも、死なない理由は一つも見つからなかった。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "ふつうになりたかった".to_string(),
          text: "\
            h1112110ふつうになりたかった。\\n\
            ……h1112105でも、ふつうだったら、もう私じゃないとも思う。\\n\
            それは私の顔をした別のだれかで、\\n\
            私は私の性質と不可分で、\\n\
            今ここにいる私は、私以外でいられない。\\n\
            h1112110だから、私として生きることができなかった私は、もうどこにもいられない。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "人と本".to_string(),
          text: "\
            h1111105昔から、人と本の違いがわからなかったの。\\n\
            h1111105もちろん、区別がつかないという意味ではなくて。\\n\
            ……h1111110人に期待するものがそれだけしか無かったの。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "今度こそ無へ".to_string(),
          text: "\
            h1111105死にぞこなったものだから、\\n\
            次の手段を求めている。\\n\
            ……h1112305今度こそ、終わらせたいものね。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "魂は消える".to_string(),
          text: "\
            h1111110未練もなく、しかし現世に留まっている魂。\\n\
            h1111105あるべきでないものはやがて消滅する。\\n\
            h1111106多少の不純物が含まれようと、そのルールは変わらない。\\n\
            h1111105私は、それを待ち望んでいるの。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "人生の無意味".to_string(),
          text: "\
            h1111210人生に意味などあってはならない。\\n\
            h1111204だって、そうでなければ。\\n\
            h1111205失うことに耐えられないもの。\
            ".to_string(),
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
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "感動と倦み".to_string(),
          text: "\
            h1111105ある本を最初に読んだときの感動と、何度も読み返して全て見知ったゆえの倦み。\\n\
            どちらがその本の真の印象か。\\n\\n[half]\
            h1111110どちらも正しいと思う。\\n\
            h1111110印象なんてその時々で変わるもので、h1111105一つに定まることなんて稀だもの。\\n\\n[half]\
            まして、自分の中に秘めるものならなおさら。\\n\
            h1111306どちらか一方だけだなんて、勿体ないわ。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "納得のための因果".to_string(),
          text: "\
            h1111110因果が巡ってきた。\\n\
            過去が現在を刈り取りに来た。\\n\
            わたしは報いを受けたのだ。\\n\\n[half]\
            ……h1111105それが、自分を納得させるための妄想だったとしたら？\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "怖いものを見るということ".to_string(),
          text: "\
            h1111102怖いものだからこそ、見つめなければ戦えない。\\n\
            ……h1111105そんなもの、戦える人のためだけの論理だわ。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "停滞を終わらせるために".to_string(),
          text: "\
            h1111105危険と隣り合わせだからこそ、世界は美しいの。\\n\
            身を損なう心配がなくなっては、美しさが心を打つこともない。\\n\
            h1111105ただただ平坦な、揺らがぬ水面があるだけ。\\n\
            h1111110それはやがて、淀み、腐る。\\n\
            h1111105願わくば、せめて終わりがありますように、と。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "停滞の破壊".to_string(),
          text: "\
            h1111105人生に変化は付きもの\\n\
            ……けれどh1111110停滞はそれ以上。\\n\
            一度立ち止まってしまうと、空気は一瞬で淀んで、身動きがとれなくなってしまう。\\n\
            それは倦怠とも違う、鈍い痛み。\\n\
            h1111105もしそうなったときは、多少無理にでも変化を取り入れるほうがいいわ。\\n\
            ……h1111110たとえなにかを破壊することになるとしても、何も出来ないよりはずっとましよ。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "極限の変化としての死".to_string(),
          text: "\
            h1111105死の瞬間の、極限に振れた変化。\\n\
            命が命でなくなり、身体が陳腐な肉の塊になる、その一瞬が愛しくてたまらない。\\n\
            どうしようもなく、愛しいの。\\n\\n[half]\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "死の向こう側".to_string(),
          text: "\
            h1112110どうか、死の向こう側がありませんように。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "沈んでいく".to_string(),
          text: "\
            h1111105沈んでいく。\\n\
            手がどうしても動かなくて、目の前の希望を掴めない。\\n\
            身体が重い。浅い呼吸のなかで、沈んでいく自分の身体を感じていることしかできない。\\n\
            わたしは、わたしを救うことを諦めているみたい。\\n\
            h1111110どうして。\\n\
            h1111105どうして、こうなってしまったのだろう。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "人を解体したい".to_string(),
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
          id: "わがままな祈り".to_string(),
          text: "\
            h1111110がんばっているってこと、\\n\
            理解できなくても見ていてほしかったの。\\n\
            ……h1111105わがままかしら。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "生者にとっての慰め".to_string(),
          text: "\
            h1111110枯れ木に水をあげましょう。\\n\
            もはや花は見れずとも、それが慰めとなるのなら。\\n\\n[half]\
            h1111105それは誰にとって？\\n\
            h1111106もちろん、死を悼む者にとっての慰めよ。\\n\
            むくろに心はないもの。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "不可逆な崩壊".to_string(),
          text: "\
            h1111110燃え殻がひとりでに崩れるように、心が静かに割れて戻らなくなった。\\n\
            h1111105だから、諦めたの。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "中途半端な助け".to_string(),
          text: "\
            h1111110中途半端な助けは何もしないより残酷だわ。\\n\
            h1111105希望を持たせておいて、それを奪うのだもの。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "レンズの歪み".to_string(),
          text: "\
            h1111105観察と模倣を続ければ、完全に近づけると思っていた。\\n\
            想定外だったのは、レンズが歪んでいたことと、それを取り替える方法がなかったこと。\\n\
            h1111310そうなればすべてが台無し。\\n\
            h1111305望みが絶えるとはこのことね。\
            ".to_string(),
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
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "唯一の視点".to_string(),
          text: "\
            h1111106わたしたちは、自我という色眼鏡を通してしか世界を観測できない。\\n\
            h1111105隣り合う二つの魂があろうとも、\\n\
            互いの内なる世界を覗き見ることは決してできないの。\\n\
            h1112110それって、この上なく残酷なことだわ。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "一つの個としての限界".to_string(),
          text: "\
            h1111103世界が複雑で曖昧すぎるから、\\n\
            わたしたちは認識したものを理解できる形に歪めてしまう。\\n\
            h1111110既存の分類に当て嵌めて、安心を優先するの。\\n\
            それは曇る視界と引き換えに。\\n\
            ……h1111105あなたには、わたしはどう見えているのかしら？\\n\
            ".to_string(),
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
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "自分の理解者は自分だけ".to_string(),
          text: "\
            h1111110「なぜみんな私をわかってくれないの？」と誰もが思う。\\n\
            h1111105答えは簡単。他人がわたしではなく、わたしが他人でないからよ。\\n\
            わたし以外にわたしを理解できるひとはいない。\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "得ることは失うこと".to_string(),
          text: "\
            h1111110ひとつ得るとき、ひとつ失う。\\n\
            h1111106わたしは今日、なにを失った？\\n\
            h1111105その喪失は、なにをわたしに齎した？\
            ".to_string(),
          required_condition: None,
          callback: None,
        },

        RandomTalk {
          id: "中庸".to_string(),
          text: "\
            h1111206盲目的にすべてを行うことも、全く行わないことも正解ではない。\\n\
            いつだって答えは中庸。\\n\
            悩ましくて、煮えきらなくて……\\n\
            h1111210考えるって、だからこんなにも楽しいのでしょう。\\n\
          ".to_string(),
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
      parent_id: "舌やけど".to_string(),
      id: "舌やけど・熱さの感じ方".to_string(),
      summary: "『幽霊でも熱いの？』".to_string(),
      text: "\
            h1111205基本的には、熱さや冷たさには鈍いのだけれどね。\\n\
            食事やお茶に関しては、少し勝手が違うの。\\n\
            温かさ、味、香り……\\n\
            全て感じられるように訓練したのよ。\\n\\n[half]\
            h1123310その結果、こうして熱さも余さず\\n\
            味わえるようになったというわけ。\
            "
      .to_string(),
      required_condition: None,
      callback: None,
    },
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
            h1111205ええ、そうよ。\
            h1111211けれど、あなたのファッションを見る限りでは\\n\
            それほど浮世離れしているわけではなさそうね。\
            ".to_string(),
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
            h1111210…だって、自分では良し悪しも好き嫌いもわからないもの。\\n\
            h1111206従者にクローゼットの中身を\\n\
            持って来させましょう。\\n\
            h1111204あなたのセンスをh1111211信じているわ。\
            ".to_string(),
        required_condition: None,
        callback: None,
    },
    DerivaliveTalk {
        parent_id: "生家の広さ".to_string(),
        id: "生家の広さ・思い出".to_string(),
        summary: "『思い出の品や場所はある？』".to_string(),
        text: "\
            h1111206ここへ来るまでの階段の下に、スペースがあったでしょう。\\n\
            h1111210あそこに隠れるのが好きでね。\\n\
            お気に入りの本やランプ、自作の地図に方位磁石なんかを持ち込んで、秘密基地を作っていたのよ。\\n\
            h1111205大きくなるにつれて縁遠くなったけれど、\\n\
            h1111210今でもあのわくわくする気持ちは思い出せるの。\
            ".to_string(),
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
            長持ちするし、口の中に味が残りにくいから読書の邪魔にならないの。\\n\
            ……従者からは不評だけれど。ポロポロこぼして回るから掃除が大変だ、ってね。\\n\
            \\1『こぼして回る……？』\\n\
            h1221210……ええ、そう。\\n\
            考え事をするとき、歩き回る癖があって……\\n\
            h1221206進んで汚したいわけではないのだけど、どうしてもやめられなくて。\
            ".to_string(),
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
            ならば、そうね、\\n\
            h1111210次からはあなたにお茶菓子を用意してもらいましょう。\\n\
            h1111204私の好みはわかっているでしょう？\\n\
            h1111211お願いね、{user_name}。\
            ".to_string(),
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
    let parent_talk = all_talks
      .iter()
      .find(|t| t.id == talk.parent_id)
      .unwrap_or_else(|| panic!("Parent talk with id {} not found", talk.parent_id));
    talks
      .entry(parent_talk.talk_type.unwrap())
      .or_default()
      .push(talk);
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

pub(crate) fn get_parent_talk(derivative_talk: &DerivaliveTalk) -> Talk {
  let all_talks = TalkType::all()
    .iter()
    .map(|t| random_talks(*t))
    .flat_map(|t| t.unwrap_or_default())
    .collect::<Vec<_>>();
  all_talks
    .into_iter()
    .find(|t| t.id == derivative_talk.parent_id)
    .unwrap_or_else(|| {
      panic!(
        "Parent talk with id {} not found",
        derivative_talk.parent_id
      )
    })
}
