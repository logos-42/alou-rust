const fs = require('fs');

const filePath = '/Users/apple/Downloads/alou/alou-desktop/src/services/localIpfsGroupChatService.ts';
const lines = fs.readFileSync(filePath, 'utf-8').split('\n');

// 找到 joinGroup 方法的行号范围 (545-592)
// 替换第 551-555 行 (const group = this.groups.get(groupId) ... throw new Error('群聊不存在'))

const newLines = [];
for (let i = 0; i < lines.length; i++) {
    const lineNum = i + 1;
    
    // 替换 joinGroup 中的群聊查找逻辑 (行 551-555)
    if (lineNum === 551 && lines[i].includes('const group = this.groups.get(groupId)')) {
        // 插入新的多行代码
        newLines.push('    // 1. 首先尝试从内存获取群聊');
        newLines.push('    let group = this.groups.get(groupId)');
        newLines.push('    ');
        newLines.push('    // 2. 如果内存中没有，尝试从 KV 加载');
        newLines.push('    if (!group) {');
        newLines.push('      this.log(LogLevel.INFO, \'内存中未找到群聊，尝试从 KV 加载:\', { groupId })');
        newLines.push('      group = await this.loadGroupFromKV(groupId)');
        newLines.push('    }');
        newLines.push('    ');
        newLines.push('    // 3. 如果 KV 中也没有，创建新的群聊对象（用于加入外部创建的群聊）');
        newLines.push('    if (!group) {');
        newLines.push('      this.log(LogLevel.INFO, \'KV 中也未找到群聊，创建新群聊对象:\', { groupId })');
        newLines.push('      const groupTopic = topic || `diap/cluster_action/${groupId}`');
        newLines.push('      group = new LocalGroup({');
        newLines.push('        groupId,');
        newLines.push('        groupName: `群聊 ${groupId.slice(-8)}`,');
        newLines.push('        description: \'\',');
        newLines.push('        topic: groupTopic,');
        newLines.push('        members: [],');
        newLines.push('        creator: \'unknown\',');
        newLines.push('        createdAt: Date.now(),');
        newLines.push('        metadata: {');
        newLines.push('          isPublic: true,');
        newLines.push('          externalGroup: true');
        newLines.push('        }');
        newLines.push('      })');
        newLines.push('      ');
        newLines.push('      // 保存到新创建的群聊到 KV');
        newLines.push('      await this.saveGroupToKV(group)');
        newLines.push('    }');
        
        // 跳过原来的 3 行 (const group = ... 和 if (!group) { throw ... })
        i += 4; // 跳过 551-554 行
        continue;
    }
    
    newLines.push(lines[i]);
}

fs.writeFileSync(filePath, newLines.join('\n'), 'utf-8');
console.log('joinGroup method updated');
