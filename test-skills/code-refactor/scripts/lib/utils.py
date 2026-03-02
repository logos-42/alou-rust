#!/usr/bin/env python3
"""
Code Refactor Skill - 工具函数库
提供 AST 操作和代码重构的辅助函数
"""

import ast
from typing import List, Optional, Dict, Any

# 尝试导入 astor，如果不存在则使用 ast.unparse
try:
    import astor
    HAS_ASTOR = True
except ImportError:
    HAS_ASTOR = False


class RefactorError(Exception):
    """重构操作异常"""
    pass


def parse_source(source_code: str) -> ast.AST:
    """
    解析源代码为 AST
    
    Args:
        source_code: Python 源代码字符串
        
    Returns:
        AST 树
        
    Raises:
        RefactorError: 解析失败时抛出
    """
    try:
        return ast.parse(source_code)
    except SyntaxError as e:
        raise RefactorError(f"源代码语法错误 (行 {e.lineno}): {e.msg}")


def generate_code(tree: ast.AST) -> str:
    """
    将 AST 转换回源代码
    
    Args:
        tree: AST 树
        
    Returns:
        生成的 Python 代码
    """
    try:
        if HAS_ASTOR:
            # 使用 astor 将 AST 转回代码
            return astor.to_source(tree)
        else:
            # 使用 Python 3.9+ 内置的 ast.unparse
            return ast.unparse(tree)
    except Exception as e:
        raise RefactorError(f"代码生成失败: {str(e)}")


class VariableRenamer(ast.NodeTransformer):
    """变量重命名处理器"""
    
    def __init__(self, old_name: str, new_name: str):
        self.old_name = old_name
        self.new_name = new_name
        self.changes = []
    
    def visit_Name(self, node: ast.Name) -> ast.Name:
        """替换名称引用"""
        if node.id == self.old_name:
            node.id = self.new_name
            self.changes.append({
                "line": getattr(node, 'lineno', 0),
                "col": getattr(node, 'col_offset', 0)
            })
        return self.generic_visit(node)
    
    def visit_arg(self, node: ast.arg) -> ast.arg:
        """替换函数参数名"""
        if node.arg == self.old_name:
            node.arg = self.new_name
        return self.generic_visit(node)
    
    def visit_FunctionDef(self, node: ast.FunctionDef) -> ast.FunctionDef:
        """处理函数定义"""
        # 不处理嵌套函数中相同名称的变量
        return self.generic_visit(node)


def rename_variable(tree: ast.AST, old_name: str, new_name: str) -> ast.AST:
    """
    重命名变量
    
    Args:
        tree: AST 树
        old_name: 原变量名
        new_name: 新变量名
        
    Returns:
        修改后的 AST 树
        
    Raises:
        RefactorError: 新名称已存在时抛出
    """
    # 检查新名称是否已存在
    for node in ast.walk(tree):
        if isinstance(node, ast.Name) and node.id == new_name:
            raise RefactorError(f"变量名 '{new_name}' 已存在于代码中")
    
    renamer = VariableRenamer(old_name, new_name)
    new_tree = renamer.visit(tree)
    ast.fix_missing_locations(new_tree)
    return new_tree


class FunctionExtractor(ast.NodeTransformer):
    """函数提取处理器"""
    
    def __init__(self, start_line: int, end_line: int, func_name: str):
        self.start_line = start_line
        self.end_line = end_line
        self.func_name = func_name
        self.extracted_nodes = []
        self.new_function = None
    
    def visit_FunctionDef(self, node: ast.FunctionDef) -> ast.FunctionDef:
        """在函数体内查找可提取的代码块"""
        new_body = []
        extract_buffer = []
        
        for stmt in node.body:
            stmt_line = getattr(stmt, 'lineno', 0)
            
            if self.start_line <= stmt_line <= self.end_line:
                extract_buffer.append(stmt)
            else:
                new_body.append(stmt)
        
        if extract_buffer:
            # 创建新函数
            self.new_function = ast.FunctionDef(
                name=self.func_name,
                args=ast.arguments(
                    posonlyargs=[],
                    args=[],
                    kwonlyargs=[],
                    defaults=[],
                    kw_defaults=[]
                ),
                body=extract_buffer,
                decorator_list=[],
                returns=None
            )
            # 在原位置插入函数调用
            new_body.insert(
                self.start_line - node.body[0].lineno if node.body else 0,
                ast.Expr(value=ast.Call(
                    func=ast.Name(id=self.func_name, ctx=ast.Load()),
                    args=[],
                    keywords=[]
                ))
            )
            node.body = new_body
        
        return self.generic_visit(node)


def extract_function(
    tree: ast.AST, 
    start_line: int, 
    end_line: int, 
    func_name: str
) -> ast.AST:
    """
    提取代码块为新函数
    
    Args:
        tree: AST 树
        start_line: 开始行号
        end_line: 结束行号
        func_name: 新函数名
        
    Returns:
        修改后的 AST 树
    """
    extractor = FunctionExtractor(start_line, end_line, func_name)
    new_tree = extractor.visit(tree)
    
    if extractor.new_function:
        # 将新函数添加到模块级别
        new_tree.body.insert(0, extractor.new_function)
    
    ast.fix_missing_locations(new_tree)
    return new_tree


class VariableInliner(ast.NodeTransformer):
    """变量内联处理器"""
    
    def __init__(self, var_name: str):
        self.var_name = var_name
        self.var_value = None
        self.should_remove_assign = False
    
    def visit_Assign(self, node: ast.Assign) -> Optional[ast.AST]:
        """查找并移除变量赋值"""
        if len(node.targets) == 1:
            target = node.targets[0]
            if isinstance(target, ast.Name) and target.id == self.var_name:
                self.var_value = node.value
                return None  # 移除赋值语句
        return self.generic_visit(node)
    
    def visit_Name(self, node: ast.Name) -> ast.AST:
        """替换变量引用为其实际值"""
        if node.id == self.var_name and self.var_value:
            # 创建值的深拷贝
            import copy
            return copy.deepcopy(self.var_value)
        return self.generic_visit(node)


def inline_variable(tree: ast.AST, var_name: str) -> ast.AST:
    """
    内联变量（将变量替换为其实际值）
    
    Args:
        tree: AST 树
        var_name: 要内联的变量名
        
    Returns:
        修改后的 AST 树
    """
    inliner = VariableInliner(var_name)
    new_tree = inliner.visit(tree)
    ast.fix_missing_locations(new_tree)
    return new_tree


class ImportOptimizer(ast.NodeTransformer):
    """导入优化处理器"""
    
    def __init__(self):
        self.used_names: set = set()
        self.import_nodes: List[ast.Import] = []
        self.from_import_nodes: List[ast.ImportFrom] = []
    
    def visit_Import(self, node: ast.Import) -> ast.AST:
        """记录 import 语句"""
        self.import_nodes.append(node)
        return self.generic_visit(node)
    
    def visit_ImportFrom(self, node: ast.ImportFrom) -> ast.AST:
        """记录 from import 语句"""
        self.from_import_nodes.append(node)
        return self.generic_visit(node)
    
    def visit_Name(self, node: ast.Name) -> ast.Name:
        """收集所有使用的名称"""
        self.used_names.add(node.id)
        return self.generic_visit(node)
    
    def remove_unused_imports(self, tree: ast.AST) -> ast.AST:
        """移除未使用的导入"""
        new_body = []
        
        for node in tree.body:
            if isinstance(node, ast.Import):
                new_names = [
                    alias for alias in node.names 
                    if alias.asname in self.used_names or alias.name in self.used_names
                ]
                if new_names:
                    node.names = new_names
                    new_body.append(node)
            elif isinstance(node, ast.ImportFrom):
                new_names = [
                    alias for alias in node.names
                    if alias.asname in self.used_names or alias.name in self.used_names
                ]
                if new_names:
                    node.names = new_names
                    new_body.append(node)
            else:
                new_body.append(node)
        
        tree.body = new_body
        return tree


def optimize_imports(tree: ast.AST) -> ast.AST:
    """
    优化导入语句
    
    Args:
        tree: AST 树
        
    Returns:
        修改后的 AST 树
    """
    optimizer = ImportOptimizer()
    optimizer.visit(tree)
    return optimizer.remove_unused_imports(tree)


def find_all_variables(tree: ast.AST) -> Dict[str, List[Dict[str, int]]]:
    """
    查找代码中所有的变量及其出现位置
    
    Args:
        tree: AST 树
        
    Returns:
        变量名到位置列表的映射
    """
    variables = {}
    
    for node in ast.walk(tree):
        if isinstance(node, ast.Name):
            name = node.id
            location = {
                "line": getattr(node, 'lineno', 0),
                "col": getattr(node, 'col_offset', 0)
            }
            if name not in variables:
                variables[name] = []
            variables[name].append(location)
    
    return variables


def analyze_function_complexity(tree: ast.AST) -> List[Dict[str, Any]]:
    """
    分析函数复杂度
    
    Args:
        tree: AST 树
        
    Returns:
        函数信息列表
    """
    functions = []
    
    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            # 计算圈复杂度（简化版）
            complexity = 1
            for child in ast.walk(node):
                if isinstance(child, (ast.If, ast.While, ast.For, 
                                      ast.ExceptHandler, ast.With,
                                      ast.Assert, ast.comprehension)):
                    complexity += 1
            
            functions.append({
                "name": node.name,
                "line": node.lineno,
                "complexity": complexity,
                "lines": len(node.body)
            })
    
    return functions
