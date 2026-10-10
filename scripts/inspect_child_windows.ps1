Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public class WinFinderAll {
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
    [DllImport("user32.dll")]
    public static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);
    [DllImport("user32.dll")]
    public static extern bool EnumChildWindows(IntPtr hWndParent, EnumWindowsProc lpEnumFunc, IntPtr lParam);
    [DllImport("user32.dll")]
    public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);
    [DllImport("user32.dll", SetLastError = true, CharSet = CharSet.Auto)]
    public static extern int GetClassName(IntPtr hWnd, StringBuilder lpClassName, int nMaxCount);
    [DllImport("user32.dll", SetLastError = true, CharSet = CharSet.Auto)]
    public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);
    [DllImport("user32.dll")]
    public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);
    [DllImport("user32.dll")]
    public static extern bool IsWindowVisible(IntPtr hWnd);
    [StructLayout(LayoutKind.Sequential)]
    public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }
}
'@

$proc = Get-Process -Name "arbelune-host", "kage-host" -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $proc) {
    Write-Host "No host process found."
    return
}

$targetPid = [uint32]$proc.Id
Write-Host "Searching all windows for PID: $targetPid ($($proc.ProcessName))"

[WinFinderAll]::EnumWindows({
    param($h, $l)
    [uint32]$pid = 0
    [WinFinderAll]::GetWindowThreadProcessId($h, [ref]$pid) | Out-Null
    if ($pid -eq $targetPid) {
        $cls = New-Object System.Text.StringBuilder 256
        $txt = New-Object System.Text.StringBuilder 256
        [WinFinderAll]::GetClassName($h, $cls, 256) | Out-Null
        [WinFinderAll]::GetWindowText($h, $txt, 256) | Out-Null
        $r = New-Object WinFinderAll+RECT
        [WinFinderAll]::GetWindowRect($h, [ref]$r) | Out-Null
        $vis = [WinFinderAll]::IsWindowVisible($h)
        Write-Host "TOP-LEVEL HWND: 0x$($h.ToString('X8')) | Class: '$($cls.ToString())' | Text: '$($txt.ToString())' | Visible: $vis | Rect: [$($r.Left), $($r.Top), Right=$($r.Right), Bottom=$($r.Bottom)]"
        
        # Enumerate children
        [WinFinderAll]::EnumChildWindows($h, {
            param($ch, $cl)
            $cCls = New-Object System.Text.StringBuilder 256
            $cTxt = New-Object System.Text.StringBuilder 256
            [WinFinderAll]::GetClassName($ch, $cCls, 256) | Out-Null
            [WinFinderAll]::GetWindowText($ch, $cTxt, 256) | Out-Null
            $cr = New-Object WinFinderAll+RECT
            [WinFinderAll]::GetWindowRect($ch, [ref]$cr) | Out-Null
            $cVis = [WinFinderAll]::IsWindowVisible($ch)
            Write-Host "  -> CHILD HWND: 0x$($ch.ToString('X8')) | Class: '$($cCls.ToString())' | Text: '$($cTxt.ToString())' | Visible: $cVis | Rect: [$($cr.Left), $($cr.Top), Right=$($cr.Right), Bottom=$($cr.Bottom)]"
            return $true
        }, [IntPtr]::Zero) | Out-Null
    }
    return $true
}, [IntPtr]::Zero) | Out-Null
