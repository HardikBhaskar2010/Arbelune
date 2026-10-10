Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public class WinFinderAll3 {
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
    [DllImport("user32.dll")]
    public static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);
    [DllImport("user32.dll")]
    public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);
    [DllImport("user32.dll", CharSet = CharSet.Auto)]
    public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);
    [DllImport("user32.dll", CharSet = CharSet.Auto)]
    public static extern int GetClassName(IntPtr hWnd, StringBuilder lpClassName, int nMaxCount);
    [DllImport("user32.dll")]
    public static extern bool IsWindowVisible(IntPtr hWnd);
}
'@

[WinFinderAll3]::EnumWindows({
    param($h, $l)
    if ([WinFinderAll3]::IsWindowVisible($h)) {
        $txt = New-Object System.Text.StringBuilder 256
        $cls = New-Object System.Text.StringBuilder 256
        [WinFinderAll3]::GetWindowText($h, $txt, 256) | Out-Null
        [WinFinderAll3]::GetClassName($h, $cls, 256) | Out-Null
        $title = $txt.ToString().Trim()
        $className = $cls.ToString().Trim()
        if ($title.Length -gt 0) {
            [uint32]$pid = 0
            [WinFinderAll3]::GetWindowThreadProcessId($h, [ref]$pid) | Out-Null
            $p = Get-Process -Id $pid -ErrorAction SilentlyContinue
            Write-Host "HWND: 0x$($h.ToString('X8')) | PID: $pid ($($p.ProcessName)) | Class: '$className' | Title: '$title'"
        }
    }
    return $true
}, [IntPtr]::Zero) | Out-Null
