# Install AIMF MIME types on Windows
# Run as Administrator

Write-Host "Installing AIMF MIME types on Windows..." -ForegroundColor Green

# Create registry entries for file associations
New-Item -Path "HKCR:.avid" -Force | Out-Null
Set-ItemProperty -Path "HKCR:.avid" -Name "(Default)" -Value "aimf.video"

New-Item -Path "HKCR:.aimg" -Force | Out-Null
Set-ItemProperty -Path "HKCR:.aimg" -Name "(Default)" -Value "aimf.image"

New-Item -Path "HKCR:.aaud" -Force | Out-Null
Set-ItemProperty -Path "HKCR:.aaud" -Name "(Default)" -Value "aimf.audio"

# Create ProgIDs
$progids = @{
    "aimf.video" = "AIMF Video"
    "aimf.image" = "AIMF Image"
    "aimf.audio" = "AIMF Audio"
}

foreach ($progid in $progids.Keys) {
    New-Item -Path "HKCR:$progid" -Force | Out-Null
    Set-ItemProperty -Path "HKCR:$progid" -Name "(Default)" -Value $progids[$progid]
    
    # Add shell command
    New-Item -Path "HKCR:$progid\shell\open\command" -Force | Out-Null
    $aimfPath = (Get-Command aimf -ErrorAction SilentlyContinue).Source
    if ($aimfPath) {
        Set-ItemProperty -Path "HKCR:$progid\shell\open\command" -Name "(Default)" -Value "`"$aimfPath`" view `"%1`""
    }
}

Write-Host "✅ AIMF file associations created" -ForegroundColor Green
Write-Host "Run 'aimf view file.avid' to test" -ForegroundColor Yellow