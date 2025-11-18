import React, { useCallback, useEffect, useMemo, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useI18n } from '@/hooks/useI18n'
import { mapChainIdToBackendChain } from '@/hooks/useAgentChat'
import { blockchainService } from '@/services/blockchainService'
import { walletService } from '@/services/walletService'
import WalletConnect from '@/components/wallet/WalletConnect'
import WalletOverview from '@/components/wallet/WalletOverview'
import NetworkSelector from '@/components/wallet/NetworkSelector'
import TransactionList from '@/components/wallet/TransactionList'
import ContractWallet from '@/components/wallet/ContractWallet'
import SignatureModal from '@/components/wallet/SignatureModal'
import AgentWallets from '@/components/wallet/AgentWallets'
import './WalletManager.css'

const networks = [
  {
    chainId: '0xaa36a7',
    name: 'Ethereum Sepolia',
    type: 'Testnet',
    icon: '🔷',
    rpcUrl: 'https://sepolia.infura.io/v3/',
  },
  {
    chainId: '0x14a34',
    name: 'Base Sepolia',
    type: 'Testnet',
    icon: '🔵',
    rpcUrl: 'https://sepolia.base.org',
  },
  {
    chainId: '0x13882',
    name: 'Polygon Amoy',
    type: 'Testnet',
    icon: '🟣',
    rpcUrl: 'https://rpc-amoy.polygon.technology',
  },
  {
    chainId: '0x1',
    name: 'Ethereum Mainnet',
    type: 'Mainnet',
    icon: '💎',
    rpcUrl: 'https://mainnet.infura.io/v3/',
  },
  {
    chainId: '0x2105',
    name: 'Base Mainnet',
    type: 'Mainnet',
    icon: '🔷',
    rpcUrl: 'https://mainnet.base.org',
  },
]

const defaultSignatureRequest = {
  from: '',
  to: '',
  value: '0',
  token: 'ETH',
  gasFee: '0',
}

const WalletManager = () => {
  const navigate = useNavigate()
  const { t } = useI18n()

  const [isDarkMode, setIsDarkMode] = useState(false)
  const [connectedWallet, setConnectedWallet] = useState(null)
  const [currentNetwork, setCurrentNetwork] = useState('0x1')
  const [transactions, setTransactions] = useState([])
  const [contractWallet, setContractWallet] = useState(null)
  const [signatureRequest, setSignatureRequest] = useState(null)
  const [isSigning, setIsSigning] = useState(false)
  const [isRefreshing, setIsRefreshing] = useState(false)
  const [ethPrice] = useState(2000)
  const [agentWallets, setAgentWallets] = useState([])
  const [supportedTokens, setSupportedTokens] = useState([])

  const getNetworkName = useCallback((chainId) => {
    const network = networks.find((network) => network.chainId === chainId)
    return network ? network.name : 'Unknown Network'
  }, [])

  const goBack = useCallback(() => {
    navigate('/')
  }, [navigate])

  const toggleDarkMode = useCallback(() => {
    setIsDarkMode((prev) => {
      const next = !prev
      if (typeof localStorage !== 'undefined') {
        localStorage.setItem('alou-theme', next ? 'dark' : 'light')
      }
      return next
    })
  }, [])

  const updateConnectedWallet = useCallback((address, chainId, balanceInfo) => {
    setConnectedWallet({
      address,
      ethBalance: balanceInfo?.balance || '0.0',
      tokenBalances: {},
    })
    setCurrentNetwork(chainId)
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('wallet_address', address)
      localStorage.setItem('wallet_chain_id', chainId)
      localStorage.setItem('wallet_type', 'metamask')
    }
    if (typeof window !== 'undefined') {
      window.dispatchEvent(new CustomEvent('wallet-changed', { detail: { address } }))
    }
  }, [])

  const loadTransactions = useCallback(
    async (addressParam) => {
      setTransactions((prev) => prev)
      const address = addressParam || connectedWallet?.address
      if (!address) return

      setTransactions([
        {
          hash: '0x1234...5678',
          type: 'send',
          from: address,
          to: '0xabcd...efgh',
          value: '0.1',
          token: 'ETH',
          timestamp: Date.now() - 3600000,
          status: 'confirmed',
        },
        {
          hash: '0x8765...4321',
          type: 'receive',
          from: '0xijkl...mnop',
          to: address,
          value: '0.05',
          token: 'ETH',
          timestamp: Date.now() - 7200000,
          status: 'confirmed',
        },
      ])
    },
    [connectedWallet?.address],
  )

  const loadAgentWallets = useCallback(async () => {
    try {
      const sessionId =
        (typeof localStorage !== 'undefined' && localStorage.getItem('session_id')) || 'default'
      const wallets = await blockchainService.listAgentWallets(sessionId)
      setAgentWallets(wallets || [])
    } catch (error) {
      console.error('Failed to load agent wallets:', error)
    }
  }, [])

  const resolveBackendChain = useCallback((chainId, chainHint) => {
    if (chainHint && chainHint.trim()) {
      return chainHint
    }
    const mapped = mapChainIdToBackendChain(chainId)
    return mapped || 'ethereum'
  }, [])

  const loadSupportedTokens = useCallback(
    async (chainId, chainHint) => {
      const backendChain = resolveBackendChain(chainId, chainHint)
      try {
        const response = await blockchainService.listSupportedTokens(backendChain)
        const tokens = response?.tokens || []
        setSupportedTokens(tokens)
        return { tokens, backendChain }
      } catch (error) {
        console.error('Failed to load supported tokens:', error)
        setSupportedTokens([])
        return { tokens: [], backendChain }
      }
    },
    [resolveBackendChain],
  )

  const loadTokenBalances = useCallback(
    async (address, chainId, chainHint, tokensOverride) => {
      if (!address) return
      const backendChain = resolveBackendChain(chainId, chainHint)
      const tokens = tokensOverride && tokensOverride.length > 0 ? tokensOverride : supportedTokens

      if (!tokens.length) {
        setConnectedWallet((prev) =>
          prev
            ? {
                ...prev,
                tokenBalances: {},
              }
            : prev,
        )
        return
      }

      const balances = {}

      await Promise.all(
        tokens.map(async (token) => {
          try {
            const result = await blockchainService.getTokenBalance(address, backendChain, token.address)
            balances[token.symbol] = {
              balance: result?.balance ?? '0',
              normalizedBalance: result?.normalized_balance ?? null,
              rawBalance: result?.raw_balance ?? null,
              decimals: result?.decimals ?? token.decimals,
              tokenAddress: result?.token_address ?? token.address,
              timestamp: result?.timestamp,
            }
          } catch (error) {
            console.error(`Failed to load ${token.symbol} balance:`, error)
            balances[token.symbol] = {
              balance: '0',
              normalizedBalance: null,
              rawBalance: null,
              decimals: token.decimals,
              tokenAddress: token.address,
            }
          }
        }),
      )

      setConnectedWallet((prev) =>
        prev
          ? {
              ...prev,
              tokenBalances: balances,
            }
          : prev,
      )
    },
    [resolveBackendChain, supportedTokens],
  )

  const connectMetaMask = useCallback(
    async ({ forceSelect = false, silent = false } = {}) => {
      try {
        if (!walletService.isWalletAvailable()) {
          if (!silent) {
            alert(t('installMetaMask'))
          }
          return false
        }

        let accounts = []
        if (forceSelect) {
          accounts = await walletService.requestAccounts({ forceSelect: true })
        } else {
          accounts = await walletService.getAccounts()
          if (!accounts.length && !silent) {
            accounts = await walletService.requestAccounts()
          }
        }

        if (!accounts || accounts.length === 0) {
          if (!silent) {
            alert(t('connectionFailed'))
          }
          return false
        }

        const address = accounts[0]
        const chainId = await walletService.getCurrentChainId()
        const balanceInfo = await blockchainService.getBalance(address)

        updateConnectedWallet(address, chainId, balanceInfo)
        await loadTransactions(address)
        const { tokens, backendChain } = await loadSupportedTokens(chainId)
        await loadTokenBalances(address, chainId, backendChain, tokens)
        await loadAgentWallets()
        return true
      } catch (error) {
        if (!silent) {
          if (error?.code === 4001) {
            alert(t('connectionFailed'))
          } else {
            console.error('Failed to connect MetaMask:', error)
            alert(t('connectionFailed'))
          }
        } else if (error?.code !== 4001) {
          console.error('Failed to connect MetaMask silently:', error)
        }
        return false
      }
    },
    [
      loadAgentWallets,
      loadSupportedTokens,
      loadTokenBalances,
      loadTransactions,
      t,
      updateConnectedWallet,
    ],
  )

  const connectWalletConnect = useCallback(() => {
    alert(t('walletConnectComingSoon'))
  }, [t])

  const checkWalletConnection = useCallback(() => {
    const savedAddress =
      typeof localStorage !== 'undefined' ? localStorage.getItem('wallet_address') : null
    if (savedAddress) {
      void connectMetaMask({ silent: true })
    }
  }, [connectMetaMask])

  const disconnectWallet = useCallback(() => {
    setConnectedWallet(null)
    setSupportedTokens([])
    if (typeof localStorage !== 'undefined') {
      localStorage.removeItem('wallet_address')
      localStorage.removeItem('wallet_type')
      localStorage.removeItem('wallet_chain_id')
    }
    if (typeof window !== 'undefined') {
      window.dispatchEvent(new CustomEvent('wallet-changed', { detail: { address: null } }))
    }
  }, [])

  const switchToNetwork = useCallback(async (network) => {
    if (typeof window === 'undefined' || !window.ethereum) return
    try {
      await window.ethereum.request({
        method: 'wallet_switchEthereumChain',
        params: [{ chainId: network.chainId }],
      })
      setCurrentNetwork(network.chainId)
    } catch (error) {
      if (error?.code === 4902) {
        try {
          await window.ethereum.request({
            method: 'wallet_addEthereumChain',
            params: [
              {
                chainId: network.chainId,
                chainName: network.name,
                rpcUrls: [network.rpcUrl],
              },
            ],
          })
          setCurrentNetwork(network.chainId)
        } catch (addError) {
          console.error('Failed to add network:', addError)
        }
      }
    }
  }, [])

  const refreshTransactions = useCallback(async () => {
    setIsRefreshing(true)
    await loadTransactions()
    setTimeout(() => {
      setIsRefreshing(false)
    }, 1000)
  }, [loadTransactions])

  const viewTransaction = useCallback((tx) => {
    window.open(`https://etherscan.io/tx/${tx.hash}`, '_blank', 'noopener')
  }, [])

  const createContractWallet = useCallback(() => {
    try {
      setContractWallet({
        address: '0xContract...Wallet',
        balance: '0.0',
      })
      alert(t('contractWalletCreated'))
    } catch (error) {
      console.error('Failed to create contract wallet:', error)
      alert(t('createFailed'))
    }
  }, [t])

  const depositToContract = useCallback(() => {
    const amount = window.prompt(t('enterDepositAmount'))
    if (!amount || !connectedWallet) return
    setSignatureRequest({
      from: connectedWallet.address,
      to: contractWallet?.address || '',
      value: amount,
      token: 'ETH',
      gasFee: '0.001',
    })
  }, [connectedWallet, contractWallet?.address, t])

  const withdrawFromContract = useCallback(() => {
    const amount = window.prompt(t('enterWithdrawAmount'))
    if (!amount || !connectedWallet) return
    setSignatureRequest({
      from: contractWallet?.address || '',
      to: connectedWallet.address,
      value: amount,
      token: 'ETH',
      gasFee: '0.001',
    })
  }, [connectedWallet, contractWallet?.address, t])

  const manageContract = useCallback(() => {
    alert(t('contractManagementComingSoon'))
  }, [t])

  const confirmSignature = useCallback(async () => {
    if (!signatureRequest || typeof window === 'undefined' || !window.ethereum) return

    setIsSigning(true)
    try {
      const txHash = await window.ethereum.request({
        method: 'eth_sendTransaction',
        params: [
          {
            from: signatureRequest.from,
            to: signatureRequest.to,
            value: `0x${(parseFloat(signatureRequest.value) * 1e18).toString(16)}`,
          },
        ],
      })

      alert(`${t('transactionSent')}: ${txHash}`)
      setSignatureRequest(null)
      await connectMetaMask()
      await loadTransactions()
    } catch (error) {
      console.error('Transaction failed:', error)
      alert(t('transactionFailed'))
    } finally {
      setIsSigning(false)
    }
  }, [connectMetaMask, loadTransactions, signatureRequest, t])

  const cancelSignature = useCallback(() => {
    setSignatureRequest(null)
  }, [])

  const refreshWalletBalance = useCallback(async () => {
    if (!connectedWallet?.address) return
    try {
      const balanceInfo = await blockchainService.getBalance(connectedWallet.address)
      if (balanceInfo) {
        setConnectedWallet((prev) =>
          prev
            ? {
                ...prev,
                ethBalance: balanceInfo.balance,
              }
            : prev,
        )
      }
      await loadTokenBalances(connectedWallet.address, currentNetwork)
    } catch (error) {
      console.error('Failed to refresh balance:', error)
    }
  }, [connectedWallet?.address, currentNetwork, loadTokenBalances])

  const handleNetworkChanged = useCallback(
    (event) => {
      if (!event?.detail) return
      setCurrentNetwork(event.detail.chainId)
      refreshWalletBalance()
      console.log(`Switched to ${getNetworkName(event.detail.chainId)}`)
    },
    [getNetworkName, refreshWalletBalance],
  )

  useEffect(() => {
    if (typeof localStorage !== 'undefined') {
      const savedTheme = localStorage.getItem('alou-theme')
      setIsDarkMode(
        savedTheme === 'dark' ||
          (typeof window !== 'undefined' &&
            window.matchMedia('(prefers-color-scheme: dark)').matches),
      )
    }

    checkWalletConnection()

    if (typeof window !== 'undefined') {
      window.addEventListener('network-changed', handleNetworkChanged)
    }

    let chainChangedHandler
    if (typeof window !== 'undefined' && window.ethereum) {
      chainChangedHandler = (chainId) => {
        setCurrentNetwork(chainId)
        if (typeof localStorage !== 'undefined') {
          localStorage.setItem('wallet_chain_id', chainId)
        }
      }
      window.ethereum.on('chainChanged', chainChangedHandler)
    }

    return () => {
      if (typeof window !== 'undefined') {
        window.removeEventListener('network-changed', handleNetworkChanged)
        if (chainChangedHandler && window.ethereum) {
          window.ethereum.removeListener('chainChanged', chainChangedHandler)
        }
      }
    }
  }, [checkWalletConnection, handleNetworkChanged])

  const networkName = useMemo(
    () => getNetworkName(currentNetwork),
    [currentNetwork, getNetworkName],
  )

  return (
    <div className={`wallet-manager${isDarkMode ? ' dark-mode' : ''}`}>
      <nav className="top-nav">
        <div className="nav-content">
          <div className="logo-section">
            <div className="logo">💰</div>
            <h1 className="app-title">{t('walletManagement')}</h1>
          </div>
          <div className="nav-controls">
            <button
              type="button"
              onClick={toggleDarkMode}
              className="theme-toggle"
              title={t('theme')}
            >
              {isDarkMode ? '🌞' : '🌙'}
            </button>
            <button type="button" onClick={goBack} className="back-btn">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M20,11V13H8L13.5,18.5L12.08,19.92L4.16,12L12.08,4.08L13.5,5.5L8,11H20Z" />
              </svg>
              {t('back')}
            </button>
          </div>
        </div>
      </nav>

      <div className="wallet-container">
        <div className="wallet-content">
          {!connectedWallet ? (
            <WalletConnect
              onConnectMetaMask={() => connectMetaMask({ forceSelect: true })}
              onConnectWalletConnect={connectWalletConnect}
            />
          ) : (
            <div className="wallet-info-section">
              <WalletOverview
                wallet={connectedWallet}
                currentNetwork={currentNetwork}
                networkName={networkName}
                ethPrice={ethPrice}
                supportedTokens={supportedTokens}
                onSwitchWallet={() => connectMetaMask({ forceSelect: true })}
                onDisconnect={disconnectWallet}
              />

              <NetworkSelector
                networks={networks}
                currentNetwork={currentNetwork}
                onSwitchNetwork={switchToNetwork}
              />

              <TransactionList
                transactions={transactions}
                isRefreshing={isRefreshing}
                onRefresh={refreshTransactions}
                onViewTransaction={viewTransaction}
              />

              <AgentWallets wallets={agentWallets} />

              <ContractWallet
                contractWallet={contractWallet}
                onCreate={createContractWallet}
                onDeposit={depositToContract}
                onWithdraw={withdrawFromContract}
                onManage={manageContract}
              />

              <SignatureModal
                show={!!signatureRequest}
                request={signatureRequest || defaultSignatureRequest}
                isSigning={isSigning}
                onConfirm={confirmSignature}
                onCancel={cancelSignature}
              />
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

export default WalletManager
