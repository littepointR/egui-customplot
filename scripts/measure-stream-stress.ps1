param(
    [Parameter(Mandatory = $true)][string]$Executable,
    [Parameter(Mandatory = $true)][string]$OutputPrefix,
    [double]$Seconds = 1800
)

$ErrorActionPreference = 'Stop'
$binary = (Resolve-Path -LiteralPath $Executable).Path
$stdout = "$OutputPrefix.csv"
$stderr = "$OutputPrefix.stderr.txt"
$resources = "$OutputPrefix-process.csv"
$metadata = "$OutputPrefix-environment.txt"
foreach ($path in @($stdout, $stderr, $resources, $metadata)) {
    if (Test-Path -LiteralPath $path) { throw "Refusing to overwrite historical evidence: $path" }
}
@(
    "date=$(Get-Date -Format o)",
    "cpu=$((Get-CimInstance Win32_Processor | Select-Object -First 1).Name)",
    "os=$([Environment]::OSVersion.VersionString)",
    "rust=$(rustc --version)",
    "git_head=$(git rev-parse HEAD)",
    "executable=$binary",
    "executable_sha256=$((Get-FileHash -LiteralPath $binary -Algorithm SHA256).Hash)",
    "seconds=$Seconds; process_sample_interval_ms=5000; priority=normal",
    'Headless stress, not the desktop demo; concurrent development on the machine.',
    'Private bytes / working set are process observations, not live allocations or GPU memory.'
) | Set-Content -LiteralPath $metadata -Encoding utf8
$env:CUSTOMPLOT_STRESS_SECONDS = $Seconds.ToString([Globalization.CultureInfo]::InvariantCulture)
$process = Start-Process -FilePath $binary -PassThru -NoNewWindow -RedirectStandardOutput $stdout -RedirectStandardError $stderr
$clock = [Diagnostics.Stopwatch]::StartNew()
'elapsed_s,pid,private_bytes,working_set_bytes,peak_working_set_bytes,cpu_total_ms,handles,threads' | Set-Content -LiteralPath $resources -Encoding utf8
try {
    while (-not $process.WaitForExit(5000)) {
        $process.Refresh()
        @(
            $clock.Elapsed.TotalSeconds.ToString('F3', [Globalization.CultureInfo]::InvariantCulture),
            $process.Id,
            $process.PrivateMemorySize64,
            $process.WorkingSet64,
            $process.PeakWorkingSet64,
            $process.TotalProcessorTime.TotalMilliseconds.ToString('F3', [Globalization.CultureInfo]::InvariantCulture),
            $process.HandleCount,
            $process.Threads.Count
        ) -join ',' | Add-Content -LiteralPath $resources -Encoding utf8
    }
    $process.WaitForExit()
    if ($process.ExitCode -ne 0) { throw "Stress process failed with exit code $($process.ExitCode); see $stderr" }
    "completed=$(Get-Date -Format o); exit_code=$($process.ExitCode); elapsed_s=$($clock.Elapsed.TotalSeconds)" | Add-Content -LiteralPath $metadata -Encoding utf8
    Get-Content -LiteralPath $stdout -Tail 3
} finally {
    if (-not $process.HasExited) { $process.Kill(); $process.WaitForExit() }
    $process.Dispose()
}
