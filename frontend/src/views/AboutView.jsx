import React from 'react'
import './AboutView.css'

const AboutView = () => {
  // GitHub Release 下载链接
  const downloadUrl = 'https://github.com/logos-42/Alou-pay/releases/download/v0.1.0/Alou_0.1.0_x64-setup.exe'
  const version = '0.1.0'

  const handleDownload = () => {
    // 在新窗口打开下载链接
    window.open(downloadUrl, '_blank')
  }

  return (
    <div className="about">
      <div className="about-container">
        <h1>关于 Alou</h1>
        <p className="about-description">
          Alou 是一个 Web3 AI Agent 桌面应用，提供安全、便捷的加密钱包管理和 AI 智能体交互功能。
        </p>
        
        <div className="download-section">
          <h2>下载桌面版</h2>
          <p className="download-description">
            体验更强大的桌面应用功能，支持离线使用和更好的性能。
          </p>
          <button 
            className="download-button"
            onClick={handleDownload}
          >
            <span className="download-icon">⬇️</span>
            下载 Alou Desktop v{version}
          </button>
          <p className="download-note">
            适用于 Windows 10/11 (64位)
          </p>
        </div>

        <div className="features-section">
          <h2>主要功能</h2>
          <ul className="features-list">
            <li>🔐 安全的钱包管理</li>
            <li>🤖 AI 智能体交互</li>
            <li>🌐 Web3 集成</li>
            <li>💾 本地数据存储</li>
          </ul>
        </div>
      </div>
    </div>
  )
}

export default AboutView
