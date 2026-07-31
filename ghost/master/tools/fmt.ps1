# cargo fmt + トーク文字列内部インデントの正規化（rustfmt が触れない領域）
$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")
cargo fmt @args
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
python tools/normalize_string_indent.py src
exit $LASTEXITCODE
