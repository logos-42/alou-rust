import React, { useState, useEffect } from 'react'
import ipfsService from '@/services/ipfsService'
import './IpfsStatus.css'

const IpfsStatus = () => {
  const [isRunning, setIsRunning] = useState(false)
  const [nodeInfo, setNodeInfo] = useState(null)
  const [loading, setLoading] = useState(false)
  const [downloading, setDownloading] = useState(false)
  const [kuboInstalled, setKuboInstalled] = useState(false)
  const [error, setError] = useState(null)

  useEffect(() => {
    checkKuboInstalled()
    checkNodeStatus()
    // Check status every 5 seconds
    const interval = setInterval(checkNodeStatus, 5000)
    return () => clearInterval(interval)
  }, [])

  const checkKuboInstalled = async () => {
    const installed = await ipfsService.checkKuboInstalled()
    setKuboInstalled(installed)
  }

  const checkNodeStatus = async () => {
    try {
      const result = await ipfsService.getNodeInfo()
      if (result.success) {
        setIsRunning(true)
        setNodeInfo(result.info)
        setError(null)
      } else {
        setIsRunning(false)
        setNodeInfo(null)
      }
    } catch (err) {
      setIsRunning(false)
      setNodeInfo(null)
    }
  }

  const handleDownloadKubo = async () => {
    setDownloading(true)
    setError(null)
    try {
      const result = await ipfsService.downloadKubo()
      if (result.success) {
        setKuboInstalled(true)
        // Auto-start after download
        await handleStart(false)
      } else {
        setError(result.error)
      }
    } catch (err) {
      setError(err.toString())
    } finally {
      setDownloading(false)
    }
  }

  const handleStart = async (autoDownload = true) => {
    setLoading(true)
    setError(null)
    try {
      const result = await ipfsService.startNode(autoDownload)
      if (result.success) {
        setKuboInstalled(true)
        await checkNodeStatus()
      } else {
        setError(result.error)
        // If binary not found, show download option
        if (result.error.includes('not found')) {
          setKuboInstalled(false)
        }
      }
    } catch (err) {
      setError(err.toString())
    } finally {
      setLoading(false)
    }
  }

  const handleStop = async () => {
    setLoading(true)
    setError(null)
    try {
      const result = await ipfsService.stopNode()
      if (result.success) {
        setIsRunning(false)
        setNodeInfo(null)
      } else {
        setError(result.error)
      }
    } catch (err) {
      setError(err.toString())
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="ipfs-status">
      <div className="ipfs-status-header">
        <h3>IPFS 节点状态</h3>
        <span className={`status-indicator ${isRunning ? 'running' : 'stopped'}`}>
          {isRunning ? '●' : '○'}
        </span>
      </div>

      {error && (
        <div className="ipfs-error">
          <p>{error}</p>
        </div>
      )}

      {isRunning && nodeInfo && (
        <div className="ipfs-info">
          <div className="info-item">
            <label>Peer ID:</label>
            <code>{nodeInfo.ID || 'N/A'}</code>
          </div>
          {nodeInfo.Addresses && nodeInfo.Addresses.length > 0 && (
            <div className="info-item">
              <label>地址:</label>
              <code>{nodeInfo.Addresses[0]}</code>
            </div>
          )}
        </div>
      )}

      {!kuboInstalled && (
        <div className="ipfs-download">
          <p className="download-notice">
            Kubo (IPFS) 二进制未安装。首次使用需要下载（约 50-100MB）。
          </p>
          <button
            type="button"
            className="download-btn"
            onClick={handleDownloadKubo}
            disabled={downloading}
          >
            {downloading ? '下载中...' : '下载 Kubo 二进制'}
          </button>
        </div>
      )}

      <div className="ipfs-actions">
        {!isRunning ? (
          <button
            type="button"
            className="start-btn"
            onClick={() => handleStart(true)}
            disabled={loading || !kuboInstalled}
          >
            {loading ? '启动中...' : '启动 IPFS 节点'}
          </button>
        ) : (
          <button
            type="button"
            className="stop-btn"
            onClick={handleStop}
            disabled={loading}
          >
            {loading ? '停止中...' : '停止 IPFS 节点'}
          </button>
        )}
      </div>
    </div>
  )
}

export default IpfsStatus

