const fs = require('fs');
const filePath = '/Users/apple/Downloads/alou/alou-desktop/src/services/localIpfsGroupChatService.ts';
let content = fs.readFileSync(filePath, 'utf-8');

// 修复损坏的注释
const brokenComment = `  }

  */
  async joinGroup`;

const fixedComment = `  }

  /**
   * 加入群聊
   * @param groupId - 群聊 ID
   * @param topic - 群聊主题（可选）
   * @returns 加入的群聊信息
   */
  async joinGroup`;

content = content.replace(brokenComment, fixedComment);

fs.writeFileSync(filePath, content, 'utf-8');
console.log('Comment fixed');
