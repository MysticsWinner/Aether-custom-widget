#!/usr/bin/env pwsh
#Requires -Version 5.1
<#
.SYNOPSIS
    Aether Full-Stack Launch Script
    Starts the Rust core engine daemon and the TUI dashboard side-by-side.

.DESCRIPTION
    Terminal 1: cargo run -p core_engine   (Aether Runtime daemon)
    Terminal 2: cargo run -p dashboard_tui (Live performance TUI dashboard)

    The script opens two separate PowerShell windows so both run concurrently.
    Press Ctrl+C in either window to shut it down.

.NOTES
    Prerequisites:
      - Rust toolchain (rustup + cargo)  https://rustup.rs
      - Windows 11 (x64 or ARM64)

    Optional (WinUI 3 dashboard):
      - Visual Studio 2022 or .NET 8 SDK
      - Build: cd src_gui\CustomWidget.Dashboard && dotnet build
#>

param(
    [switch]$DaemonOnly,    # Only start the core engine
    [switch]$IncludeTui,    # Optional: start TUI tester helper
    [switch]$DashboardOnly  # Only start the TUI tester helper
)

$ProjectRoot = $PSScriptRoot

# Detect available PowerShell binary (prefer pwsh, fallback to powershell)
$PsExe = if (Get-Command pwsh -ErrorAction SilentlyContinue) { "pwsh" } else { "powershell" }

function Write-Banner {
    Write-Host ""
    Write-Host "  +--------------------------------------------------------------+" -ForegroundColor Cyan
    Write-Host "  |  *  Aether 7.4 -- Next-Gen Desktop Customization Platform   |" -ForegroundColor Cyan
    Write-Host "  |      Design System + Adaptive Visual Platform (v0.7.0)      |" -ForegroundColor Cyan
    Write-Host "  +--------------------------------------------------------------+" -ForegroundColor Cyan
    Write-Host ""
}

function Start-CoreEngine {
    Write-Host "  -> Starting Aether Core Engine Daemon..." -ForegroundColor Green
    Write-Host "    IPC pipe: \\.\pipe\CustomWidgetEngineControlPipe" -ForegroundColor DarkGray
    Write-Host ""

    Start-Process $PsExe -ArgumentList @(
        "-NoExit",
        "-Command",
        "cd '$ProjectRoot'; Write-Host '  [Core Engine Host]' -ForegroundColor Cyan; cargo run -p core_engine"
    ) -WindowStyle Normal
}

function Start-TuiTester {
    Write-Host "  -> Starting Decoupled TUI Tester Helper..." -ForegroundColor Magenta
    Write-Host "    Connecting to IPC pipe for bug testing..." -ForegroundColor DarkGray
    Write-Host ""

    Start-Sleep -Seconds 3

    Start-Process $PsExe -ArgumentList @(
        "-NoExit",
        "-Command",
        "cd '$ProjectRoot'; Write-Host '  [TUI Tester Helper]' -ForegroundColor Magenta; cargo run -p dashboard_tui"
    ) -WindowStyle Normal
}

# -- main ----------------------------------------------------------------------

Write-Banner

# Check cargo is available
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "  [X] ERROR: cargo not found. Install Rust: https://rustup.rs" -ForegroundColor Red
    exit 1
}

Write-Host "  Rust toolchain: $(cargo --version)" -ForegroundColor DarkGray
Write-Host ""

# Ensure everything compiles first
Write-Host "  -> Verifying workspace builds..." -ForegroundColor Yellow
Push-Location $ProjectRoot
$checkResult = cargo check --workspace 2>&1
Pop-Location

if ($LASTEXITCODE -ne 0) {
    Write-Host "  [X] Build check failed. Fix errors above before launching." -ForegroundColor Red
    exit 1
}

Write-Host "  [OK] Workspace compiles cleanly." -ForegroundColor Green
Write-Host ""

if ($DashboardOnly) {
    Start-TuiTester
}
elseif ($IncludeTui) {
    Start-CoreEngine
    Start-TuiTester
}
else {
    # Default: launch core daemon runtime
    Start-CoreEngine

    Write-Host "  [OK] Core Daemon launched in background." -ForegroundColor Green
    Write-Host ""
    Write-Host "  USAGE:" -ForegroundColor Yellow
    Write-Host "    WinUI 3 Management Dashboard (Aether Studio):" -ForegroundColor DarkGray
    Write-Host "      cd src_gui\CustomWidget.Dashboard && dotnet run -p:Platform=x64" -ForegroundColor White
    Write-Host "    Decoupled TUI Tester Helper (Optional Bug Testing):" -ForegroundColor DarkGray
    Write-Host "      cargo run -p dashboard_tui" -ForegroundColor White
    Write-Host ""
}
