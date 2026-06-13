Add-Type @"
using System;
using System.Runtime.InteropServices;
using System.Text;
public class Win32 {
  [DllImport("user32.dll")] public static extern IntPtr FindWindowEx(IntPtr p, IntPtr a, string c, string n);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWndProc cb, IntPtr l);
  [DllImport("user32.dll")] public static extern bool EnumChildWindows(IntPtr p, EnumWndProc cb, IntPtr l);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetClassName(IntPtr h, StringBuilder s, int max);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  public delegate bool EnumWndProc(IntPtr h, IntPtr l);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L,T,R,B; }
}
"@

$shellHost = [IntPtr]::Zero
$shellView = [IntPtr]::Zero
[Win32]::EnumWindows([Win32+EnumWndProc]{ param($h,$l)
  $def = [Win32]::FindWindowEx($h, [IntPtr]::Zero, "SHELLDLL_DefView", $null)
  if ($def -ne [IntPtr]::Zero) { $script:shellHost = $h; $script:shellView = $def; return $false }
  return $true
}, [IntPtr]::Zero) | Out-Null

function Show-H($label, $h) {
  if ($h -eq [IntPtr]::Zero) { return }
  $r = New-Object Win32+RECT; [void][Win32]::GetWindowRect($h, [ref]$r)
  $cls = New-Object System.Text.StringBuilder 64; [void][Win32]::GetClassName($h, $cls, 64)
  Write-Host ("{0}: 0x{1:X} class={2} vis={3} rect={4},{5},{6},{7}" -f $label, $h.ToInt64(), $cls, [Win32]::IsWindowVisible($h), $r.L,$r.T,$r.R,$r.B)
}

Show-H "shell_host" $shellHost
Show-H "shell_view" $shellView
Write-Host "--- children of shell_host ---"
[Win32]::EnumChildWindows($shellHost, [Win32+EnumWndProc]{ param($h,$l)
  Show-H "  child" $h
  return $true
}, [IntPtr]::Zero) | Out-Null

$mpvProcs = Get-Process mpv -ErrorAction SilentlyContinue
foreach ($p in $mpvProcs) {
  Write-Host "--- mpv pid $($p.Id) ---"
  [Win32]::EnumWindows([Win32+EnumWndProc]{ param($h,$l)
    $pid = 0; [void][Win32]::GetWindowThreadProcessId($h, [ref]$pid)
    if ($pid -eq $script:targetPid) { Show-H "  mpv-win" $h }
    return $true
  }, [IntPtr]::Zero) | Out-Null
}
