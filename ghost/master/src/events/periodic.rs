use crate::events::aitalk::on_ai_talk;
use crate::events::common::*;
use crate::variables::get_global_vars;
use rand::Rng;
use shiorust::message::{Request, Response};

pub fn on_second_change(req: &Request) -> Response {
    let vars = get_global_vars();
    let total_time = vars.total_time.unwrap();
    vars.total_time = Some(total_time + 1);
    vars.volatility.ghost_up_time += 1;

    let refs = get_references(req);
    let idle_secs = refs[4].parse::<i32>().unwrap();
    vars.volatility.idle_seconds = idle_secs;

    if vars.volatility.ghost_up_time % vars.random_talk_interval.unwrap() == 0 {
        on_ai_talk(req)
    } else {
        new_response_nocontent()
    }
}

pub fn on_hour_time_signal(_req: &Request) -> Response {
    let now = chrono::Local::now();
    let hour = now.format("%H").to_string();
    let minute = now.format("%M").to_string();
    let mut m = format!("\\1\\_q{}時", hour);
    if minute != "00" {
        m += &format!("{}分", minute);
    }
    m += "\\n\\n";

    let tanka_list = [
      "もう二度と死ななくてよい安らぎに\\n見つめてゐたり祖母の寝顔を\\n\\f[align,right](梶原さい子)",
      "眼のまはり真紅(まあか)くなして泣きやめぬ\\n妻のうしろに吾子死にてあり\\n\\f[align,right](木下利玄)",
      "我が母よ死にたまひゆく我が母よ\\n我(わ)を生まし乳足(ちた)らひし母よ\\n\\f[align,right](斎藤茂吉)",
      "眠られぬ母のためわが誦む童話\\n母の寝入りし後王子死す\\n\\f[align,right](岡井隆)",
      "死せる犬またもわが眼にうかび来ぬ、\\nかの川ばたの夕ぐれの色\\n\\f[align,right](金子薫園)",
      "死に一歩踏み入りしとふ実感は\\nひるがへつて生の実感なりし\\n\\f[align,right](後藤悦良)",
      "蛍光灯のカヴァーの底を死場所としたる\\nこの世の虫のかずかず\\n\\f[align,right](小池光)",
      "死に向かふ生の底知れぬ虚無の淵を\\nのぞき見たりき彼の夜の君に\\n\\f[align,right](柴生田稔)",
      "やわらかく厚い果肉を掘りすすみ\\n核の付近で死んでいる虫\\n\\f[align,right](北辻千展)",
      "死にし子をまつたく忘れてゐる日あり\\n百日忌日(ひやくにちきじつ)にそれをしぞ嘆く\\n\\f[align,right](吉野秀雄)",
      "十トンの恐竜もゐしこの星に\\n四十八キロの妻生きて死す\\n\\f[align,right](高野公彦)",
      "生まれてはつひに死ぬてふことのみぞ\\n定めなき世に定めありける\\n\\f[align,right](平維盛)",
    ];

    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..tanka_list.len());

    new_response_with_value(m + tanka_list[index], true)
}
