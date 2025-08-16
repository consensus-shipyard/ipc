/**
 * Subnet configuration service
 */
import { apiService } from '@/services/api'
import type { SubnetInstance, TestTransactionData } from '@/types/subnet'

export class ConfigService {
  /**
   * Get subnet instance details
   */
  static async getInstance(subnetId: string): Promise<{ data: SubnetInstance }> {
    const decodedId = decodeURIComponent(subnetId)
    return apiService.getInstance(decodedId)
  }

  /**
   * Approve a subnet
   */
  static async approveSubnet(subnetId: string, gatewayOwnerAddress: string) {
    return apiService.approveSubnet(subnetId, gatewayOwnerAddress)
  }

  /**
   * Send a test transaction
   */
  static async sendTestTransaction(subnetId: string, data: TestTransactionData) {
    return apiService.sendTestTransaction(decodeURIComponent(subnetId), data)
  }

  /**
   * Get gateway owner address
   */
  static async getGatewayOwner(instance: SubnetInstance): Promise<string> {
    const defaultAddress = '0x0a36d7c34ba5523d5bf783bb47f62371e52e0298'

    if (!instance) return defaultAddress

    try {
      // Try to get gateway information from the API
      const gatewaysResponse = await fetch('/api/gateways')
      const gatewaysResult = await gatewaysResponse.json()

      if (gatewaysResult && Array.isArray(gatewaysResult)) {
        // Find the gateway that matches this subnet's gateway address
        const gatewayAddr = instance.config?.gateway_addr?.toString()
        if (gatewayAddr) {
          const matchingGateway = gatewaysResult.find((gw: { gateway_address: string; deployer_address: string }) =>
            gw.gateway_address === gatewayAddr
          )
          if (matchingGateway) {
            return matchingGateway.deployer_address
          }
        }
      }
    } catch (err) {
      console.warn('Failed to fetch gateway information:', err)
    }

    // Fallback to config or default address
    return instance.config?.deployer_address || defaultAddress
  }

  /**
   * Export subnet configuration
   */
  static exportConfig(instance: SubnetInstance): void {
    const configData = {
      name: instance.name,
      config: instance.config,
      validators: instance.validators,
      exported_at: new Date().toISOString()
    }

    const blob = new Blob([JSON.stringify(configData, null, 2)], {
      type: 'application/json'
    })
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = `${instance.name}-config.json`
    link.click()
    URL.revokeObjectURL(url)
  }
}
