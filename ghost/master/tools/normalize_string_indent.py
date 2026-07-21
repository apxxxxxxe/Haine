#!/usr/bin/env python3
r"""Rustソース中の `\` 行継続文字列リテラルの継続行インデントを、開始行インデント+2に揃える。

rustfmt は文字列リテラルの中身に触れないため、トーク本文の行頭空白はこのスクリプトで揃える。
行継続 `\<newline>` の直後の行頭空白はRust仕様で値から捨てられるので、この編集は値を変えない。
継続でない生の改行を含む文字列・raw string・コメント・char literal は触らない。
編集前後で各リテラルの「値」（\<newline>+空白 を除去したもの）が一致することを assert する。

使い方:
  normalize_string_indent.py <dir|file>...   # .rs ファイルを in-place で正規化
  normalize_string_indent.py --stdin         # stdin のソースを正規化して stdout へ（エディタ連携用）
"""
import re
import sys
from pathlib import Path

ELIDE = re.compile(r"\\\n[ \t]*")


def literal_value(token_text: str) -> str:
  """行継続の elision のみ適用した比較用の値。他のエスケープはそのまま。"""
  return ELIDE.sub("", token_text)


def find_strings(src: str):
  """通常の文字列リテラルの (start, end, all_newlines_escaped) を列挙する。

  end は閉じ引用符の位置。raw string / コメント / char literal はスキップ。
  """
  results = []
  i = 0
  n = len(src)
  while i < n:
    c = src[i]
    if c == "/" and i + 1 < n:
      if src[i + 1] == "/":  # 行コメント
        j = src.find("\n", i)
        i = n if j == -1 else j + 1
        continue
      if src[i + 1] == "*":  # ブロックコメント（ネスト対応）
        depth = 1
        j = i + 2
        while j < n - 1 and depth > 0:
          if src[j] == "/" and src[j + 1] == "*":
            depth += 1
            j += 2
          elif src[j] == "*" and src[j + 1] == "/":
            depth -= 1
            j += 2
          else:
            j += 1
        i = j
        continue
    if c == "r" and i + 1 < n and src[i + 1] in '"#':  # raw string
      m = re.match(r'r(#*)"', src[i:])
      if m:
        closer = '"' + m.group(1)
        j = src.find(closer, i + len(m.group(0)))
        i = n if j == -1 else j + len(closer)
        continue
    if c == "'":  # char literal（ライフタイムは非マッチ）
      m = re.match(r"'(\\.[^']*|[^'\\])'", src[i:])
      if m:
        i += len(m.group(0))
        continue
      i += 1
      continue
    if c == '"':
      start = i
      j = i + 1
      all_escaped = True
      has_newline = False
      while j < n:
        if src[j] == "\\":
          if j + 1 < n and src[j + 1] == "\n":
            has_newline = True
          j += 2
          continue
        if src[j] == "\n":
          has_newline = True
          all_escaped = False
        if src[j] == '"':
          break
        j += 1
      if j >= n:
        break  # 閉じられていない（走査ミス）→ 以降触らない
      if has_newline:
        results.append((start, j, all_escaped))
      i = j + 1
      continue
    i += 1
  return results


def normalize_text(src: str, label: str = "<stdin>"):
  """(正規化後ソース, 変更行数) を返す。"""
  line_starts = [0]
  for idx, ch in enumerate(src):
    if ch == "\n":
      line_starts.append(idx + 1)

  def line_no(pos: int) -> int:
    lo, hi = 0, len(line_starts) - 1
    while lo < hi:
      mid = (lo + hi + 1) // 2
      if line_starts[mid] <= pos:
        lo = mid
      else:
        hi = mid - 1
    return lo

  lines = src.split("\n")
  plan = {}
  literals = []
  for start, end, all_escaped in find_strings(src):
    if not all_escaped:
      continue  # 生の改行を含む文字列は不触
    opener_line = line_no(start)
    opener_indent = len(lines[opener_line]) - len(lines[opener_line].lstrip(" "))
    target = " " * (opener_indent + 2)
    cont_lines = list(range(opener_line + 1, line_no(end) + 1))
    literals.append((start, end, cont_lines, target))
    for ln in cont_lines:
      plan[ln] = target

  if not plan:
    return src, 0

  before_values = [literal_value(src[s : e + 1]) for s, e, _, _ in literals]

  new_lines = list(lines)
  changed = 0
  for ln, target in plan.items():
    stripped = new_lines[ln].lstrip(" \t")
    new = target + stripped
    if new != new_lines[ln]:
      new_lines[ln] = new
      changed += 1
  new_src = "\n".join(new_lines)

  # 値の不変検証: 編集後ソースを再走査して同じリテラル群の値を比較
  new_literals = [t for t in find_strings(new_src) if t[2]]
  assert len(new_literals) == len(literals), f"{label}: literal count changed"
  for (s, e, _), before in zip(new_literals, before_values):
    after = literal_value(new_src[s : e + 1])
    assert after == before, f"{label}: literal value changed near byte {s}"

  return new_src, changed


def process_file(path: Path) -> int:
  # newline='' で改行変換を殺す（Windowsで CRLF 化させない）
  with open(path, encoding="utf-8", newline="") as fp:
    src = fp.read()
  new_src, changed = normalize_text(src, str(path))
  if changed:
    with open(path, "w", encoding="utf-8", newline="") as fp:
      fp.write(new_src)
  return changed


def main():
  args = sys.argv[1:]
  if args == ["--stdin"]:
    src = sys.stdin.buffer.read().decode("utf-8")
    new_src, _ = normalize_text(src)
    sys.stdout.buffer.write(new_src.encode("utf-8"))
    return
  if not args or "--stdin" in args:
    print(__doc__, file=sys.stderr)
    sys.exit(2)
  total = 0
  for arg in args:
    p = Path(arg)
    targets = sorted(p.rglob("*.rs")) if p.is_dir() else [p]
    for path in targets:
      n = process_file(path)
      if n:
        print(f"{n:5d}  {path}")
      total += n
  print(f"total reindented lines: {total}")


if __name__ == "__main__":
  main()
