# Zed Windows 便携版安装脚本
# 解压后运行此脚本可创建桌面快捷方式并添加到 PATH
# 用法: 右键 install.ps1 -> 使用 PowerShell 运行，或在解压目录中执行 .\install.ps1

$ErrorActionPreference = 'Stop'

$scriptDir = $PSScriptRoot
$zedExe = Join-Path $scriptDir "zed.exe"
$cliExe = Join-Path $scriptDir "zed-cli.exe"

if (-not (Test-Path $zedExe)) {
    Write-Host "错误: 未找到 zed.exe，请确保在 Zed 解压目录中运行此脚本。" -ForegroundColor Red
    exit 1
}

# 创建桌面快捷方式
$desktopPath = [Environment]::GetFolderPath("Desktop")
$shortcutPath = Join-Path $desktopPath "Zed.lnk"

try {
    $WshShell = New-Object -ComObject WScript.Shell
    $Shortcut = $WshShell.CreateShortcut($shortcutPath)
    $Shortcut.TargetPath = $zedExe
    $Shortcut.WorkingDirectory = $scriptDir
    $Shortcut.Description = "Zed 编辑器"
    $Shortcut.Save()
    [System.Runtime.Interopservices.Marshal]::ReleaseComObject($WshShell) | Out-Null
    Write-Host "已创建桌面快捷方式: $shortcutPath" -ForegroundColor Green
} catch {
    Write-Host "创建快捷方式失败: $_" -ForegroundColor Yellow
}

# 添加到用户 PATH
$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($currentPath -notlike "*$scriptDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$currentPath;$scriptDir", "User")
    Write-Host "已添加到 PATH，新开终端可使用 zed 命令" -ForegroundColor Green
} else {
    Write-Host "PATH 中已包含 Zed 目录" -ForegroundColor Gray
}

Write-Host ""
Write-Host "安装完成！" -ForegroundColor Green
Write-Host "  - 双击桌面 Zed 图标启动" -ForegroundColor Cyan
Write-Host "  - 或在终端输入 zed 启动（需新开终端）" -ForegroundColor Cyan
