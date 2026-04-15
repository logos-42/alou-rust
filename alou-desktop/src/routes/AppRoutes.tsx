import React from 'react'
import { Routes, Route, Navigate } from 'react-router-dom'

import HomeView from '@/views/HomeView'
import CollabView from '@/views/CollabView'

// 保留所有现有功能入口（暂时重定向到主页）
// 后续可以逐步恢复每个功能页面

const AppRoutes = () => (
  <Routes>
    <Route path="/" element={<HomeView />} />
    <Route path="/collab/new" element={<CollabView />} />
    
    {/* 保留功能入口 - 暂时重定向到主页，后续逐步恢复 */}
    <Route path="/chat" element={<Navigate to="/" replace />} />
    <Route path="/group-chat" element={<Navigate to="/" replace />} />
    <Route path="/autonomous-loop" element={<Navigate to="/" replace />} />
    <Route path="/wallet" element={<Navigate to="/" replace />} />
    <Route path="/ipfs" element={<Navigate to="/" replace />} />
    <Route path="/skills" element={<Navigate to="/" replace />} />
    <Route path="/settings" element={<Navigate to="/" replace />} />
    
    <Route path="*" element={<Navigate to="/" replace />} />
  </Routes>
)

export default AppRoutes
