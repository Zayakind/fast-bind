# Simple build script for Notes app

Write-Host "Building Notes for current platform..." -ForegroundColor Green

# Create releases directory
$ReleaseDir = "releases"
if (Test-Path $ReleaseDir) {
    Remove-Item $ReleaseDir -Recurse -Force
}
New-Item -ItemType Directory -Path $ReleaseDir | Out-Null

# Platform settings
$PlatformName = "windows-x64"
$Extension = ".exe"

Write-Host "Platform: $PlatformName" -ForegroundColor Blue

# Build release version
Write-Host "Building release version..." -ForegroundColor Yellow
cargo build --release

if ($LASTEXITCODE -eq 0) {
    Write-Host "Build successful!" -ForegroundColor Green
    
    # Create platform directory
    $PlatformDir = Join-Path $ReleaseDir $PlatformName
    New-Item -ItemType Directory -Path $PlatformDir | Out-Null
    
    # Copy executable
    $BinaryName = "Notes$Extension"
    $SourcePath = Join-Path (Join-Path "target" "release") $BinaryName
    $DestPath = Join-Path $PlatformDir $BinaryName
    
    if (Test-Path $SourcePath) {
        Copy-Item $SourcePath $DestPath
        Write-Host "File copied: $DestPath" -ForegroundColor Cyan
        
        # Create archive
        $ArchiveName = "Notes-$PlatformName.zip"
        $ArchivePath = Join-Path $ReleaseDir $ArchiveName
        Compress-Archive -Path $PlatformDir -DestinationPath $ArchivePath -Force
        Write-Host "Archive created: $ArchivePath" -ForegroundColor Cyan
        
        # Show file information
        $FileInfo = Get-Item $DestPath
        $SizeMB = [math]::Round($FileInfo.Length / 1MB, 2)
        Write-Host "Executable size: $SizeMB MB" -ForegroundColor White
        
        $ArchiveInfo = Get-Item $ArchivePath
        $ArchiveSizeMB = [math]::Round($ArchiveInfo.Length / 1MB, 2)
        Write-Host "Archive size: $ArchiveSizeMB MB" -ForegroundColor White
        
        Write-Host ""
        Write-Host "Build completed successfully!" -ForegroundColor Green
        Write-Host "Files are in: $ReleaseDir" -ForegroundColor Cyan
        
    } else {
        Write-Host "File not found: $SourcePath" -ForegroundColor Red
    }
} else {
    Write-Host "Build failed" -ForegroundColor Red
}

Write-Host ""
Write-Host "For cross-compilation to other platforms:" -ForegroundColor Cyan
Write-Host "1. Install Docker" -ForegroundColor White
Write-Host "2. Run: .\build-all-platforms.ps1" -ForegroundColor White 