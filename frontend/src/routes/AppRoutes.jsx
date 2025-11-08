import React, { useEffect } from 'react'
import { Routes, Route, Navigate, useLocation } from 'react-router-dom'
import useAuthStore from '@/stores/authStore'

import HomeView from '@/views/HomeView'
import LoginView from '@/views/LoginView'
import AuthCallbackView from '@/views/AuthCallbackView'
import WalletView from '@/views/WalletView'
import AboutView from '@/views/AboutView'

const ProtectedRoute = ({ children }) => {
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated)
  const init = useAuthStore((state) => state.init)
  const location = useLocation()

  useEffect(() => {
    init()
  }, [init])

  if (!isAuthenticated) {
    return <Navigate to="/login" replace state={{ from: location }} />
  }

  return children
}

const AuthHiddenRoute = ({ children }) => {
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated)
  const init = useAuthStore((state) => state.init)

  useEffect(() => {
    init()
  }, [init])

  if (isAuthenticated) {
    return <Navigate to="/" replace />
  }

  return children
}

const AppRoutes = () => (
  <Routes>
    <Route path="/" element={<HomeView />} />
    <Route
      path="/wallet"
      element={
        <ProtectedRoute>
          <WalletView />
        </ProtectedRoute>
      }
    />
    <Route path="/about" element={<AboutView />} />
    <Route
      path="/login"
      element={
        <AuthHiddenRoute>
          <LoginView />
        </AuthHiddenRoute>
      }
    />
    <Route path="/auth/callback" element={<AuthCallbackView />} />
    <Route path="*" element={<Navigate to="/" replace />} />
  </Routes>
)

export default AppRoutes

