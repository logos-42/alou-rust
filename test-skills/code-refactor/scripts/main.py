#!/usr/bin/env python3
"""
Code Refactor Skill - 主执行脚本
基于 AST 的智能代码重构工具

Usage:
    python main.py --input '{"operation": "rename_variable", "file": "src/main.py", ...}'
"""

import argparse
import json
import sys
from pathlib import Path

# 添加 lib 目录到路径
sys.path.insert(0, str(Path(__file__).parent))

from lib.utils import (
    RefactorError,
    parse_source,
    generate_code,
    rename_variable,
    extract_function,
    inline_variable,
    optimize_imports,
)


def main():
    parser = argparse.ArgumentParser(description="代码重构工具")
    parser.add_argument(
        "--input",
        type=str,
        required=True,
        help="JSON 格式的输入参数"
    )
    args = parser.parse_args()

    try:
        # 解析输入参数
        params = json.loads(args.input)
        operation = params.get("operation")
        file_path = params.get("file")

        if not operation:
            raise RefactorError("缺少必需的参数: operation")
        if not file_path:
            raise RefactorError("缺少必需的参数: file")

        # 读取源文件
        source_file = Path(file_path)
        if not source_file.exists():
            raise RefactorError(f"文件不存在: {file_path}")

        source_code = source_file.read_text(encoding="utf-8")
        tree = parse_source(source_code)

        # 执行重构操作
        result = {"success": False, "operation": operation}

        if operation == "rename_variable":
            old_name = params.get("old_name")
            new_name = params.get("new_name")
            if not old_name or not new_name:
                raise RefactorError("重命名变量需要提供 old_name 和 new_name")
            
            modified_tree = rename_variable(tree, old_name, new_name)
            result["changes"] = {
                "renamed": f"{old_name} -> {new_name}",
                "locations": find_variable_occurrences(tree, old_name)
            }

        elif operation == "extract_function":
            start_line = params.get("start_line")
            end_line = params.get("end_line")
            func_name = params.get("function_name")
            
            if not all([start_line, end_line, func_name]):
                raise RefactorError("提取函数需要提供 start_line, end_line 和 function_name")
            
            modified_tree = extract_function(tree, start_line, end_line, func_name)
            result["changes"] = {
                "extracted_function": func_name,
                "lines": f"{start_line}-{end_line}"
            }

        elif operation == "inline_variable":
            var_name = params.get("variable_name")
            if not var_name:
                raise RefactorError("内联变量需要提供 variable_name")
            
            modified_tree = inline_variable(tree, var_name)
            result["changes"] = {
                "inlined_variable": var_name
            }

        elif operation == "optimize_imports":
            modified_tree = optimize_imports(tree)
            result["changes"] = {
                "optimized": True
            }

        else:
            raise RefactorError(f"不支持的操作: {operation}")

        # 生成新代码
        new_code = generate_code(modified_tree)

        # 写回文件
        backup_path = source_file.with_suffix(".py.bak")
        source_file.rename(backup_path)
        source_file.write_text(new_code, encoding="utf-8")

        result["success"] = True
        result["file"] = file_path
        result["backup"] = str(backup_path)

    except RefactorError as e:
        result = {
            "success": False,
            "error": "refactor_error",
            "message": str(e)
        }
    except json.JSONDecodeError as e:
        result = {
            "success": False,
            "error": "json_parse_error",
            "message": f"JSON 解析错误: {str(e)}"
        }
    except Exception as e:
        result = {
            "success": False,
            "error": "unexpected_error",
            "message": str(e)
        }

    # 输出 JSON 结果
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0 if result.get("success") else 1


def find_variable_occurrences(tree, var_name):
    """查找变量出现的所有位置"""
    import ast
    locations = []
    for node in ast.walk(tree):
        if isinstance(node, ast.Name) and node.id == var_name:
            if hasattr(node, 'lineno'):
                locations.append({
                    "line": node.lineno,
                    "col": getattr(node, 'col_offset', 0)
                })
    return locations


if __name__ == "__main__":
    sys.exit(main())
