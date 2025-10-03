# Vault Password Manager Installation Script for Windows
# This script installs the Vault CLI tool on Windows

param(
    [string]$InstallDir = "$env:LOCALAPPDATA\Programs\Vault",
    [switch]$Force
)

# Colors for output
$Red = "Red"
$Green = "Green"
$Yellow = "Yellow"
$Blue = "Blue"

function Write-ColorOutput {
    param([string]$Message, [string]$Color = "White")
    Write-Host $Message -ForegroundColor $Color
}

Write-ColorOutput "🔐 Vault Password Manager Installer" $Blue
Write-ColorOutput "====================================" $Blue
Write-Host ""

# Check if running as administrator
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")

if ($isAdmin) {
    Write-ColorOutput "⚠️  Running as Administrator - installing system-wide" $Yellow
    $InstallDir = "$env:ProgramFiles\Vault"
} else {
    Write-ColorOutput "ℹ️  Installing to user directory" $Blue
}

# Check if vault is already installed
$existingVault = Get-Command vault -ErrorAction SilentlyContinue
if ($existingVault -and -not $Force) {
    Write-ColorOutput "⚠️  Vault is already installed at: $($existingVault.Source)" $Yellow
    $response = Read-Host "Do you want to continue and overwrite? (y/N)"
    if ($response -ne "y" -and $response -ne "Y") {
        Write-ColorOutput "Installation cancelled" $Blue
        exit 0
    }
}

# Create install directory
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

Write-ColorOutput "📦 Downloading Vault..." $Blue

# Check if Rust/Cargo is installed
$cargo = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $cargo) {
    Write-ColorOutput "❌ Cargo not found. Please install Rust first:" $Red
    Write-ColorOutput "   Visit: https://rustup.rs/" $Blue
    exit 1
}

# Create temporary directory
$TmpDir = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_ }

try {
    Write-ColorOutput "🔨 Building from source..." $Blue
    
    Set-Location $TmpDir
    git clone "https://github.com/vault/vault" vault-src
    Set-Location "vault-src\app"
    
    Write-ColorOutput "🔧 Compiling (this may take a few minutes)..." $Blue
    cargo build --release
    
    $BinaryPath = "target\release\vault.exe"
    
    # Install binary
    Write-ColorOutput "📋 Installing to $InstallDir..." $Blue
    
    if (-not (Test-Path $BinaryPath)) {
        Write-ColorOutput "❌ Binary not found after build" $Red
        exit 1
    }
    
    Copy-Item $BinaryPath "$InstallDir\vault.exe" -Force
    
    # Add to PATH if not already there
    $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($currentPath -notlike "*$InstallDir*") {
        Write-ColorOutput "📋 Adding to PATH..." $Blue
        $newPath = "$currentPath;$InstallDir"
        [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
        $env:PATH = "$env:PATH;$InstallDir"
    }
    
    # Verify installation
    $vault = Get-Command vault -ErrorAction SilentlyContinue
    if ($vault) {
        $version = & vault --version 2>$null | Select-Object -First 1
        Write-ColorOutput "✅ Vault installed successfully!" $Green
        Write-ColorOutput "   Version: $version" $Green
        Write-ColorOutput "   Location: $($vault.Source)" $Green
    } else {
        Write-ColorOutput "❌ Installation failed - vault command not found" $Red
        Write-ColorOutput "💡 You may need to restart your terminal or add $InstallDir to your PATH manually" $Yellow
        exit 1
    }
    
    Write-Host ""
    Write-ColorOutput "🚀 Quick Start:" $Blue
    Write-ColorOutput "===============" $Blue
    Write-ColorOutput "1. Initialize a vault:    vault init --tenant my-org --admin admin@example.com" $Green
    Write-ColorOutput "2. Login:                 vault login --tenant my-org" $Green
    Write-ColorOutput "3. Store a secret:        vault put my-secret" $Green
    Write-ColorOutput "4. Retrieve a secret:     vault get my-secret" $Green
    Write-Host ""
    Write-ColorOutput "📚 For more information:" $Blue
    Write-ColorOutput "   Documentation: https://vault.dev/docs"
    Write-ColorOutput "   GitHub: https://github.com/vault/vault"
    Write-Host ""
    Write-ColorOutput "🎉 Happy secret managing!" $Green
    
} finally {
    # Cleanup
    Set-Location $env:TEMP
    Remove-Item $TmpDir -Recurse -Force -ErrorAction SilentlyContinue
}