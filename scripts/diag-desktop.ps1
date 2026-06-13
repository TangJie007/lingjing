Add-Type @"
using System;
using System.Runtime.InteropServices;
using System.Text;
public class Win32 {
  [DllImport("user32.dll")] public static extern IntPtr FindWindow(string c, string n);
  [DllImport("user32.dll")] public static extern IntPtr FindWindowEx(IntPtr p, IntPtr a, string c, string n);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWndProc cb, IntPtr l);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern IntPtr GetParent(IntPtr h);
  public delegate bool EnumWndProc(IntPtr h, IntPtr l);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L,T,R,B; }
}
"@

$worker = [IntPtr]::Zero
$icons = [IntPtr]::Zero
[Win32]::EnumWindows([Win32+EnumWndProc]{ param($h,$l)
  $def = [Win32]::FindWindowEx($h, [IntPtr]::Zero, "SHELLDLL_DefView", $null)
  if ($def -ne [IntPtr]::Zero) {
    $script:icons = $h
    $script:worker = [Win32]::FindWindowEx([IntPtr]::Zero, $h, "WorkerW", $null)
    return $false
  }
  return $true
}, [IntPtr]::Zero) | Out-Null

$progman = [Win32]::FindWindow("Progman", $null)
$r1 = New-Object Win32+RECT
$r2 = New-Object Win32+RECT
[void][Win32]::GetWindowRect($icons, [ref]$r1)
[void][Win32]::GetWindowRect($worker, [ref]$r2)
Write-Host "Progman: 0x$($progman.ToString('X'))"
Write-Host "IconHost: 0x$($icons.ToString('X')) visible=$([Win32]::IsWindowVisible($icons)) rect=$($r1.L),$($r1.T),$($r1.R),$($r1.B)"
Write-Host "WorkerW:  0x$($worker.ToString('X')) visible=$([Win32]::IsWindowVisible($worker)) parent=0x$(([Win32]::GetParent($worker)).ToString('X')) rect=$($r2.L),$($r2.T),$($r2.R),$($r2.B)"
