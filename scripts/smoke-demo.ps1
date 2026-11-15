param(
    [string]$Executable = 'target/release/customplot-demo.exe',
    [Parameter(Mandatory = $true)][string]$Output,
    [string[]]$Galleries = @('default', '--scalar', '--statistical')
)

$ErrorActionPreference = 'Stop'
if (Test-Path -LiteralPath $Output) { throw "Refusing to overwrite evidence: $Output" }
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
        if ($process.HasExited -or $process.MainWindowHandle -eq 0 -or !$process.Responding) {
            throw "Gallery $gallery failed to create a responsive window"
        }
        $title = $process.MainWindowTitle
        $close = $process.CloseMainWindow()
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
