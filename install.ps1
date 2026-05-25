# PowerShell one-liner: iex (irm https://aimf.io/install.ps1)

$version = "latest"
$url = "https://github.com/ai-mf/media-engine/releases/${version}/download/aimf-windows-x86_64.exe"
$output = "$env:USERPROFILE\aimf.exe"

Write-Host "⬇️ Downloading AIMF..." -ForegroundColor Cyan
Invoke-WebRequest -Uri $url -OutFile $output

# Add to PATH
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
[Environment]::SetEnvironmentVariable("Path", "$userPath;$env:USERPROFILE", "User")

Write-Host "✅ AIMF installed!" -ForegroundColor Green
& $output --version