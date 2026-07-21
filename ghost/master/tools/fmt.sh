#!/usr/bin/env bash
# cargo fmt + トーク文字列内部インデントの正規化（rustfmt が触れない領域）
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
cargo fmt "$@"
python3 tools/normalize_string_indent.py src
