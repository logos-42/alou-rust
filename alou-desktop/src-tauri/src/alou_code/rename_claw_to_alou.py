#!/usr/bin/env python3
"""
Comprehensive rename script to replace all 'alou' references with 'alou' 
across the codebase while preserving capitalization patterns.
"""

import os
import re
from pathlib import Path

# Define replacement patterns
REPLACEMENTS = [
    # Word-boundary replacements for identifiers and product names
    (r'\bclaw-code\b', 'alou-code'),
    (r'\bClaw-Code\b', 'Alou-Code'),
    (r'\bCLAW-CODE\b', 'ALOU-CODE'),
    
    # Binary and CLI references
    (r'\bclaw\.exe\b', 'alou.exe'),
    (r'\bClaw\.exe\b', 'Alou.exe'),
    (r'\bCLAW\.EXE\b', 'ALOU.EXE'),
    
    # Variable/function names with alou
    (r'\bclaw_config\b', 'alou_config'),
    (r'\bClawConfig\b', 'AlouConfig'),
    (r'\bCLAW_CONFIG\b', 'ALOU_CONFIG'),
    
    # General word-boundary replacements (order matters - more specific first)
    (r'\bclaws\b', 'alous'),  # plural
    (r'\bClaws\b', 'Alous'),
    (r'\bCLAWS\b', 'ALOUS'),
    
    (r'\bclaw\b', 'alou'),
    (r'\bClaw\b', 'Alou'),
    (r'\bCLAW\b', 'ALOU'),
    
    # Path references
    (r'\.alou/', '.alou/'),
    (r'\.alou\b', '.alou'),
    (r'ALOU_HOME', 'ALOU_HOME'),
    (r'Alou_HOME', 'Alou_HOME'),
    (r'alou_home', 'alou_home'),
]

# Files/directories to skip
SKIP_PATTERNS = [
    '.git/',
    'target/',
    'node_modules/',
    '__pycache__/',
    'clawhip',  # External project reference - keep as-is
]

# File extensions to process
VALID_EXTENSIONS = {
    '.md', '.sh', '.rs', '.py', '.json', '.toml', 
    '.yaml', '.yml', '.txt', '.json5', 'Containerfile',
    '.js', '.ts', '.jsx', '.tsx', '.css', '.html',
    '.xml', '.ini', '.cfg', '.conf', '.env', '.lock',
    '.lua', '.rb', '.go', '.java', '.c', '.cpp', '.h', '.hpp',
    '',  # Files without extensions
}

def should_process_file(filepath):
    """Check if file should be processed."""
    # Skip certain directories
    for pattern in SKIP_PATTERNS:
        if pattern in str(filepath):
            # But don't skip if it's part of the filename itself (like clawhip)
            if pattern.endswith('/') or pattern not in os.path.basename(filepath):
                return False
    
    # Check file extension
    suffix = filepath.suffix.lower()
    name = filepath.name.lower()
    
    # Files without extensions (like Containerfile)
    if suffix == '' and name in ['containerfile', 'makefile', 'dockerfile']:
        return True
        
    return suffix in VALID_EXTENSIONS or name in VALID_EXTENSIONS

def process_file(filepath):
    """Process a single file and apply replacements."""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # Apply each replacement pattern
        for pattern, replacement in REPLACEMENTS:
            content = re.sub(pattern, replacement, content)
        
        # Only write if content changed
        if content != original_content:
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(content)
            return True
        return False
        
    except Exception as e:
        print(f"Error processing {filepath}: {e}")
        return False

def rename_file_with_claw_in_name(filepath):
    """Rename files that have 'alou' in their filename."""
    parent = filepath.parent
    name = filepath.name
    
    # Apply replacements to filename
    new_name = name
    for pattern, replacement in REPLACEMENTS:
        new_name = re.sub(pattern, replacement, new_name)
    
    if new_name != name:
        new_path = parent / new_name
        try:
            filepath.rename(new_path)
            print(f"Renamed file: {filepath} -> {new_path}")
            return new_path
        except Exception as e:
            print(f"Error renaming {filepath}: {e}")
    
    return filepath

def main():
    root_dir = Path('/Users/apple/alou-code')
    
    print("Starting comprehensive alou -> alou rename...")
    print("=" * 60)
    
    # Collect all files first
    files_to_process = []
    files_with_claw_in_name = []
    
    for filepath in root_dir.rglob('*'):
        if not filepath.is_file():
            continue
            
        if not should_process_file(filepath):
            continue
            
        files_to_process.append(filepath)
        
        # Check if filename contains 'alou'
        if 'alou' in filepath.name.lower():
            files_with_claw_in_name.append(filepath)
    
    print(f"\nFound {len(files_to_process)} files to process")
    print(f"Found {len(files_with_claw_in_name)} files with 'alou' in name")
    
    # Process file contents
    modified_count = 0
    for filepath in files_to_process:
        if process_file(filepath):
            modified_count += 1
            print(f"Modified: {filepath}")
    
    print(f"\nModified {modified_count} files")
    
    # Rename files with alou in name
    print("\nRenaming files with 'alou' in name...")
    for filepath in files_with_claw_in_name:
        rename_file_with_claw_in_name(filepath)
    
    print("\n" + "=" * 60)
    print("Rename complete!")

if __name__ == '__main__':
    main()
