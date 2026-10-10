Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public class WinFinder {
    public delegate bool EnumThreadDelegate(IntPtr hWnd, IntPtr lParam);
    [DllImport("user32.dll")]
    public static extern bool EnumThreadWindows(int dwThreadId, EnumThreadDelegate lpfn, IntPtr lParam);
    [DllImport("user32.dll", CharSet = CharSet.Auto)]
    public static extern int GetClassName(IntPtr hWnd, StringBuilder lpClassName, int nMaxCount);
    [DllImport("user32.dll", CharSet = CharSet.Auto)]
    public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);
}
'@

$proc = Get-Process -Name "arbelune-host" -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $proc) {
    Write-Host "No arbelune-host process found."
    return
}

Write-Host "Checking HWNDs for PID: $($proc.Id)"
foreach ($t in $proc.Threads) {
    [WinFinder]::EnumThreadWindows($t.Id, {
        param($h, $l)
        $cls = New-Object System.Text.StringBuilder 256
        $txt = New-Object System.Text.StringBuilder 256
        [WinFinder]::GetClassName($h, $cls, 256) | Out-Null
        [WinFinder]::GetWindowText($h, $txt, 256) | Out-Null
        Write-Host "  HWND: 0x$($h.ToString('X8')) | Class: '$($cls.ToString())' | Text: '$($txt.ToString())'"
        return $true
    }, [IntPtr]::Zero) | Out-Null
}
