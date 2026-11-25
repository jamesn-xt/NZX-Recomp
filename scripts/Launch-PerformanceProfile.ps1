param(
    [Parameter(Mandatory)][string]$GameExe,
    [Parameter(Mandatory)][string]$Config,
    [Parameter(Mandatory)][string]$DxcDirectory,
    [string]$Executable = (Join-Path $PSScriptRoot 'nzx-recomp.exe'),
    [string]$LogDirectory = (Join-Path $PSScriptRoot 'logs'),
    [ValidateRange(30,7200)][int]$Seconds = 1800,
    [switch]$InflightFetch,
    [switch]$CheckOnly
)
$ErrorActionPreference = 'Stop'
$dxil = Join-Path $DxcDirectory 'dxil.dll'
$dxc = Join-Path $DxcDirectory 'dxc.exe'
foreach ($file in @($GameExe,$Config,$Executable,$dxil,$dxc)) {
    if (!(Test-Path -LiteralPath $file -PathType Leaf)) { throw "Missing file: $file" }
}
$Executable = (Resolve-Path -LiteralPath $Executable).Path
$GameExe = (Resolve-Path -LiteralPath $GameExe).Path
$Config = (Resolve-Path -LiteralPath $Config).Path
if ($CheckOnly) { Get-FileHash -LiteralPath $Executable; return }
if (Get-Process nzx-recomp,victory -ErrorAction SilentlyContinue) { throw 'Close the existing runtime instance first.' }
$memory = Get-CimInstance Win32_PerfFormattedData_PerfOS_Memory
if (($memory.CommitLimit-$memory.CommittedBytes) -lt 4GB) { throw 'Less than 4 GiB available commit.' }
New-Item -ItemType Directory -Path $LogDirectory -Force | Out-Null
Get-ChildItem Env:VICTORY_* | Remove-Item
$env:VICTORY_CONFIG = $Config
$env:VICTORY_DXIL = $dxil
$env:VICTORY_PC_DXC = $dxc
$env:VICTORY_LOG = 'victory_gpu=info,warn'
$env:VICTORY_CARD_HASH_WIDE = '1'
$env:VICTORY_TEXTURE_DETILE_WORKERS = '3'
$env:VICTORY_LEAN_WINDOWS = '1'
$env:VICTORY_CARD_REHASH_RUNS = '1'
$env:VICTORY_WIDE_REFRESH = '1'
if ($InflightFetch) { $env:VICTORY_CARD_FETCH_INFLIGHT = '1' }
$prefix = Join-Path $LogDirectory (Get-Date -Format 'yyyyMMdd-HHmmss')
$gameProcess = Start-Process -FilePath $Executable -ArgumentList @('run',('"'+$GameExe+'"'),'--execute','--console','series-s') -WorkingDirectory $PSScriptRoot -WindowStyle Hidden -RedirectStandardOutput "$prefix.stdout.log" -RedirectStandardError "$prefix.stderr.log" -PassThru
$started = Get-Date
Write-Host "Started PID $($gameProcess.Id); logs $prefix.*.log; in-flight fetch=$InflightFetch."
Write-Host 'Focus the game. Enter=A, WASD=movement, IJKL=camera. No scripted input is sent.'
try {
    while (!$gameProcess.HasExited) {
        Start-Sleep -Seconds 5
        $gameProcess.Refresh()
        if ($gameProcess.HasExited) { break }
        $memory = Get-CimInstance Win32_PerfFormattedData_PerfOS_Memory
        if (($memory.CommitLimit-$memory.CommittedBytes) -lt 2GB) { Write-Host 'Stopped: commit guard.'; break }
        if (((Get-Date)-$started).TotalSeconds -ge $Seconds) { Write-Host 'Stopped: time limit.'; break }
    }
} finally {
    $gameProcess.Refresh()
    if (!$gameProcess.HasExited) { $gameProcess.Kill(); $gameProcess.WaitForExit() }
}
