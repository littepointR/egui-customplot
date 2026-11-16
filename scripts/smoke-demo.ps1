param(
    [string]$Executable = 'target/release/customplot-demo.exe',
    [Parameter(Mandatory = $true)][string]$Output,
    [string[]]$Galleries = @('default', '--scalar', '--statistical', '--stairs', '--stems', '--quiver', '--polygons', '--contours', '--functions', '--geometry', '--distributions', '--multivariate', '--polar', '--networks')
)

$ErrorActionPreference = 'Stop'
if (Test-Path -LiteralPath $Output) { throw "Refusing to overwrite evidence: $Output" }
if (-not ('DemoSmokeWin' -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public class DemoSmokeWin {
    public delegate bool EnumProc(IntPtr hWnd, IntPtr lParam);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc lpEnumFunc, IntPtr lParam);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint pid);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetWindowText(IntPtr hWnd, StringBuilder str, int maxCount);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool IsHungAppWindow(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr hWnd, uint msg, IntPtr wParam, IntPtr lParam);
}
'@
}

$binary = (Resolve-Path -LiteralPath $Executable).Path
@(
    "date=$(Get-Date -Format o)",
    "os=$([Environment]::OSVersion.VersionString)",
    "executable=$binary",
    "sha256=$((Get-FileHash -LiteralPath $binary -Algorithm SHA256).Hash)",
    'Window lifecycle smoke only; not visual correctness, GPU timing or FPS.'
) | Set-Content -LiteralPath $Output
foreach ($gallery in $Galleries) {
    $arguments = @{ FilePath = $binary; PassThru = $true }
    if ($gallery -ne 'default') { $arguments.ArgumentList = @($gallery) }
    $process = Start-Process @arguments
    try {
        $idle = $process.WaitForInputIdle(20000)
        # Let the first native frame present before testing responsiveness/close.
        Start-Sleep -Seconds 2
        $process.Refresh()
        $targetPid = [uint32]$process.Id
        $script:smokeHwnd = [IntPtr]::Zero
        $script:smokeTitle = ''
        # Avoid UAC/IME helper windows by matching the visible demo window title.
        [DemoSmokeWin]::EnumWindows({
            param($hwnd, $lParam)
            $winPid = [uint32]0
            [void][DemoSmokeWin]::GetWindowThreadProcessId($hwnd, [ref]$winPid)
            if ($winPid -eq $targetPid -and [DemoSmokeWin]::IsWindowVisible($hwnd)) {
                $buf = New-Object System.Text.StringBuilder 256
                [void][DemoSmokeWin]::GetWindowText($hwnd, $buf, 256)
                $t = $buf.ToString()
                if ($t.StartsWith('egui-customplot')) {
                    $script:smokeHwnd = $hwnd
                    $script:smokeTitle = $t
                    return $false
                }
            }
            return $true
        }, [IntPtr]::Zero) | Out-Null
        if ($process.HasExited -or $script:smokeHwnd -eq [IntPtr]::Zero -or [DemoSmokeWin]::IsHungAppWindow($script:smokeHwnd)) {
            throw "Gallery $gallery failed to create a responsive window"
        }
        $title = $script:smokeTitle
        $close = [DemoSmokeWin]::PostMessage($script:smokeHwnd, 0x0010, [IntPtr]::Zero, [IntPtr]::Zero)
        if (!$close -or !$process.WaitForExit(10000)) { throw "Gallery $gallery failed graceful close" }
        $process.Refresh()
        if ($process.ExitCode -ne 0) { throw "Gallery $gallery exited with $($process.ExitCode)" }
        "gallery=$gallery; input_idle=$idle; title=$title; responsive=true; close_requested=$close; exit_code=$($process.ExitCode)" |
            Add-Content -LiteralPath $Output
    } finally {
        # Only this script's process is eligible for failure cleanup.
        if (!$process.HasExited) { $process.Kill(); $process.WaitForExit() }
        $process.Dispose()
    }
}
'completed=true' | Add-Content -LiteralPath $Output
