#Requires -Version 5.1
<#
.SYNOPSIS
    Start a craft app. VENDORED from mcp-central-docs\templates\craft-app (identical in every
    craft repo); per-repo settings are in scripts\craft.config.psd1.

.DESCRIPTION
    Default: incremental cargo build of the desktop app, then run it, so the window always
    runs the current code (an up-to-date build takes a second or two).
    If the app is already running, it asks whether to kill that window. No leaves it
    running and brings it to the front. Yes closes it, then builds and starts this copy.

.PARAMETER Release
    Optimized build (target\release).
.PARAMETER NoBuild
    Run the last build as it is.
.PARAMETER New
    Start another window even if one is running.
.PARAMETER Smoke
    Headless check on a throwaway database (no window): app smoke and CLI smoke from the config.
.PARAMETER Mcp
    Run the MCP stdio server (the CLI binary).
.PARAMETER Cli
    Run the CLI with the remaining arguments.

.EXAMPLE
    .\start.ps1
    .\start.ps1 -Release --dark --lang en
    .\start.ps1 -Smoke
    .\start.ps1 -Cli status
#>
param(
    [switch]$Release,
    [switch]$NoBuild,
    [switch]$New,
    [switch]$Smoke,
    [switch]$Mcp,
    [switch]$Cli,
    [Parameter(ValueFromRemainingArguments = $true)][string[]]$Rest
)

$ErrorActionPreference = 'Continue'
$RepoRoot = $PSScriptRoot
. (Join-Path $RepoRoot 'scripts\craft-common.ps1')
Initialize-CraftCargo
$cfg = Get-CraftConfig $RepoRoot
Set-Location -LiteralPath $RepoRoot
if ($null -eq $Rest) { $Rest = @() }
$mode = if ($Release) { 'release' } else { 'debug' }
$targetDir = Get-CraftTargetDir $RepoRoot
function Exe([string]$bin) { Join-Path (Join-Path $targetDir $mode) "$bin.exe" }

function Build([string]$pkg, [string]$bin) {
    if ($NoBuild) { return }
    $exe = Exe $bin
    if (Get-Process -Name $bin -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $exe }) {
        Write-Host "[$($cfg.Name)] $bin is running from $exe, so it cannot be rebuilt now: using that build" -ForegroundColor Yellow
        return
    }
    Write-Host "[$($cfg.Name)] building $bin ($mode)..." -ForegroundColor Cyan
    $code = Invoke-CraftBuild -Package $pkg -Bin $bin -Release:$Release
    if ($code -ne 0) { Write-Host "[$($cfg.Name)] build failed (exit $code)" -ForegroundColor Red; exit $code }
}

function Run([string]$bin, [object[]]$argList) {
    $exe = Exe $bin
    if (-not (Test-Path -LiteralPath $exe)) { Write-Host "[$($cfg.Name)] not built: $exe (drop -NoBuild)" -ForegroundColor Red; exit 1 }
    & $exe @argList
    exit $LASTEXITCODE
}

if ($Mcp) {
    if (-not $cfg.McpBin) { Write-Host "[$($cfg.Name)] this app has no MCP binary (scripts\craft.config.psd1)" -ForegroundColor Red; exit 1 }
    Build $cfg.McpPackage $cfg.McpBin
    Write-Host "[$($cfg.Name)] MCP stdio server; Ctrl+C to stop" -ForegroundColor Cyan
    Run $cfg.McpBin (@($cfg.McpArgs) + $Rest)
}
if ($Cli) {
    if (-not $cfg.CliBin) { Write-Host "[$($cfg.Name)] this app has no CLI binary (scripts\craft.config.psd1)" -ForegroundColor Red; exit 1 }
    Build $cfg.CliPackage $cfg.CliBin
    Run $cfg.CliBin $Rest
}

if ($Smoke) {
    $db = Join-Path ([IO.Path]::GetTempPath()) "$($cfg.Name)-start-smoke-$PID.db"
    $failed = 0
    if ($cfg.AppBin -and @($cfg.AppSmoke).Count -gt 0) {
        Build $cfg.AppPackage $cfg.AppBin
        & (Exe $cfg.AppBin) @(Expand-CraftArgs $cfg.AppSmoke $db)
        if ($LASTEXITCODE -ne 0) { $failed = $LASTEXITCODE; Write-Host "[$($cfg.Name)] app smoke failed" -ForegroundColor Red }
    }
    if ($cfg.CliBin -and @($cfg.CliSmoke).Count -gt 0) {
        Build $cfg.CliPackage $cfg.CliBin
        & (Exe $cfg.CliBin) @(Expand-CraftArgs $cfg.CliSmoke $db)
        if ($LASTEXITCODE -ne 0) { $failed = $LASTEXITCODE; Write-Host "[$($cfg.Name)] cli smoke failed" -ForegroundColor Red }
    }
    Remove-Item "$db*" -Force -ErrorAction SilentlyContinue
    if (@($cfg.AppSmoke).Count -eq 0 -and @($cfg.CliSmoke).Count -eq 0) {
        if ($cfg.AppBin) { Build $cfg.AppPackage $cfg.AppBin }
        if ($cfg.CliBin) { Build $cfg.CliPackage $cfg.CliBin }
        Write-Host "[$($cfg.Name)] built; no smoke configured (AppSmoke / CliSmoke in scripts\craft.config.psd1)" -ForegroundColor Yellow
        exit 0
    }
    if ($failed -eq 0) { Write-Host "[$($cfg.Name)] smoke ok" -ForegroundColor Green }
    exit $failed
}

if (-not $cfg.AppBin) {
    Write-Host "[$($cfg.Name)] no desktop app configured; running the CLI" -ForegroundColor Yellow
    Build $cfg.CliPackage $cfg.CliBin
    Run $cfg.CliBin $Rest
}

function Show-CraftForegroundWindow {
    param($Process)
    $Process.Refresh()
    $hwnd = $Process.MainWindowHandle
    if ($hwnd -eq [IntPtr]::Zero) { return $false }
    if (-not ('CraftWindow' -as [type])) {
        Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class CraftWindow {
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
    [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool BringWindowToTop(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, IntPtr processId);
    [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
    [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint idAttach, uint idAttachTo, bool fAttach);
}
'@
    }
    if ([CraftWindow]::IsIconic($hwnd)) { [void][CraftWindow]::ShowWindow($hwnd, 9) }
    $front = [CraftWindow]::GetForegroundWindow()
    $frontThread = [CraftWindow]::GetWindowThreadProcessId($front, [IntPtr]::Zero)
    $thisThread = [CraftWindow]::GetCurrentThreadId()
    $attached = $false
    if ($frontThread -ne $thisThread) {
        $attached = [CraftWindow]::AttachThreadInput($thisThread, $frontThread, $true)
    }
    [void][CraftWindow]::BringWindowToTop($hwnd)
    [void][CraftWindow]::ShowWindow($hwnd, 9)
    $ok = [CraftWindow]::SetForegroundWindow($hwnd)
    if ($attached) { [void][CraftWindow]::AttachThreadInput($thisThread, $frontThread, $false) }
    return $ok
}

$running = @(Get-Process -Name $cfg.AppBin -ErrorAction SilentlyContinue)
if ($running.Count -gt 0 -and -not $New) {
    $open = $null
    foreach ($proc in $running) {
        $proc.Refresh()
        if ($proc.MainWindowHandle -ne [IntPtr]::Zero) { $open = $proc }
    }
    if (-not $open) { $open = $running[0] }
    $title = $open.MainWindowTitle
    if (-not $title) { $title = '(no window title)' }
    Write-Host ""
    Write-Host "[$($cfg.Name)] already running." -ForegroundColor Yellow
    Write-Host "[$($cfg.Name)] PID $($open.Id), started $($open.StartTime), title: $title" -ForegroundColor Yellow
    $answer = Read-Host "[$($cfg.Name)] Kill it and start this build? [y/N]"
    if ($answer -eq 'y' -or $answer -eq 'Y' -or $answer -eq 'yes' -or $answer -eq 'YES') {
        Write-Host "[$($cfg.Name)] closing PID $($open.Id) and the other $($cfg.AppBin) processes." -ForegroundColor Yellow
        foreach ($proc in $running) { Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue }
        Start-Sleep -Seconds 1
    } else {
        $brought = Show-CraftForegroundWindow $open
        if ($brought) {
            Write-Host "[$($cfg.Name)] left the open window in front. Nothing was killed." -ForegroundColor Green
            exit 0
        }
        Write-Host "[$($cfg.Name)] could not bring PID $($open.Id) to the front. It is still running. Nothing was killed." -ForegroundColor Red
        exit 1
    }
}

Build $cfg.AppPackage $cfg.AppBin
Write-Host "[$($cfg.Name)] starting $(Exe $cfg.AppBin)" -ForegroundColor Cyan
Run $cfg.AppBin (@($cfg.AppArgs) + $Rest)
