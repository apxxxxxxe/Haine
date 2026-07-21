$isRequirementsInstalled = $true

function Send-SSTP {
  param(
    [string]$message,
    [string]$uniqueid
  )

  $tcpClient = New-Object System.Net.Sockets.TcpClient("localhost", 9801)
  try {
    $stream = $tcpClient.GetStream()
    $writer = New-Object System.IO.StreamWriter($stream, [System.Text.UTF8Encoding]::new($false))
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

    # 応答を読み切ることでスクリプト実行完了まで待つ
    $reader = New-Object System.IO.StreamReader($stream, [System.Text.Encoding]::UTF8)
    $response = $reader.ReadLine()
    Write-Host "SSTP: $response"
  } finally {
    $tcpClient.Close()
  }
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

if (!$isRequirementsInstalled) {
    Write-Host "Requirements are not installed. Please install the requirements and try again."
    exit 1
}

# ./ghost/master/debug が存在するか確認し、存在するなら内容を読み込む
if (Test-Path $PSScriptRoot\ghost\master\debug) {
  $uniqueid = Get-Content $PSScriptRoot\ghost\master\debug
}

# ろうそく画像をリサイズしてサーフェス画像としてリネーム
$prefix = "$PSScriptRoot\shell\master"
$size = 300
$surface_number_original = 10000000
$collision_images = @()
$collision_image = "$prefix\immersion_candle_collision.png"
$collision_images += $collision_image
magick "$prefix\immersion_candle_base.png" -strip -resize ${size}x${size} "$prefix\surface$surface_number_original.png"
magick "$prefix\surface$surface_number_original.png" -strip -fill "rgb(0,255,0)" -colorize 100 $collision_image
for ($i = 1; $i -le 5; $i++) {
  for ($j = 1; $j -le 2; $j++) {
    $surface_number = $surface_number_original + $i + 10 * ($j - 1)
    magick "$prefix\immersion_candle_fire_${i}_${j}.png" -strip -resize ${size}x${size} "$prefix\surface$surface_number.png"
  }
}

# 消えるろうそく画像をリサイズしてサーフェス画像としてリネーム
for ($i = 1; $i -le 5; $i++) {
  $surface_number = $surface_number_original + $i + 100
  magick "$prefix\immersion_candle_fire_${i}_0.png" -strip -resize ${size}x${size} "$prefix\surface$surface_number.png"
}

# $collision_imagesを重ねて出力
# PNG32必須: パレット型PNGだとSSPのregion当たり判定が色を拾えず [Empty rectangle] になる
$collision_image_name = "$prefix\immersion_candle_master_collision.png"
magick $collision_images -strip -background none -flatten "PNG32:$collision_image_name"
Remove-Item $collision_images

Send-SSTP "\1\_qビルド中\![unload,shiori]\e" $uniqueid

Start-Sleep -Seconds 1

cd $PSScriptRoot\ghost\master
cargo build --release

# unload完了前だとDLLが使用中のことがあるためリトライする
$copied = $false
for ($retry = 0; $retry -lt 20; $retry++) {
  try {
    Copy-Item -Force -ErrorAction Stop $PSScriptRoot\ghost\master\target\i686-pc-windows-msvc\release\haine.dll $PSScriptRoot\ghost\master\
    $copied = $true
    break
  } catch {
    Write-Host "haine.dll is in use, retrying... ($($retry + 1)/20)"
    Start-Sleep -Milliseconds 500
  }
}
if (!$copied) {
  Write-Host "Failed to copy haine.dll: file is still in use."
  Send-SSTP "\1\_qビルド失敗\![reload,ghost]\e" $uniqueid
  exit 1
}

Send-SSTP "\1\_qビルド完了\![reload,ghost]\e" $uniqueid
