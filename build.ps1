cd ghost\master
cargo build --release
cp -Verbose -Force .\target\i686-pc-windows-msvc\release\haine.dll .\

# カレントディレクトリ以下の.rsファイルから、7文字の数字をすべて検索し、リスト化する
$files = Get-ChildItem -Path .\src -Filter *.rs -Recurse | ForEach-Object {
    $content = Get-Content $_.FullName -Raw
    $matches = [Regex]::Matches($content, '\d{7}')
    $matches | ForEach-Object { $_.Value }
}

# 重複を除去して、該当の7文字数列をカンマ区切りで標準出力する
$arg = (($files | Select-Object -Unique) -join ',')

surfaces-mixer -f -w $arg -i $PSScriptRoot\shell\master\surfaces.yaml -o  $PSScriptRoot\shell\master\surfaces.txt
