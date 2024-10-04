$isRequirementsInstalled = $true

function Send-SSTP {
  param(
    [string]$message,
    [string]$uniqueid
  )

  $tcpClient = New-Object System.Net.Sockets.TcpClient("localhost", 9801)
  $stream = $tcpClient.GetStream()
  $writer = New-Object System.IO.StreamWriter($stream)
  $writer.WriteLine("SEND SSTP/1.0")
  $writer.WriteLine("Charset: UTF-8")
  $writer.WriteLine("Sender: Haine Builder")
  $writer.WriteLine("Script: $message")
  $writer.WriteLine("Option: notranslate")
  if ($uniqueid) {
    $writer.WriteLine("ID: $uniqueid")
  }
  $writer.WriteLine()
  $writer.Flush()
}

# check if magick is installed
if (!(Get-Command "magick" -ErrorAction SilentlyContinue)) {
    Write-Host "magick is not installed"
    $isRequirementsInstalled = $false
}

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

# ./ghost/master/debug が存在するか確認し、存在するなら内容を読み込む
if (Test-Path $PSScriptRoot\ghost\master\debug) {
  $uniqueid = Get-Content $PSScriptRoot\ghost\master\debug
}

Send-SSTP "\1\_qビルド中\![unload,shiori]\e" $uniqueid

Start-Sleep -Seconds 1

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

surfaces-mixer -f -w $arg -i $PSScriptRoot\shell\master\surfaces.yaml -o  $PSScriptRoot\shell\master\surfaces.txt

# ろうそく画像をリサイズしてサーフェス画像としてリネーム
$prefix = "$PSScriptRoot\shell\master"
$size = 300
$surface_number_original = 10000000
$collision_images = @()
$collision_image = "$prefix\immersion_candle_collision.png"
$collision_images += $collision_image
magick convert -resize ${size}x${size} "$prefix\immersion_candle_base.png" "$prefix\surface$surface_number_original.png"
magick convert -fill "rgb(0,255,0)" -colorize 100 "$prefix\surface$surface_number_original.png" $collision_image
for ($i = 1; $i -le 5; $i++) {
  for ($j = 1; $j -le 2; $j++) {
    $surface_number = $surface_number_original + $i + 10 * ($j - 1)
    magick convert -resize ${size}x${size} "$prefix\immersion_candle_fire_${i}_${j}.png" "$prefix\surface$surface_number.png"
  }
}

# 消えるろうそく画像をリサイズしてサーフェス画像としてリネーム
for ($i = 1; $i -le 5; $i++) {
  $surface_number = $surface_number_original + $i + 100
  magick convert -resize ${size}x${size} "$prefix\immersion_candle_fire_${i}_0.png" "$prefix\surface$surface_number.png"
}

# $collision_imagesを重ねて出力
$collision_image_name = "$prefix\immersion_candle_master_collision.png"
magick convert $collision_images -composite $collision_image_name
Remove-Item $collision_images

Send-SSTP "\1\_qビルド完了\![reload,ghost]\e" $uniqueid
