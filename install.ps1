# geek-cli installer for Windows PowerShell
#   irm https://yangtzeu.work/geek/install.ps1 | iex
#   irm https://github.com/Yangtze-University-Geek-Class/geek-cli/releases/latest/download/install.ps1 | iex
#
# Env overrides:
#   $env:GEEK_VERSION     = "v0.1.2"
#   $env:GEEK_INSTALL_DIR = "$env:USERPROFILE\.local\bin"

$ErrorActionPreference = "Stop"

$repo       = "Yangtze-University-Geek-Class/geek-cli"
$version    = if ($env:GEEK_VERSION)     { $env:GEEK_VERSION }     else { "latest" }
$installDir = if ($env:GEEK_INSTALL_DIR) { $env:GEEK_INSTALL_DIR } else { "$env:USERPROFILE\.local\bin" }

function Info($m) { Write-Host "[geek-cli] $m" -ForegroundColor Cyan }
function Ok($m)   { Write-Host "[geek-cli] $m" -ForegroundColor Green }
function Err($m)  { Write-Host "[geek-cli] $m" -ForegroundColor Red }

# -- platform --
$arch = $env:PROCESSOR_ARCHITECTURE
switch ($arch) {
  "AMD64" { $target = "x86_64-pc-windows-msvc" }
  "ARM64" { $target = "aarch64-pc-windows-msvc" }
  default { Err "unsupported arch: $arch"; exit 1 }
}
$binaryName = "geek-$target.exe"

if ($version -eq "latest") {
  $base = "https://github.com/$repo/releases/latest/download"
} else {
  $base = "https://github.com/$repo/releases/download/$version"
}
$url    = "$base/$binaryName"
$shaUrl = "$base/$binaryName.sha256"

# -- download --
New-Item -ItemType Directory -Force -Path $installDir | Out-Null
$dest = Join-Path $installDir "geek.exe"
$tmp  = [System.IO.Path]::GetTempFileName()
Info "downloading $binaryName from $url"
try { Invoke-WebRequest -Uri $url -OutFile $tmp -UseBasicParsing }
catch { Err "download failed: $($_.Exception.Message)"; exit 1 }

# -- verify --
try {
  $want = (Invoke-WebRequest -Uri $shaUrl -UseBasicParsing).Content.Trim().Split()[0]
  $got  = (Get-FileHash -Algorithm SHA256 -Path $tmp).Hash.ToLower()
  if ($got -ne $want.ToLower()) { Err "sha256 mismatch (want $want, got $got)"; exit 1 }
  Ok "sha256 verified"
} catch {
  Info "sha256 verify skipped: $($_.Exception.Message)"
}

Move-Item -Force $tmp $dest
Ok "installed: $dest"

# -- PATH check --
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$installDir*") {
  Info "adding $installDir to user PATH"
  [Environment]::SetEnvironmentVariable("Path", "$userPath;$installDir", "User")
  Info "open a new terminal for PATH to take effect"
}

try { Ok (& $dest --version) } catch { }
Ok "next: geek login"
