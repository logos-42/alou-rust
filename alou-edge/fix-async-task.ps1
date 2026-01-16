# 修复异步多步骤任务问题

Write-Host "=== 修复异步多步骤任务问题 ===" -ForegroundColor Green

# 步骤 1: 启用 Durable Object 迁移配置
Write-Host "`n步骤 1: 启用 Durable Object 迁移配置..." -ForegroundColor Cyan

$wranglerFile = "wrangler.toml"
$wranglerContent = Get-Content $wranglerFile -Raw

# 取消注释迁移配置
$newContent = $wranglerContent -replace '#\s*\[\[migrations\]\]', '[[migrations]]'
$newContent = $newContent -replace '#\s*tag = "v1"', 'tag = "v1"'
$newContent = $newContent -replace '#\s*new_sqlite_classes = \["AITaskDO"\]', 'new_sqlite_classes = ["AITaskDO"]'

# 检查是否还有注释掉的迁移配置
if ($newContent -match "#\s*注释掉迁移配置") {
    $newContent = $newContent -replace "#\s*注释掉迁移配置.*", ""
}

if ($wranglerContent -ne $newContent) {
    $newContent | Out-File $wranglerFile -Encoding UTF8
    Write-Host "  ✅ 已启用迁移配置" -ForegroundColor Green
} else {
    Write-Host "  ℹ️ 迁移配置已经是启用状态" -ForegroundColor Cyan
}

# 步骤 2: 移除 ai_task.rs 中的直接执行代码
Write-Host "`n步骤 2: 移除直接执行代码，只保留 alarm 机制..." -ForegroundColor Cyan

$aiTaskFile = "src/durable_objects/ai_task.rs"
$aiTaskContent = Get-Content $aiTaskFile -Raw

# 移除直接执行的代码（第 350-352 行）
$oldDirectExec = @'
        // 直接执行任务（测试）
        console_log!("[TEST] Direct task execution");
        let _ = self.execute_task().await;
        
'@

$newDirectExec = @'

'@

if ($aiTaskContent -match "直接执行任务\(测试\)") {
    $newAiTaskContent = $aiTaskContent -replace [regex]::Escape($oldDirectExec), $newDirectExec
    $newAiTaskContent | Out-File $aiTaskFile -Encoding UTF8
    Write-Host "  ✅ 已移除直接执行代码" -ForegroundColor Green
} else {
    Write-Host "  ℹ️ 直接执行代码已经被移除" -ForegroundColor Cyan
}

# 步骤 3: 增加 alarm 延迟时间（从 5000ms 改为 10000ms）
Write-Host "`n步骤 3: 增加 alarm 延迟时间..." -ForegroundColor Cyan

$aiTaskContent = Get-Content $aiTaskFile -Raw
$newAlarmDelay = $aiTaskContent -replace 'let scheduled_time = now_ms \+ 5000;', 'let scheduled_time = now_ms + 10000;'

if ($aiTaskContent -ne $newAlarmDelay) {
    $newAlarmDelay | Out-File $aiTaskFile -Encoding UTF8
    Write-Host "  ✅ 已将 alarm 延迟从 5000ms 增加到 10000ms" -ForegroundColor Green
} else {
    Write-Host "  ℹ️ alarm 延迟已经是 10000ms" -ForegroundColor Cyan
}

# 步骤 4: 验证配置
Write-Host "`n步骤 4: 验证配置..." -ForegroundColor Cyan

# 检查迁移配置
$wranglerContent = Get-Content $wranglerFile -Raw
if ($wranglerContent -match '\[\[migrations\]\]' -and $wranglerContent -match 'new_sqlite_classes = \["AITaskDO"\]') {
    Write-Host "  ✅ 迁移配置正确" -ForegroundColor Green
} else {
    Write-Host "  ❌ 迁移配置不正确" -ForegroundColor Red
}

# 检查是否还有直接执行代码
$aiTaskContent = Get-Content $aiTaskFile -Raw
if ($aiTaskContent -notmatch "直接执行任务\(测试\)") {
    Write-Host "  ✅ 直接执行代码已移除" -ForegroundColor Green
} else {
    Write-Host "  ❌ 直接执行代码仍然存在" -ForegroundColor Red
}

# 检查 alarm 延迟
if ($aiTaskContent -match 'let scheduled_time = now_ms \+ 10000;') {
    Write-Host "  ✅ alarm 延迟为 10000ms" -ForegroundColor Green
} else {
    Write-Host "  ❌ alarm 延迟不正确" -ForegroundColor Red
}

Write-Host "`n=== 下一步操作 ===" -ForegroundColor Green
Write-Host "1. 重新部署: npx wrangler deploy" -ForegroundColor White
Write-Host "2. 查看日志: npx wrangler tail --format pretty" -ForegroundColor White
Write-Host "3. 测试任务并观察 '!!! ALARM ACTIVE !!!' 日志" -ForegroundColor White
Write-Host "4. 如果看到 alarm 触发，说明问题已解决" -ForegroundColor White
Write-Host "`n注意: 前端仍需添加轮询机制来持续查询任务状态。" -ForegroundColor Yellow
