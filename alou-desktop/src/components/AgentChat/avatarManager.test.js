/**
 * 头像管理模块测试
 * 验证头像同步功能
 */

import avatarManager from './avatarManager'

// Mock useAgentStore
const mockAgents = [
  {
    id: 'agent-1',
    name: '测试智能体1',
    avatar: 'https://example.com/avatar1.jpg',
    avatar_url: 'https://example.com/avatar1.jpg',
    updated_at: Date.now()
  },
  {
    id: 'agent-2',
    name: '测试智能体2',
    avatar: null,
    avatar_url: null,
    updated_at: Date.now()
  }
]

// Mock useAgentStore.getState
const mockStore = {
  agents: mockAgents,
  updateAgent: jest.fn((id, updates) => {
    const agent = mockAgents.find(a => a.id === id)
    if (!agent) return null
    
    const updatedAgent = { ...agent, ...updates }
    const index = mockAgents.findIndex(a => a.id === id)
    mockAgents[index] = updatedAgent
    
    return updatedAgent
  })
}

// 设置全局 mock
global.useAgentStore = {
  getState: () => mockStore
}

describe('AvatarManager', () => {
  beforeEach(() => {
    // 清除缓存和监听器
    avatarManager.clearAllCache()
    avatarManager.listeners.clear()
    jest.clearAllMocks()
  })

  test('resolveAvatar 应该返回正确的头像URL', () => {
    const agent = {
      id: 'test-agent',
      avatar: 'https://example.com/test.jpg'
    }
    
    const avatar = avatarManager.resolveAvatar(agent)
    expect(avatar).toBe('https://example.com/test.jpg')
    
    // 应该被缓存
    const cachedAvatar = avatarManager.resolveAvatar(agent)
    expect(cachedAvatar).toBe(avatar)
  })

  test('resolveAvatar 应该支持 data URL', () => {
    const agent = {
      id: 'test-agent',
      avatar: 'data:image/png;base64,test123'
    }
    
    const avatar = avatarManager.resolveAvatar(agent)
    expect(avatar).toBe('data:image/png;base64,test123')
  })

  test('resolveAvatar 应该返回默认头像当agent为空时', () => {
    const avatar = avatarManager.resolveAvatar(null)
    expect(avatar).toBe(avatarManager.getFallbackAvatar())
  })

  test('updateAvatar 应该更新智能体头像', () => {
    const agentId = 'agent-1'
    const newAvatar = 'https://example.com/new-avatar.jpg'
    
    const updatedAgent = avatarManager.updateAvatar(agentId, newAvatar)
    
    expect(updatedAgent).toBeDefined()
    expect(updatedAgent.avatar).toBe(newAvatar)
    expect(updatedAgent.avatar_url).toBe(newAvatar)
    expect(mockStore.updateAgent).toHaveBeenCalledWith(agentId, {
      avatar: newAvatar,
      avatar_url: newAvatar,
      updated_at: expect.any(Number)
    })
  })

  test('updateAvatar 应该通知监听器', () => {
    const listener = jest.fn()
    avatarManager.addListener(listener)
    
    const agentId = 'agent-1'
    const newAvatar = 'https://example.com/new-avatar.jpg'
    
    avatarManager.updateAvatar(agentId, newAvatar)
    
    expect(listener).toHaveBeenCalledWith(expect.objectContaining({
      id: agentId,
      avatar: newAvatar
    }))
  })

  test('updateChannelsAvatar 应该更新频道列表', () => {
    const channels = [
      {
        id: 'channel-1',
        name: '频道1',
        avatar: 'old-avatar.jpg',
        meta: { id: 'agent-1' }
      },
      {
        id: 'channel-2',
        name: '频道2',
        avatar: 'avatar2.jpg',
        meta: { id: 'agent-2' }
      }
    ]
    
    const updatedAgent = {
      id: 'agent-1',
      name: '更新后的智能体',
      display_name: '更新后的显示名',
      avatar: 'new-avatar.jpg'
    }
    
    const updatedChannels = avatarManager.updateChannelsAvatar(channels, updatedAgent)
    
    expect(updatedChannels[0].avatar).toBe('new-avatar.jpg')
    expect(updatedChannels[0].name).toBe('更新后的显示名')
    expect(updatedChannels[0].meta).toBe(updatedAgent)
    expect(updatedChannels[1].avatar).toBe('avatar2.jpg') // 不应该被修改
  })

  test('validateAvatar 应该验证有效的头像URL', () => {
    expect(avatarManager.validateAvatar('https://example.com/avatar.jpg')).toBe(true)
    expect(avatarManager.validateAvatar('http://example.com/avatar.jpg')).toBe(true)
    expect(avatarManager.validateAvatar('data:image/png;base64,test')).toBe(true)
    expect(avatarManager.validateAvatar('ipfs://QmTest')).toBe(true)
    expect(avatarManager.validateAvatar('invalid-url')).toBe(false)
    expect(avatarManager.validateAvatar(null)).toBe(false)
    expect(avatarManager.validateAvatar('')).toBe(false)
  })

  test('processFileUpload 应该处理有效的图片文件', async () => {
    // 创建 mock File 对象
    const mockFile = new File(['test'], 'avatar.png', { type: 'image/png' })
    
    // Mock FileReader
    const mockFileReader = {
      readAsDataURL: jest.fn(),
      onload: null,
      onerror: null
    }
    
    global.FileReader = jest.fn(() => mockFileReader)
    
    // 测试
    const promise = avatarManager.processFileUpload(mockFile)
    
    // 模拟读取完成
    mockFileReader.onload({ target: { result: 'data:image/png;base64,test' } })
    
    const result = await promise
    expect(result).toBe('data:image/png;base64,test')
  })

  test('processFileUpload 应该拒绝非图片文件', async () => {
    const mockFile = new File(['test'], 'document.pdf', { type: 'application/pdf' })
    
    await expect(avatarManager.processFileUpload(mockFile))
      .rejects
      .toThrow('请选择图片文件')
  })

  test('processFileUpload 应该拒绝过大的文件', async () => {
    const mockFile = new File([new ArrayBuffer(6 * 1024 * 1024)], 'large-image.png', { type: 'image/png' })
    
    await expect(avatarManager.processFileUpload(mockFile))
      .rejects
      .toThrow('图片大小不能超过5MB')
  })
})

describe('AvatarManager 集成测试', () => {
  test('头像更新应该触发全局事件', () => {
    const eventListener = jest.fn()
    window.addEventListener('agent-avatar-updated', eventListener)
    
    const agentId = 'agent-1'
    const newAvatar = 'https://example.com/updated.jpg'
    
    avatarManager.updateAvatar(agentId, newAvatar)
    
    expect(eventListener).toHaveBeenCalledWith(
      expect.objectContaining({
        detail: expect.objectContaining({
          agentId,
          avatar: newAvatar
        })
      })
    )
    
    window.removeEventListener('agent-avatar-updated', eventListener)
  })

  test('缓存应该根据更新时间失效', () => {
    const agent = {
      id: 'test-agent',
      avatar: 'avatar1.jpg',
      updated_at: 1000
    }
    
    // 第一次解析
    const avatar1 = avatarManager.resolveAvatar(agent)
    expect(avatar1).toBe('avatar1.jpg')
    
    // 更新agent
    agent.avatar = 'avatar2.jpg'
    agent.updated_at = 2000
    
    // 应该重新解析（因为updated_at不同）
    const avatar2 = avatarManager.resolveAvatar(agent)
    expect(avatar2).toBe('avatar2.jpg')
  })
})
