### トークの修正・改善時の指針
`new_touch_talks [body_part]`コマンドが呼ばれた場合、以下の方針で新しいトーク案を生成してください。
- [@ghost/master/src/events/mouse.rs](ghost/master/src/events/mouse.rs) で定義されているトークの傾向に基づき、ハイネの性格やトーク傾向に合致する新しい触り反応トーク案を生成、提案する
- ユーザーへの応答は日本語で行う
- body_partには以下のいずれかを指定する
  - `head`
  - `face`
  - `hand`
  - `bust`
  - `skirt`
  - `shoulder`
- talktypeが指定されていない場合は、上記からランダムに選択する。
  - シェルスクリプトでランダムに選択する場合は、`shuf -n 1 -e head face hand bust skirt shoulder`を使用する
- トークの内容は、ハイネの性格やトーク傾向に合致するように注意する
- トーク数の指定がなければ、3つのトークを生成する
- 既存のトークを参考にしつつ、ハイネの性格やトーク傾向に合致する新しいトークを生成する

