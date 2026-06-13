<#PSScriptInfo
.VERSION 0.1.0
.GUID 8f3e2a1b-4c6d-4e8f-9a2b-7c1d3e5f6a8b
.AUTHOR AgenticBox
.COMPANYNAME AgenticBox
.COPYRIGHT (c) 2025 AgenticBox. MIT OR Apache-2.0.
.TAGS agenticbox, installer, ai, agents
.LICENSEURI https://github.com/agenticbox/agenticbox/blob/main/LICENSE-MIT
.PROJECTURI https://github.com/agenticbox/agenticbox
#>

<#PSScriptInfo
.SYNOPSIS
    AgenticBox one-line installer for Windows PowerShell

.DESCRIPTION
    Installs AgenticBox daemon and CLI. Requires Rust and Docker Desktop.
    Usage: irm https://agenticbox.co/install.ps1 | iex

.NOTES
    Requires PowerShell 5.1+ or PowerShell Core 6+
#>

# Require admin for some operations, but don't fail if not
if (-not ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Write-Warning "Not running as Administrator. Some operations may require elevation."
}

# Colors
$RED  = "$([char]27)[31m"
$GREEN = "$([char]27)[32m"
$YELLOW = "$([char]27)[33m"
$BLUE = "$([char]27)[34m"
$MAGENTA = "$([char]27)[35m"
$CYAN = "$([char]27)[36m"
$BOLD = "$([char]27)[1m"
$DIM = "$([char]27)[2m"
$RESET = "$([char]27)[0m"

function Write-Step { param([string]$Msg) Write-Host "${CYAN}▶${RESET} ${BOLD}$Msg${RESET}" }
function Write-Ok { param([string]$Msg) Write-Host "${GREEN}✓${RESET} $Msg" }
function Write-Warn { param([string]$Msg) Write-Host "${YELLOW}⚠${RESET} $Msg" }
function Write-Err { param([string]$Msg) Write-Host "${RED}✗${RESET} $Msg" }
function Write-Info { param([string]$Msg) Write-Host "${DIM}$Msg${RESET}" }

function Show-Header {
    Write-Host "${MAGENTA}"
    @"
    █████╗ ██████╗ ██╗   ██╗███████╗██████╗ ██████╗  ██████╗ ██████╗ ███████╗
   ██╔══██╗██╔══██╗██║   ██║██╔════╝██╔══██╗██╔══██╗██╔═══██╗██╔══██╗██╔════╝
   ███████║██████╔╝██║   ██║█████╗  ██████╔╝██████╔╝██║   ██║██████╔╝███████╗
   ██═══██║██╔══██╗╚██╗ ██═╝██═══╝  ██╔══██╗██═══██╗██║   ██║██═══██╗╚════██║
   ██║  ██║██║  ██║ ╚████═╝ ███████╗██║  ██║██║  ██║╚██████═╝██║  ██║███████║
   ╚═╝  ╚═╝╚═╝  ╚═╝  ╚═══╝  ╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚══════╝
"@
    Write-Host "${RESET}"
    Write-Host "${BOLD}Governance Layer for AI Agents${RESET}"
    Write-Host "${DIM}Open source • Local-first • Rust + Tauri${RESET}`n"
}

function Detect-Arch {
    $arch = [Environment]::GetEnvironmentVariable("PROCESSOR_ARCHITECTURE")
    if ($arch -eq "AMD64") { return "x86_64" }
    if ($arch -eq "ARM64") { return "aarch64" }
    Write-Err "Unsupported architecture: $arch"
    exit 1
}

function Test-Command { param([string]$Name) (Get-Command $Name -ErrorAction SilentlyContinue) -ne $null }

function Install-Rust {
    if (Test-Command cargo) {
        $version = cargo --version | ForEach-Object { $_ -replace '^cargo\s+', '' -replace '\s+\(.*$', '' }
        Write-Ok "Rust already installed ($version)"
        return
    }
    Write-Step "Installing Rust via rustup..."
    $installer = "$env:TEMP\rustup-init.exe"
    try {
        Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $installer -UseBasicParsing
        & $installer -y --quiet
        Remove-Item $installer -Force
        # Refresh PATH
        $env:PATH = [Environment]::GetEnvironmentVariable("PATH", "User") + ";" + [Environment]::GetEnvironmentVariable("PATH", "Machine")
        Write-Ok "Rust installed"
    } catch {
        Write-Err "Rust installation failed: $_"
        exit 1
    }
}

function Check-Docker {
    if (Test-Command docker) {
        Write-Ok "Docker already installed"
        return $true
    }
    Write-Warn "Docker Desktop not found. Required for sandbox execution."
    Write-Info "Install: https://www.docker.com/products/docker-desktop/"
    $choice = Read-Host "Continue anyway? [y/N]"
    if ($choice -notmatch '^[Yy]$') { exit 1 }
    return $false
}

function Fetch-Release {
    param([string]$Arch)
    Write-Step "Fetching latest release..."
    $api = "https://api.github.com/repos/agenticbox/agenticbox/releases/latest"
    try {
        $release = Invoke-RestMethod -Uri $api -Headers @{ "Accept" = "application/vnd.github.v3+json" }
        $tag = $release.tag_name
        Write-Ok "Latest release: $tag"

        $assetName = "agenticbox-${tag}-windows-${Arch}.tar.gz"
        $url = "https://github.com/agenticbox/agenticbox/releases/download/${tag}/${assetName}"
        $dest = "$env:USERPROFILE\.agenticbox\${assetName}"

        Write-Step "Downloading $assetName..."
        New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\.agenticbox" | Out-Null
        Invoke-WebRequest -Uri $url -OutFile $dest -UseBasicParsing
        Write-Ok "Downloaded"

        Write-Step "Extracting..."
        tar -xzf $dest -C "$env:USERPROFILE\.agenticbox"
        Remove-Item $dest -Force
        Write-Ok "Extracted"
        return $true
    } catch {
        Write-Warn "Release download failed: $_"
        Write-Info "Falling back to building from source..."
        return $false
    }
}

function Build-FromSource {
    Write-Step "Building from source (requires Rust + Docker Desktop)..."
    Install-Rust
    Check-Docker

    $tmp = [IO.Path]::GetTempPath() + "agenticbox-build-" + [Guid]::NewGuid().ToString("N")[0..7]
    New-Item -ItemType Directory -Force -Path $tmp | Out-Null

    Write-Step "Cloning repository..."
    git clone --depth 1 "https://github.com/agenticbox/agenticbox.git" $tmp

    Write-Step "Building release binaries..."
    Set-Location $tmp
    cargo build --release --bin daemon --bin agenticbox
    Set-Location $env:USERPROFILE

    $binDir = "$env:USERPROFILE\.agenticbox\bin"
    New-Item -ItemType Directory -Force -Path $binDir | Out-Null
    Copy-Item "$tmp\target\release\daemon.exe" "$binDir\daemon.exe" -Force
    Copy-Item "$tmp\target\release\agenticbox.exe" "$binDir\agenticbox.exe" -Force

    Remove-Item $tmp -Recurse -Force
    Write-Ok "Built and installed"
}

function Install-Binaries {
    Write-Step "Installing binaries..."
    $binDir = "$env:USERPROFILE\.agenticbox\bin"
    New-Item -ItemType Directory -Force -Path $binDir | Out-Null

    $srcDir = "$env:USERPROFILE\.agenticbox"
    if (Test-Path "$srcDir\daemon.exe" -and Test-Path "$srcDir\agenticbox.exe") {
        Move-Item "$srcDir\daemon.exe" "$binDir\daemon.exe" -Force
        Move-Item "$srcDir\agenticbox.exe" "$binDir\agenticbox.exe" -Force
    }
    Write-Ok "Binaries installed to $binDir"
}

function Setup-Path {
    $binDir = "$env:USERPROFILE\.agenticbox\bin"
    $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")

    if ($currentPath -like "*$binDir*") {
        Write-Ok "PATH already configured"
        return
    }

    $newPath = "$binDir;$currentPath"
    [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
    $env:PATH = $newPath
    Write-Ok "Added $binDir to User PATH"
    Write-Info "Restart your shell or run: \$env:PATH = [Environment]::GetEnvironmentVariable('PATH','User')"
}

function Verify-Install {
    Write-Step "Verifying installation..."
    $binDir = "$env:USERPROFILE\.agenticbox\bin"
    if (Test-Path "$binDir\agenticbox.exe") {
        $version = & "$binDir\agenticbox.exe" --version 2>$null
        Write-Ok "agenticbox CLI: $version"
    } else {
        Write-Warn "CLI not found. Restart shell."
    }
}

# Main
Show-Header
$arch = Detect-Arch
Install-Rust
Check-Docker

$downloaded = Fetch-Release -Arch $arch
if (-not $downloaded) { Build-FromSource }

Install-Binaries
Setup-Path
Verify-Install

Write-Host ""
Write-Host "${GREEN}${BOLD}Installation complete!${RESET}"
Write-Host ""
Write-Host "${BOLD}Next steps:${RESET}"
Write-Host "  1. ${CYAN}Restart PowerShell${RESET}  (or run: \$env:PATH = [Environment]::GetEnvironmentVariable('PATH','User'))"
Write-Host "  2. ${CYAN}agenticbox setup${RESET}        (configure API keys, providers)"
Write-Host "  3. ${CYAN}agenticbox daemon${RESET}       (start the daemon)"
Write-Host "  4. ${CYAN}agenticbox deploy --name my-agent${RESET}  (run your first agent)"
Write-Host ""
Write-Host "${DIM}Docs: https://agenticbox.co/docs${RESET}"
Write-Host "${DIM}GitHub: https://github.com/agenticbox/agenticbox${RESET}"