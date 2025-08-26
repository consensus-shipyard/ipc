/**
 * Chain statistics service
 */
import { apiService } from '@/services/api'
import type { ChainStats, SubnetStatus } from '@/types/subnet'

export class StatsService {
  /**
   * Get chain statistics for a subnet
   */
  static async getChainStats(subnetId: string): Promise<{ data: { data: ChainStats } }> {
    return apiService.getSubnetStats(decodeURIComponent(subnetId))
  }

  /**
   * Get subnet status
   */
  static async getSubnetStatus(subnetId: string): Promise<{ data: { data: SubnetStatus } }> {
    return apiService.getSubnetStatus(decodeURIComponent(subnetId))
  }

  /**
   * Get both chain stats and status in parallel
   */
  static async getAllStats(subnetId: string) {
    const [statsResponse, statusResponse] = await Promise.all([
      this.getChainStats(subnetId),
      this.getSubnetStatus(subnetId)
    ])

    return {
      chainStats: statsResponse.data?.data || null,
      subnetStatus: statusResponse.data?.data || null
    }
  }
}
