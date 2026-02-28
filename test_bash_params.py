#!/usr/bin/env python3
"""测试 bash 参数格式"""

import json

# 测试不同的参数格式
test_cases = [
    # 格式 1：当前前端传递的格式
    {
        "operation": "execute",
        "shell": "bash",
        "command": "echo 'Hello'",
        "timeout_seconds": 30,
        "environment": [],
        "working_dir": None
    },
    # 格式 2：没有 operation 字段
    {
        "shell": "bash",
        "command": "echo 'Hello'",
        "timeout_seconds": 30,
        "environment": [],
        "working_dir": None
    },
    # 格式 3：使用 Execute 大写
    {
        "operation": "Execute",
        "shell": "bash",
        "command": "echo 'Hello'",
    },
    # 格式 4：AI 可能返回的格式
    {
        "command": "echo 'Hello'"
    }
]

for i, case in enumerate(test_cases, 1):
    print(f"\n=== 格式 {i} ===")
    print(json.dumps(case, indent=2))
