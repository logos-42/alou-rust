/**
 * PubSub群聊组件
 * 显示和管理IPFS PubSub群聊
 * 无需钱包验证，AI智能体可以自主创建
 */

import React, { useState, useEffect, useCallback } from 'react';
import { 
  Card, 
  CardContent, 
  CardHeader, 
  CardTitle, 
  CardDescription, 
  CardFooter 
} from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { Badge } from '@/components/ui/badge';
import { 
  MessageSquare, 
  Users, 
  Plus, 
  Send, 
  RefreshCw,
  Bot,
  Sparkles,
  Clock,
  Hash
} from 'lucide-react';
import enhancedToolService from '@/services/toolServiceWithPubSub';
import PubSubToolService, { PubSubGroup, PubSubMessage } from '@/services/pubsubToolService';
import { useToast } from '@/hooks/use-toast';

/**
 * PubSub群聊组件属性
 */
interface PubSubGroupChatProps {
  /** 是否显示AI创建按钮 */
  showAiCreateButton?: boolean;
  /** 初始群聊ID */
  initialGroupId?: string;
  /** 用户ID */
  userId?: string;
  /** 用户名称 */
  userName?: string;
}

/**
 * PubSub群聊组件
 */
const PubSubGroupChat: React.FC<PubSubGroupChatProps> = ({
  showAiCreateButton = true,
  initialGroupId,
  userId = 'user_' + Date.now(),
  userName = '用户'
}) => {
  const { toast } = useToast();
  const [groups, setGroups] = useState<PubSubGroup[]>([]);
  const [selectedGroup, setSelectedGroup] = useState<PubSubGroup | null>(null);
  const [messages, setMessages] = useState<PubSubMessage[]>([]);
  const [newMessage, setNewMessage] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [newGroupName, setNewGroupName] = useState('');
  const [newGroupDescription, setNewGroupDescription] = useState('');
  const [isCreatingGroup, setIsCreatingGroup] = useState(false);
  
  const pubSubService = PubSubToolService.getInstance();

  /**
   * 加载群聊列表
   */
  const loadGroups = useCallback(async () => {
    setIsLoading(true);
    try {
      const result = await pubSubService.listGroups();
      if (result.success && result.data?.groups) {
        setGroups(result.data.groups);
        
        // 如果有初始群聊ID，自动选择
        if (initialGroupId && !selectedGroup) {
          const group = result.data.groups.find(g => g.id === initialGroupId);
          if (group) {
            setSelectedGroup(group);
            loadMessages(group.id);
          }
        }
      }
    } catch (error) {
      console.error('Failed to load groups:', error);
      toast({
        title: '加载失败',
        description: '无法加载群聊列表',
        variant: 'destructive'
      });
    } finally {
      setIsLoading(false);
    }
  }, [initialGroupId, selectedGroup, pubSubService, toast]);

  /**
   * 加载消息
   */
  const loadMessages = useCallback(async (groupId: string) => {
    try {
      const result = await pubSubService.getGroupMessages(groupId);
      if (result.success && result.data?.messages) {
        setMessages(result.data.messages);
      }
    } catch (error) {
      console.error('Failed to load messages:', error);
    }
  }, [pubSubService]);

  /**
   * 创建新群聊
   */
  const handleCreateGroup = async () => {
    if (!newGroupName.trim()) {
      toast({
        title: '输入错误',
        description: '请输入群聊名称',
        variant: 'destructive'
      });
      return;
    }

    setIsCreatingGroup(true);
    try {
      const result = await pubSubService.createGroup({
        name: newGroupName,
        description: newGroupDescription,
        isPublic: true,
        maxMembers: 100
      });

      if (result.success && result.group) {
        toast({
          title: '创建成功',
          description: `群聊 "${newGroupName}" 已创建`,
          variant: 'default'
        });
        
        setNewGroupName('');
        setNewGroupDescription('');
        await loadGroups();
        
        // 自动选择新创建的群聊
        if (result.group) {
          setSelectedGroup(result.group);
          loadMessages(result.group.id);
        }
      } else {
        throw new Error(result.error || '创建失败');
      }
    } catch (error) {
      console.error('Failed to create group:', error);
      toast({
        title: '创建失败',
        description: error instanceof Error ? error.message : '未知错误',
        variant: 'destructive'
      });
    } finally {
      setIsCreatingGroup(false);
    }
  };

  /**
   * AI自主创建群聊
   */
  const handleAiCreateGroup = async () => {
    setIsLoading(true);
    try {
      const result = await enhancedToolService.simulateAiCreatingGroup();
      
      if (result.success) {
        toast({
          title: 'AI创建成功',
          description: 'AI智能体已自主创建群聊',
          variant: 'default'
        });
        
        await loadGroups();
      } else {
        throw new Error(result.error || 'AI创建失败');
      }
    } catch (error) {
      console.error('AI创建群聊失败:', error);
      toast({
        title: 'AI创建失败',
        description: error instanceof Error ? error.message : '未知错误',
        variant: 'destructive'
      });
    } finally {
      setIsLoading(false);
    }
  };

  /**
   * 发送消息
   */
  const handleSendMessage = async () => {
    if (!newMessage.trim() || !selectedGroup) {
      return;
    }

    try {
      const result = await pubSubService.sendMessage(
        selectedGroup.id,
        userId,
        userName,
        newMessage.trim()
      );

      if (result.success) {
        // 添加消息到本地列表
        const newMsg: PubSubMessage = {
          id: result.data?.messageId || `msg_${Date.now()}`,
          senderId: userId,
          senderName: userName,
          content: newMessage.trim(),
          type: 'text',
          timestamp: Date.now()
        };
        
        setMessages(prev => [...prev, newMsg]);
        setNewMessage('');
        
        // 滚动到底部
        setTimeout(() => {
          const messagesContainer = document.getElementById('messages-container');
          if (messagesContainer) {
            messagesContainer.scrollTop = messagesContainer.scrollHeight;
          }
        }, 100);
      }
    } catch (error) {
      console.error('Failed to send message:', error);
      toast({
        title: '发送失败',
        description: '消息发送失败',
        variant: 'destructive'
      });
    }
  };

  /**
   * 加入群聊
   */
  const handleJoinGroup = async (group: PubSubGroup) => {
    try {
      const result = await pubSubService.joinGroup(group.id, userName);
      
      if (result.success) {
        toast({
          title: '加入成功',
          description: `已加入群聊 "${group.name}"`,
          variant: 'default'
        });
        
        setSelectedGroup(group);
        loadMessages(group.id);
      }
    } catch (error) {
      console.error('Failed to join group:', error);
      toast({
        title: '加入失败',
        description: '无法加入群聊',
        variant: 'destructive'
      });
    }
  };

  /**
   * 格式化时间
   */
  const formatTime = (timestamp: number) => {
    return new Date(timestamp).toLocaleTimeString('zh-CN', {
      hour: '2-digit',
      minute: '2-digit'
    });
  };

  /**
   * 初始化
   */
  useEffect(() => {
    loadGroups();
    
    // 添加群聊更新监听器
    const handleGroupUpdate = (group: PubSubGroup) => {
      setGroups(prev => {
        const index = prev.findIndex(g => g.id === group.id);
        if (index >= 0) {
          const newGroups = [...prev];
          newGroups[index] = group;
          return newGroups;
        } else {
          return [...prev, group];
        }
      });
      
      if (selectedGroup?.id === group.id) {
        setSelectedGroup(group);
      }
    };
    
    pubSubService.addGroupUpdateListener(handleGroupUpdate);
    
    return () => {
      // 清理监听器
      // 注意：在实际应用中需要实现移除监听器的方法
    };
  }, [loadGroups, pubSubService, selectedGroup]);

  return (
    <div className="flex flex-col h-full">
      {/* 标题栏 */}
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <MessageSquare className="h-5 w-5 text-primary" />
          <h2 className="text-xl font-bold">PubSub群聊</h2>
          <Badge variant="outline" className="ml-2">
            <Hash className="h-3 w-3 mr-1" />
            IPFS
          </Badge>
        </div>
        
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={loadGroups}
            disabled={isLoading}
          >
            <RefreshCw className={`h-4 w-4 mr-2 ${isLoading ? 'animate-spin' : ''}`} />
            刷新
          </Button>
          
          {showAiCreateButton && (
            <Button
              variant="default"
              size="sm"
              onClick={handleAiCreateGroup}
              disabled={isLoading}
            >
              <Bot className="h-4 w-4 mr-2" />
              AI创建群聊
            </Button>
          )}
        </div>
      </div>

      <div className="flex flex-1 gap-4 overflow-hidden">
        {/* 左侧：群聊列表 */}
        <Card className="w-1/3 flex flex-col">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Users className="h-5 w-5" />
              群聊列表
            </CardTitle>
            <CardDescription>
              基于IPFS PubSub的去中心化群聊
            </CardDescription>
          </CardHeader>
          
          <CardContent className="flex-1 overflow-y-auto">
            {isLoading ? (
              <div className="flex items-center justify-center h-32">
                <RefreshCw className="h-8 w-8 animate-spin text-muted-foreground" />
              </div>
            ) : groups.length === 0 ? (
              <div className="text-center text-muted-foreground py-8">
                <MessageSquare className="h-12 w-12 mx-auto mb-4 opacity-50" />
                <p>还没有群聊</p>
                <p className="text-sm">创建一个新的群聊开始协作</p>
              </div>
            ) : (
              <div className="space-y-2">
                {groups.map(group => (
                  <Card
                    key={group.id}
                    className={`cursor-pointer transition-all hover:bg-accent ${
                      selectedGroup?.id === group.id ? 'border-primary bg-accent' : ''
                    }`}
                    onClick={() => {
                      setSelectedGroup(group);
                      loadMessages(group.id);
                    }}
                  >
                    <CardContent className="p-4">
                      <div className="flex justify-between items-start">
                        <div>
                          <h4 className="font-semibold">{group.name}</h4>
                          {group.description && (
                            <p className="text-sm text-muted-foreground mt-1">
                              {group.description}
                            </p>
                          )}
                        </div>
                        <Badge variant="secondary">
                          {group.members.length}人
                        </Badge>
                      </div>
                      
                      <div className="flex items-center justify-between mt-3">
                        <div className="flex items-center text-xs text-muted-foreground">
                          <Clock className="h-3 w-3 mr-1" />
                          {formatTime(group.lastActivity)}
                        </div>
                        
                        {!group.members.some(m => m.id === userId) && (
                          <Button
                            size="sm"
                            variant="outline"
                            onClick={(e) => {
                              e.stopPropagation();
                              handleJoinGroup(group);
                            }}
                          >
                            加入
                          </Button>
                        )}
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            )}
          </CardContent>
          
          <CardFooter className="border-t pt-4">
            <div className="w-full space-y-3">
              <div className="space-y-2">
                <Input
                  placeholder="群聊名称"
                  value={newGroupName}
                  onChange={(e) => setNewGroupName(e.target.value)}
                  disabled={isCreatingGroup}
                />
                <Textarea
                  placeholder="群聊描述（可选）"
                  value={newGroupDescription}
                  onChange={(e) => setNewGroupDescription(e.target.value)}
                  disabled={isCreatingGroup}
                  rows={2}
                />
              </div>
              <Button
                className="w-full"
                onClick={handleCreateGroup}
                disabled={isCreatingGroup || !newGroupName.trim()}
              >
                {isCreatingGroup ? (
                  <>
                    <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
                    创建中...
                  </>
                ) : (
                  <>
                    <Plus className="h-4 w-4 mr-2" />
                    创建新群聊
                  </>
                )}
              </Button>
            </div>
          </CardFooter>
        </Card>

        {/* 右侧：聊天界面 */}
        <Card className="flex-1 flex flex-col">
          {selectedGroup ? (
            <>
              <CardHeader className="border-b">
                <div className="flex justify-between items-center">
                  <div>
                    <CardTitle>{selectedGroup.name}</CardTitle>
                    <CardDescription>
                      主题: {selectedGroup.topic} • {selectedGroup.members.length} 名成员
                    </CardDescription>
                  </div>
                  <Badge variant={selectedGroup.isActive ? "default" : "secondary"}>
                    {selectedGroup.isActive ? '活跃' : '休眠'}
                  </Badge>
                </div>
              </CardHeader>
              
              <CardContent className="flex-1 overflow-y-auto p-4" id="messages-container">
                {messages.length === 0 ? (
                  <div className="flex flex-col items-center justify-center h-full text-muted-foreground">
                    <MessageSquare className="h-16 w-16 mb-4 opacity-30" />
                    <p className="text-lg">还没有消息</p>
                    <p className="text-sm">发送第一条消息开始聊天</p>
                  </div>
                ) : (
                  <div className="space-y-4">
                    {messages.map((msg) => (
                      <div
                        key={msg.id}
                        className={`flex ${msg.senderId === userId ? 'justify-end' : 'justify-start'}`}
                      >
                        <div
                          className={`max-w-[70%] rounded-lg p-3 ${
                            msg.senderId === userId
                              ? 'bg-primary text-primary-foreground'
                              : msg.senderId === 'system'
                              ? 'bg-secondary text-secondary-foreground'
                              : 'bg-muted'
                          }`}
                        >
                          <div className="flex items-center gap-2 mb-1">
                            <span className="font-semibold text-sm">
                              {msg.senderName}
                              {msg.senderId === 'ai_system' && (
                                <Sparkles className="h-3 w-3 inline ml-1" />
                              )}
                            </span>
                            <span className="text-xs opacity-70">
                              {formatTime(msg.timestamp)}
                            </span>
                          </div>
                          <p className="whitespace-pre-wrap">{msg.content}</p>
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </CardContent>
              
              <CardFooter className="border-t p-4">
                <div className="flex w-full gap-2">
                  <Input
                    placeholder="输入消息..."
                    value={newMessage}
                    onChange={(e) => setNewMessage(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter' && !e.shiftKey) {
                        e.preventDefault();
                        handleSendMessage();
                      }
                    }}
                    disabled={!selectedGroup}
                  />
                  <Button
                    onClick={handleSendMessage}
                    disabled={!newMessage.trim() || !selectedGroup}
                  >
                    <Send className="h-4 w-4" />
                  </Button>
                </div>
              </CardFooter>
            </>
          ) : (
            <div className="flex flex-col items-center justify-center h-full text-muted-foreground">
              <MessageSquare className="h-24 w-24 mb-6 opacity-20" />
              <h3 className="text-xl font-semibold mb-2">选择或创建一个群聊</h3>
              <p className="text-center max-w-md">
                选择一个现有的群聊加入，或者创建一个新的群聊开始协作。
                <br />
                <span className="text-primary">无需钱包验证，AI智能体可以自主创建！</span>
              </p>
            </div>
          )}
        </Card>
      </div>

      {/* 状态栏 */}
      <div className="mt-4 text-xs text-muted-foreground flex justify-between">
        <div>
          用户: <span className="font-medium">{userName}</span> (ID: {userId})
        </div>
        <div>
          总群聊数: <span className="font-medium">{groups.length}</span> • 
          基于 <span className="font-medium">IPFS PubSub</span> 技术
        </div>
      </div>
    </div>
  );
};

export default PubSubGroupChat;