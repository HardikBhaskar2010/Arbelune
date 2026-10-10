$binDir = Resolve-Path "$PSScriptRoot\..\target\x86_64-pc-windows-msvc\debug"
$exePath = Join-Path $binDir "kage-host.exe"

$p = New-Object System.Diagnostics.Process
$p.StartInfo.FileName = $exePath
$p.StartInfo.WorkingDirectory = $binDir
$p.StartInfo.UseShellExecute = $false
$p.StartInfo.RedirectStandardOutput = $true
$p.StartInfo.RedirectStandardError = $true
$p.StartInfo.EnvironmentVariables["RUST_LOG"] = "debug"
$p.StartInfo.EnvironmentVariables["RUST_BACKTRACE"] = "1"

$p.Start() | Out-Null
Write-Host "Started host PID: $($p.Id)"

for ($i = 1; $i -le 15; $i++) {
    Start-Sleep -Seconds 1
    if ($p.HasExited) {
        Write-Host "Process exited at second $i with code: $($p.ExitCode)"
        $out = $p.StandardOutput.ReadToEnd()
        $err = $p.StandardError.ReadToEnd()
        Write-Host "--- STDOUT ---"
        Write-Host $out
        Write-Host "--- STDERR ---"
        Write-Host $err
        break
    } else {
        Write-Host "Second $i : Running (HWND: $($p.MainWindowHandle), Title: '$($p.MainWindowTitle)')"
    }
}
