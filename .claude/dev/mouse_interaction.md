# 実装詳細: マウスインタラクション

2026-07-21 に main 相当の方式へ復元した。部位別のプールからのランダム選択と、連続タッチ回数（`touch_count`）による反応の変化だけで構成する。かつての 3 段階制（警戒→動揺→受容）と、その撤去後に一時採用した単層モデル R1 は、どちらも廃止済み。`RELATIONSHIP_STAGE` や段階ゲートの類はコードに存在しない。

## 機構

- 反応部位: head / face / hand（なで）、bust（なで・ダブルクリック同一扱い）、skirt（めくり）、shoulder（ホイール）。当たり判定は `shell/master/surfaces.txt` の collisionex 定義
- 選択は `common_choice_process`（ルーレット式ランダム）＋ `phased_talks`（連続回数で候補プールを切り替え）
- `TouchInfo`（`system/variables.rs`）が部位ごとの連続回数を保持。時間経過（`reset_if_timeover`）と別部位への移動でリセット
- bust は回数で反応が推移する: 通常 → 叱責（`DIALOG_SEXIAL_SCOLD`）→ 呆れ（`DIALOG_SEXIAL_AKIRE`）→ 12 回目で霧化ペナルティ → 以降は無言
- 初回性的接触（`FIRST_SEXIAL_TOUCH`）: 起動 30 秒以内かつ `FirstClose` 済みなら、bust/skirt の初回に専用反応（`DIALOG_SEXIAL_FIRST`）。揮発変数なので起動ごとに戻る
- 書斎（`TalkingPlace::Library`）ではなで反応の代わりに `on_ai_talk` へ流す（skirt は無反応）

## 存続する技術機構

- **チェイントーク**: ランダムトーク→制限時間内に部位を触る→特殊反応。`ChainTalkState` 構造体（`variables.rs`）、`CHAIN_TALK_STATE` volatile 変数、`check_chain_talk()`（`mouse.rs`）、`\\p[2]` ヒント表示、自動テスト `mouse::tests::test_chain_talk_mechanism`。段階非依存の独立機構
- **蝋燭操作（没入度・居間⇔書斎）**: `two_candle_double_click`

## トーク作成方針

- `\\1` は**ユーザの独白**（受動的表現・三人称的描写は誤り）。客観描写も、ハイネの身体反応の目撃も、双方向の交流も反応に応じて選ぶ
- **観察眼は健在**: ユーザの手の動き・表情を正確に捉える。捉えたものを分析として返すか言語化せず感じるかは反応ごとに振れてよい
- 避けるべき表現: 立場のひけらかし、感情の冗長な説明、過去の詳細開示、死への軽々しい言及、中途半端な受容（「嫌いではないけれど」等）
