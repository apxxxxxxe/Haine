
$isRequirementsInstalled = $true

# check if cargo is installed
if (!(Get-Command "cargo" -ErrorAction SilentlyContinue)) {
    Write-Host "cargo is not installed"
    $isRequirementsInstalled = $false
}

# check if surfaces-mixer is installed
if (!(Get-Command "surfaces-mixer" -ErrorAction SilentlyContinue)) {
    Write-Host "surfaces-mixer is not installed"
    Write-Host "Please install surfaces-mixer using the following command"
    Write-Host "go install github.com/apxxxxxxe/surfaces-mixer@v0.3.0"
    $isRequirementsInstalled = $false
}

if (!$isRequirementsInstalled) {
    Write-Host "Requirements are not installed. Please install the requirements and try again."
    exit 1
}

cd $PSScriptRoot\ghost\master
cargo build --release
cp -Verbose -Force $PSScriptRoot\ghost\master\target\i686-pc-windows-msvc\release\haine.dll $PSScriptRoot\ghost\master\

# カレントディレクトリ以下の.rsファイルから、7文字の数字をすべて検索し、リスト化する
$files = Get-ChildItem -Path $PSScriptRoot\ghost\master\src -Filter *.rs -Recurse | ForEach-Object {
  $content = Get-Content $_.FullName -Raw
  $matches = [Regex]::Matches($content, '\d{7}')
  $matches | ForEach-Object {
    $baseNumber = $_.Value.Substring(0, 5)
    $lastTwoDigits = $_.Value.Substring(5)
    1..15 | ForEach-Object {
      $baseNumber + $_.ToString("00")
    }
  }
}

# 重複を除去して、該当の7文字数列をカンマ区切りで標準出力する
$arg = (($files | Select-Object -Unique) -join ',')

echo $arg

surfaces-mixer -f -w $arg -i $PSScriptRoot\shell\master\surfaces.yaml -o  $PSScriptRoot\shell\master\surfaces.txt
