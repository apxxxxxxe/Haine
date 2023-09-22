use crate::events::common::*;
use crate::events::GlobalVariables;
use shiorust::message::{Request, Response};

pub fn version(_req: &Request, _vars: &mut GlobalVariables) -> Response {
    new_response_with_value(String::from(env!("CARGO_PKG_VERSION")), false)
}

pub fn on_ai_talk(_req: &Request, _vars: &mut GlobalVariables) -> Response {
    let talks = [
        "\
        h1111209生と死はグラデーションか、二極対立か。\\n\
        議論に興味はないけれど、h1111205どちらにせよ、決定的な瞬間があるはず。\\n\
        h1111205その、死の瞬間。正直に言うと、私は、それがどうしようもなく愛しいの。命が命でなくなり、身体が陳腐な肉の塊になる、その一瞬が愛しくてたまらない。\\n\\n\
        h1111206私はきっと正しくない。あなたの目には、贔屓目にも綺麗には写らない。それは、悲しいことだけれど。\\n\\n\
        h1111309だけど、それがなんだというの。倫理は私を救わない。あなたの心を私が握る必要もない。私が諦める理由にはならないわ。\\n\
        h1111304それを諦められるくらい、私は、愛を諦められないのだから。\\n\\n\
        h1111205諦めなければ、いつか。\\n\
        h1111205……啖呵を切った手前でこれを言うのは、傲慢だけれど。\\n\
        h1111207見届けてね、%(username)。\
        ",

        "\
        h1111209霧がなければ生きられない。\\n\
        霧があるから生きている。\\n\
        私は霧に生かされている。\\n\
        h1111205私に明日は、\\_a[Nolonger]もう来ない。\\_a\
        ",

        "\
        h1111209あなたたちが歩いている姿を、いつも窓から見ているの。\\n\
        h1111204いつも何かをして、どこかへ向かっている。\\n\
        h1111207羨ましいわ。h1111207私は\\_a[Fastened,どういうこと？]見ていることしかできない\\_aから、なおさら。\
        ",
    ];

    let talk = choose_one(&talks).unwrap();

    new_response_with_value(talk, true)
}
