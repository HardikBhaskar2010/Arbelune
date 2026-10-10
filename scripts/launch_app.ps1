param(
    [switch]$CheckOnly
)

$binDir = Resolve-Path "$PSScriptRoot\..\target\x86_64-pc-windows-msvc\debug"
$exePath = Join-Path $binDir "arbelune-host.exe"
if (-not (Test-Path $exePath)) {
    $exePath = Join-Path $binDir "kage-host.exe"
}

if ($CheckOnly) {
    Get-Process -Name *arbelune*, *kage* -ErrorAction SilentlyContinue | Select-Object Id, ProcessName, WorkingSet64
    return
}

Write-Host "Launching Arbelune from: $exePath" -ForegroundColor Cyan
$proc = Start-Process -FilePath $exePath -WorkingDirectory $binDir -PassThru

Start-Sleep -Seconds 2

if ($proc -and -not $proc.HasExited) {
    Write-Host "Arbelune is active on desktop! Host PID: $($proc.Id)" -ForegroundColor Green
    Get-Process -Name *arbelune*, *kage* -ErrorAction SilentlyContinue | Select-Object Id, ProcessName, @{Name="MemoryMB"; Expression={[math]::Round($_.WorkingSet64 / 1MB, 1)}}
} else {
    Write-Host "Process terminated. Exit code: $($proc.ExitCode)" -ForegroundColor Red
}
