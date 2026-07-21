#!/usr/bin/env bash
# rust-analyzer の rustfmt.overrideCommand 用ラッパー。
# stdin のソースを rustfmt → 文字列内インデント正規化 の順に通して stdout へ返す。
# rustfmt が失敗（構文エラー等）したら何も出力せず非0で終了し、バッファ破壊を防ぐ。
set -euo pipefail
dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT
rustfmt --edition 2021 "$@" > "$tmp"
python3 "$dir/normalize_string_indent.py" --stdin < "$tmp"
