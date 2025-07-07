## 新たなトーク案生成の指針
`new_talks [talktype]`コマンドが呼ばれた場合、以下の方針で新しいトークを生成してください。
- [@ghost/master/src/events/talk/randomtalk.rs](ghost/master/src/events/talk/randomtalk.rs) で定義されているトークの傾向に基づき、ハイネの性格やトーク傾向に合致する新しいトークを生成する
- ユーザーへの応答は日本語で行う
- talktypeは以下のいずれかを指定する
  - `SelfIntroduce`
  - `WithYou`
  - `Lore`
  - `Past`
  - `Abstract`
- talktypeが指定されていない場合は、`WithYou`をデフォルトとする
- トークの内容は、ハイネの性格やトーク傾向に合致するように注意する
- トーク数の指定がなければ、3つのトークを生成する
- 既存のトークを参考にしつつ、ハイネの性格やトーク傾向に合致する新しいトークを生成する
