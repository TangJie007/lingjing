Add-Type @"
using System;
using System.Runtime.InteropServices;
using System.Text;
public class Win32 {
  [DllImport("user32.dll")] public static extern IntPtr FindWindow(string c, string n);
  [DllImport("user32.dll")] public static extern IntPtr FindWindowEx(IntPtr p, IntPtr a, string c, string n);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWndProc cb, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern bool EnumChildWindows(IntPtr p, EnumWndProc cb, IntPtr l);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern IntPtr GetParent(IntPtr h);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetClassName(IntPtr h, StringBuilder s, int max);
  [DllImport("user32.dll")] public static extern int GetSystemMetrics(int i);
  public delegate bool EnumWndProc(IntPtr h, IntPtr l);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L,T,R,B; }
}
"@

function Show-Hwnd($prefix, $h) {
  if ($h -eq [IntPtr]::Zero) { Write-Host "$prefix <null>"; return }
  $r = New-Object Win32+RECT
  [void][Win32]::GetWindowRect($h, [ref]$r)
  $cls = New-Object System.Text.StringBuilder 64
  [void][Win32]::GetClassName($h, $cls, 64)
  $par = [Win32]::GetParent($h)
  Write-Host ("$prefix 0x{0:X} class={1} vis={2} parent=0x{3:X} rect={4},{5},{6},{7}" -f $h.ToInt64(), $cls, [Win32]::IsWindowVisible($h), $par.ToInt64(), $r.L,$r.T,$r.R,$r.B)
}

Write-Host "Virtual screen: $($([Win32]::GetSystemMetrics(78))),$($([Win32]::GetSystemMetrics(79))) size $($([Win32]::GetSystemMetrics(78)+[Win32]::GetSystemMetrics(76)))x$($([Win32]::GetSystemMetrics(79)+[Win32]::GetSystemMetrics(77)))"
Write-Host "Primary screen: $($([Win32]::GetSystemMetrics(0)))x$($([Win32]::GetSystemMetrics(1)))"

[Win32]::EnumWindows([Win32+EnumWndProc]{ param($h,$l)
  $cls = New-Object System.Text.StringBuilder 64
  [void][Win32]::GetClassName($h, $cls, 64)
  if ($cls.ToString() -eq "WorkerW") { Show-Hwnd "Top WorkerW" $h }
  return $true
}, [IntPtr]::Zero) | Out-Null

$progman = [Win32]::FindWindow("Progman", $null)
Show-Hwnd "Progman" $progman
[Win32]::EnumChildWindows($progman, [Win32+EnumWndProc]{ param($h,$l)
  $cls = New-Object System.Text.StringBuilder 64
  [void][Win32]::GetClassName($h, $cls, 64)
  Show-Hwnd ("  Progman child " + $cls) $h
  return $true
}, [IntPtr]::Zero) | Out-Null
