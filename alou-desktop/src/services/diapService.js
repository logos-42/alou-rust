import { invoke } from '@tauri-apps/api/core'

const DEFAULT_IPFS_API = import.meta.env.VITE_IPFS_API_URL || 'http://127.0.0.1:5001'
const DEFAULT_IPFS_GATEWAY =
  import.meta.env.VITE_IPFS_GATEWAY_URL || 'http://127.0.0.1:8080'

class DiapService {
  async createLocalIdentity({ name, description, ipfsApiUrl, ipfsGatewayUrl } = {}) {
    const response = await invoke('create_local_diap_identity', {
      params: {
        agent_name: name,
        agent_description: description,
        ipfs_api_url: ipfsApiUrl || DEFAULT_IPFS_API,
        ipfs_gateway_url: ipfsGatewayUrl || DEFAULT_IPFS_GATEWAY,
      },
    })
    return response
  }
}

export default new DiapService()

