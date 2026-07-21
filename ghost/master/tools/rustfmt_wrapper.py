#!/usr/bin/env python3
"""rust-analyzer の rustfmt.overrideCommand 用ラッパー（Windows / WSL 共通）。

stdin のソースを rustfmt → 文字列内インデント正規化 の順に通して stdout へ返す。
rustfmt が失敗（構文エラー等）したら何も出力せず非0で終了し、バッファ破壊を防ぐ。

設定例（nvim の rust-analyzer settings）:
  rustfmt = { overrideCommand = { "python", "tools/rustfmt_wrapper.py" } }
  （WSL 側は "python3"。相対パスはワークスペースルート ghost/master 基準で解決される）
"""
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from normalize_string_indent import normalize_text


def main():
  src = sys.stdin.buffer.read()
  proc = subprocess.run(
    ["rustfmt", "--edition", "2021", *sys.argv[1:]],
    input=src,
    capture_output=True,
  )
  if proc.returncode != 0:
    sys.stderr.buffer.write(proc.stderr)
    sys.exit(proc.returncode)
  new_src, _ = normalize_text(proc.stdout.decode("utf-8"))
  sys.stdout.buffer.write(new_src.encode("utf-8"))


if __name__ == "__main__":
  main()
