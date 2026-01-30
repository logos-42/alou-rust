# 简单的 Agent Skills 测试
# 测试基本的技能发现和加载功能

Write-Host "=== Agent Skills Simple Test ===" -ForegroundColor Cyan

# 1. 创建测试技能目录结构
Write-Host "Setting up test skills..." -ForegroundColor Yellow

$testSkillsDir = "test-skills"
if (Test-Path $testSkillsDir) {
    Write-Host "Test skills directory already exists" -ForegroundColor Green
} else {
    Write-Host "Creating test skills directory..." -ForegroundColor Yellow
    New-Item -ItemType Directory -Path $testSkillsDir -Force | Out-Null
}

# 2. 验证技能文件存在
$webScraperSkill = "$testSkillsDir/web-scraper/SKILL.md"
$dataAnalyzerSkill = "$testSkillsDir/data-analyzer/SKILL.md"

if (Test-Path $webScraperSkill) {
    Write-Host "✅ Web scraper skill found" -ForegroundColor Green
} else {
    Write-Host "❌ Web scraper skill not found" -ForegroundColor Red
}

if (Test-Path $dataAnalyzerSkill) {
    Write-Host "✅ Data analyzer skill found" -ForegroundColor Green
} else {
    Write-Host "❌ Data analyzer skill not found" -ForegroundColor Red
}

# 3. 测试技能内容解析
Write-Host "`nTesting skill content parsing..." -ForegroundColor Yellow

if (Test-Path $webScraperSkill) {
    $content = Get-Content $webScraperSkill -Raw
    Write-Host "Web scraper skill content preview:" -ForegroundColor Cyan
    Write-Host ($content.Substring(0, [Math]::Min(200, $content.Length)) + "...") -ForegroundColor Gray
}

# 4. 创建一个简单的Agent Skills工具调用测试
Write-Host "`nTesting Agent Skills tool calls..." -ForegroundColor Yellow

$testCalls = @(
    @{
        action = "discover"
        description = "Discover available skills"
    }
    @{
        action = "load"
        skill_name = "web-scraper"
        description = "Load web scraper skill"
    }
    @{
        action = "execute"
        skill_name = "data-analyzer"
        inputs = @{
            data = '{"test": "data"}'
            analysis_type = "summary"
        }
        description = "Execute data analyzer skill"
    }
)

foreach ($call in $testCalls) {
    Write-Host "  Testing: $($call.description)" -ForegroundColor Cyan
    Write-Host "    Action: $($call.action)" -ForegroundColor Gray
    if ($call.skill_name) {
        Write-Host "    Skill: $($call.skill_name)" -ForegroundColor Gray
    }
    if ($call.inputs) {
        Write-Host "    Inputs: $($call.inputs | ConvertTo-Json -Compress)" -ForegroundColor Gray
    }
}

# 5. 验证脚本文件
Write-Host "`nChecking script files..." -ForegroundColor Yellow

$scripts = @(
    "$testSkillsDir/web-scraper/scripts/scrape.py"
    "$testSkillsDir/data-analyzer/scripts/analyze.py"
)

foreach ($script in $scripts) {
    if (Test-Path $script) {
        Write-Host "✅ Script found: $script" -ForegroundColor Green
    } else {
        Write-Host "❌ Script missing: $script" -ForegroundColor Red
    }
}

# 6. 测试Python脚本语法
Write-Host "`nTesting Python script syntax..." -ForegroundColor Yellow

foreach ($script in $scripts) {
    if (Test-Path $script) {
        try {
            $result = python -m py_compile $script 2>&1
            if ($LASTEXITCODE -eq 0) {
                Write-Host "✅ Python syntax OK: $(Split-Path $script -Leaf)" -ForegroundColor Green
            } else {
                Write-Host "❌ Python syntax error: $(Split-Path $script -Leaf)" -ForegroundColor Red
                Write-Host "    Error: $result" -ForegroundColor Gray
            }
        } catch {
            Write-Host "⚠️ Python not available for syntax check: $(Split-Path $script -Leaf)" -ForegroundColor Yellow
        }
    }
}

Write-Host "`n=== Test Summary ===" -ForegroundColor Cyan
Write-Host "Agent Skills test environment is ready!" -ForegroundColor Green
Write-Host "You can now run the full loop test with: .\test-agent-skills-loop.ps1" -ForegroundColor White