#!/usr/bin/env python3
import re
import os

# Files to fix
files_to_fix = [
    'src/tools/filesystem.rs',
    'src/tools/search.rs',
    'src/tools/system.rs',
    'src/tools/network.rs',
    'src/tools/plan.rs',
    'src/tools/todolist.rs',
    'src/tools/agent_skills.rs',
    'src/tools/agent_collaboration.rs',
    'src/tools/tool_creation.rs',
    'src/tools/tool_parts/executor.rs',
    'src/tools/git_helper/mod.rs',
    'src/tools/rollback/mod.rs',
    'src/tools/agent_creator.rs',
    'src/tools/ipfs_archive.rs',
    'src/tools/spec_tool.rs',
    'src/tools/skills/mod.rs',
    'src/tools/ui_control.rs',
    'src/tools/message_passing.rs',
    'src/tools/pubsub_tool.rs',
    'src/tools/browser_tool.rs',
    'src/tools/adapters/group_adapter.rs',
    'src/tools/registry.rs',
    'src/tools/executor.rs',
    'src/tools/refactored_tool_creation.rs',
    'src/tools/query_blockchain.rs',
    'src/tools/agent_wallet.rs',
    'src/tools/wallet_manager.rs',
    'src/tools/build_transaction.rs',
    'src/tools/broadcast_transaction.rs',
]

# Pattern to find ToolMetadata initialization without tags field
pattern_no_tags = r'(permissions: vec!\[[^\]]*\],)\s*([\)}])'
replacement_with_tags = r'\1\n                tags: vec![],\n            \2'

for filepath in files_to_fix:
    full_path = f'/Users/apple/Downloads/alou/alou-desktop/src-tauri/{filepath}'
    if not os.path.exists(full_path):
        print(f"Skipping {filepath} - file not found")
        continue
    
    with open(full_path, 'r') as f:
        content = f.read()
    
    # Check if tags field already exists
    if 'tags:' in content:
        print(f"Skipping {filepath} - tags field already exists")
        continue
    
    # Add tags field
    new_content = re.sub(pattern_no_tags, replacement_with_tags, content)
    
    if new_content != content:
        with open(full_path, 'w') as f:
            f.write(new_content)
        print(f"Fixed {filepath}")
    else:
        print(f"No changes for {filepath}")

print("Done!")
