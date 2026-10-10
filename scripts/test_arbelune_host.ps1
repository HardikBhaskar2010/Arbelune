$binDir = Resolve-Path "$PSScriptRoot\..\target\x86_64-pc-windows-msvc\debug"
$exePath = Join-Path $binDir "arbelune-host.exe"

$p = New-Object System.Diagnostics.Process
$p.StartInfo.FileName = $exePath
$p.StartInfo.WorkingDirectory = $binDir
$p.StartInfo.UseShellExecute = $false
$p.StartInfo.RedirectStandardOutput = $true
$p.StartInfo.RedirectStandardError = $true
$p.StartInfo.EnvironmentVariables["RUST_LOG"] = "info"

$p.Start() | Out-Null
Start-Sleep -Seconds 2

if ($p.HasExited) {
    Write-Host "Exited with code: $($p.ExitCode)"
    Write-Host "--- STDOUT ---"
    Write-Host ($p.StandardOutput.ReadToEnd())
    Write-Host "--- STDERR ---"
    Write-Host ($p.StandardError.ReadToEnd())
} else {
    Write-Host "arbelune-host is RUNNING! PID: $($p.Id)"
}
