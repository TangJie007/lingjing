# 下载便携版 MPV 到 src-tauri/bin/mpv（供开发与打包使用）
$ErrorActionPreference = "Stop"

$mpvUrl = "https://github.com/shinchiro/mpv-winbuild-cmake/releases/download/20260605/mpv-x86_64-20260605-git-4013a3c.7z"
$root = Split-Path $PSScriptRoot -Parent
$cacheDir = Join-Path $root ".cache"
$mpvArchive = Join-Path $cacheDir "mpv.7z"
$extractDir = Join-Path $cacheDir "mpv-extract"
$destDir = Join-Path $root "src-tauri\bin\mpv"
$sevenZip = "C:\Program Files\7-Zip\7z.exe"

if (-not (Test-Path $sevenZip)) {
  Write-Host "正在安装 7-Zip..."
  winget install 7zip.7zip --accept-package-agreements --accept-source-agreements
}

New-Item -ItemType Directory -Force -Path $cacheDir, $destDir | Out-Null

if (-not (Test-Path $mpvArchive)) {
  Write-Host "下载 MPV..."
  Invoke-WebRequest -Uri $mpvUrl -OutFile $mpvArchive -UseBasicParsing
}

if (Test-Path $extractDir) { Remove-Item $extractDir -Recurse -Force }
New-Item -ItemType Directory -Force -Path $extractDir | Out-Null
& $sevenZip x $mpvArchive "-o$extractDir" -y | Out-Null

$mpvExe = Get-ChildItem $extractDir -Filter mpv.exe -Recurse | Select-Object -First 1
if (-not $mpvExe) { throw "解压后未找到 mpv.exe" }

Copy-Item -Path (Join-Path $mpvExe.Directory.FullName "*") -Destination $destDir -Recurse -Force
Write-Host "MPV 已安装到 $destDir"
& (Join-Path $destDir "mpv.exe") --version | Select-Object -First 1
