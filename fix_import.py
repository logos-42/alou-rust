#!/usr/bin/env python3
"""临时注释 useToolCallHandler 导入"""

with open('/Users/apple/Downloads/alou/alou-desktop/src/components/AgentChat.jsx', 'r') as f:
    content = f.read()

# 注释导入
content = content.replace(
    "import { useToolCallHandler } from '@/hooks/useAgentChat'",
    "// import { useToolCallHandler } from '@/hooks/useAgentChat'  // 临时注释"
)

with open('/Users/apple/Downloads/alou/alou-desktop/src/components/AgentChat.jsx', 'w') as f:
    f.write(content)

print("已注释 useToolCallHandler 导入")
