# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

このリポジトリには「プログラムとしてのプロジェクト構造」と「創作物としての設定・規則」が併存する。本ファイルは**案内役**に徹し、詳細は下記の子ファイルに退避してある。常時参照する短いコマンド類のみここに残す。

## 資料の所在

### 創作設定 `.claude/lore/`
- `haine_about.md` — 世界観・人物・関係性の総合資料（人物設定／既存の設定の SSOT）
- `haine_derived.md` — 現行トークを下から読み直した人物像の観測＋停滞を破壊で抜ける実験場（canon ではない。書き戻し候補の置き場）
- `mansion_layout.md` — 館の部屋・設備・小道具の一覧
- `canterberio_layout.md` — 街カンテルベリオの区画・施設・地形の一覧（mansion_layout の街版。物理・地理の台帳）
- `situations.md` — 場面別のハイネの振る舞い
- `talk_audit_progress.md` — トーク監査の進捗

### 開発 `.claude/dev/`
- `architecture.md` — プロジェクト構造・イベント系・ビルド工程・テスト方針
- `mouse_interaction.md` — マウス反応の実装詳細（単層モデル R1）

### トーク指針
- 旧 `talk-guide` スキルは 2026-07-02 に**意図的に廃止**（品質向上に寄与しない制約の増築で汚染されたため。同じ増築を繰り返さない）。
- トーク作成は `.claude/skills/haine-mimicry/skill.md`（手本駆動）。表現の基礎は同スキル同梱の `references/expression-basics.md`、書式・ウェイトの技術仕様は `.claude/dev/talk_format.md`。
- 草稿のレビューは `.claude/skills/stop-ai-slop-talk-jp/`（台詞版 AI slop 点検。haine-mimicry ステップ4.5 が呼ぶ）。**生成中は開かない**——書き終わってから使う。台詞の作法を規則で足す場所ではないので、増築したくなったら既存資料に既にあるか先に確認する。

## 文書を書くときの言葉遣い

`.claude/` 配下の資料（lore・dev・本ファイル）やスキル定義を書く・直すときに守る。トーク本文（ハイネの台詞）の作法は haine-mimicry スキル（同梱 `references/expression-basics.md`）と `.claude/dev/talk_format.md`、台詞の AI 臭点検は `stop-ai-slop-talk-jp` に分離してある。

1. **AI 臭の語法を持ち込まない**。禁止リストと判定の詳細は `.claude/skills/stop-ai-slop-jp/`。とくに紛れ込みやすいのは:
   - **false agency** — モノを主語に人間の動詞をさせる（「温度差が物語る」「課題が浮き彫りになる」）。誰が何をしたかに書き換える。
   - **必殺技造語・翻訳調動詞** — 普通の感想を漢字熟語や直訳調で膨らます（「凝縮」「結実」「示している」「収斂する」）。
   - **二項対比テンプレ** — 「単なる A ではなく B」は直接 B を書く。
2. **比喩を定義語にするなら、初出で普通の言葉に開く**。「手品化」「成形」のように比喩一語で意味を運ばせない。一度ふつうの言葉で説明してから、以降の反復ショートカットとして使う。すでに定義済みで反復が前提の核概念はそのまま使ってよい。

## Build Commands

- **Primary build**: `pwsh.exe .\build.ps1` or `.\build.bat`
- **Rust build only**: `cd ghost/master && cargo build --release`
- **Format code**: `cd ghost/master && cargo fmt`
- **Lint code**: `cd ghost/master && cargo clippy`

## Development Dependencies

The build system requires:
- PowerShell
- Rust/Cargo
- ImageMagick (`magick` command)
- surfaces-mixer: `go install github.com/apxxxxxxe/surfaces-mixer@v0.3.0`

## Code Style
- Uses 2-space indentation (configured in `rustfmt.toml`)
- Clippy type complexity threshold set to 1000
- Extensive use of macros for error handling and regex patterns
- Async/await patterns with Tokio runtime

## パワーバランス設計

ハイネ（ホスト）とユーザ（ゲスト）の権力勾配を踏まえたトーク作成の手すり。診断と手すりは `.claude/skills/haine-mimicry/skill.md`「北極星制約」節に集約——禁止は一つ（立場差を利用してユーザに優位に立たない）、狙いも一つ（対等な二者の摩擦を書く）。旧 `haine_vs_user.md` は 2026-07-01 に旧 talk-guide へ統合、talk-guide 廃止（2026-07-02）後は北極星制約節が唯一の生きた記述。
