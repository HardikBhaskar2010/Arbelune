$binDir = "C:\Users\sneha\Videos\Kage\target\x86_64-pc-windows-msvc\debug"
$kageExe = "$binDir\kage-host.exe"
$arbeluneExe = "$binDir\arbelune-host.exe"

# Terminate existing host instances if running
Get-Process -Name "arbelune-host", "kage-host" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 800

if (Test-Path $kageExe) {
    Copy-Item -Path $kageExe -Destination $arbeluneExe -Force
}

$res = ([wmiclass]"Win32_Process").Create($arbeluneExe, $binDir, $null)
Write-Host "WMI ReturnValue: $($res.ReturnValue), ProcessId: $($res.ProcessId)"

Start-Sleep -Seconds 3

if ($res.ProcessId) {
    $p = Get-Process -Id $res.ProcessId -ErrorAction SilentlyContinue
    if ($p) {
        Write-Host "SUCCESS: Arbelune is running detached on desktop! PID: $($p.Id), Memory: $([math]::Round($p.WorkingSet64 / 1MB, 1)) MB"
    } else {
        Write-Host "Process $($res.ProcessId) exited."
    }
}
