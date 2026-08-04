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
    [switch]$DaemonOnly,    # Only start the core engine (no TUI)
    [switch]$DashboardOnly  # Only start the TUI (daemon already running)
)

$ProjectRoot = $PSScriptRoot

# Detect available PowerShell binary (prefer pwsh, fallback to powershell)
$PsExe = if (Get-Command pwsh -ErrorAction SilentlyContinue) { "pwsh" } else { "powershell" }

function Write-Banner {
    Write-Host ""
    Write-Host "  +--------------------------------------------------------------+" -ForegroundColor Cyan
    Write-Host "  |  *  Aether -- Next-Gen Windows Desktop Customization         |" -ForegroundColor Cyan
    Write-Host "  |      Phase 15 Production Release Candidate                   |" -ForegroundColor Cyan
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
        "cd '$ProjectRoot'; Write-Host '  [Core Engine]' -ForegroundColor Cyan; cargo run -p core_engine"
    ) -WindowStyle Normal
}

function Start-Dashboard {
    Write-Host "  -> Starting Aether TUI Dashboard..." -ForegroundColor Magenta
    Write-Host "    Connecting to IPC pipe (will retry until core engine is ready)..." -ForegroundColor DarkGray
    Write-Host ""

    # Wait a moment for the daemon to start before opening the dashboard
    Start-Sleep -Seconds 3

    Start-Process $PsExe -ArgumentList @(
        "-NoExit",
        "-Command",
        "cd '$ProjectRoot'; Write-Host '  [TUI Dashboard]' -ForegroundColor Magenta; cargo run -p dashboard_tui"
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

if ($DaemonOnly) {
    Start-CoreEngine
    Write-Host "  Core Engine started. Connect the dashboard separately with:" -ForegroundColor Cyan
    Write-Host "  cargo run -p dashboard_tui" -ForegroundColor White
}
elseif ($DashboardOnly) {
    Start-Dashboard
}
else {
    # Start both
    Start-CoreEngine
    Start-Dashboard

    Write-Host "  [OK] Both components launched in separate windows." -ForegroundColor Green
    Write-Host ""
    Write-Host "  USAGE:" -ForegroundColor Yellow
    Write-Host "    Core Engine window -> press Ctrl+C to shut down the daemon" -ForegroundColor DarkGray
    Write-Host "    Dashboard window   -> press 'q' or Ctrl+C to close the TUI" -ForegroundColor DarkGray
    Write-Host ""
    Write-Host "  WinUI 3 Dashboard (Aether Studio):" -ForegroundColor Yellow
    Write-Host "    cd src_gui\CustomWidget.Dashboard" -ForegroundColor DarkGray
    Write-Host "    dotnet run -p:Platform=x64" -ForegroundColor DarkGray
    Write-Host ""
}
