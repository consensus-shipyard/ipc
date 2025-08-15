/**
 * Validator management service
 */
import { apiService } from '@/services/api'
import type {
  AddValidatorData,
  RemoveValidatorData,
  UpdateStakeData,
  FederatedPowerData
} from '@/types/subnet'

export class ValidatorService {
  /**
   * Add a new validator to a subnet
   */
  static async addValidator(data: AddValidatorData) {
    return apiService.addValidator(data)
  }

  /**
   * Remove a validator from a subnet
   */
  static async removeValidator(data: RemoveValidatorData) {
    return apiService.removeValidator(data)
  }

  /**
   * Update validator stake (stake or unstake)
   */
  static async updateStake(data: UpdateStakeData) {
    return apiService.updateValidatorStake(data)
  }

  /**
   * Set federated power for validators
   */
  static async setFederatedPower(data: FederatedPowerData) {
    return apiService.setFederatedPower(data)
  }

  /**
   * Get node configuration for a validator
   */
  static async getNodeConfig(subnetId: string, validatorAddress: string) {
    const encodedSubnetId = encodeURIComponent(subnetId)
    const encodedAddress = encodeURIComponent(validatorAddress)

    const response = await fetch(
      `/api/subnets/${encodedSubnetId}/node-config?validator_address=${encodedAddress}`
    )
    return response.json()
  }

  /**
   * Get node commands for a validator
   */
  static async getNodeCommands(subnetId: string, validatorAddress: string) {
    const encodedSubnetId = encodeURIComponent(subnetId)
    const encodedAddress = encodeURIComponent(validatorAddress)

    const response = await fetch(
      `/api/subnets/${encodedSubnetId}/node-commands?validator_address=${encodedAddress}`
    )
    return response.json()
  }

  /**
   * Get both node config and commands in parallel
   */
  static async getNodeConfigAndCommands(subnetId: string, validatorAddress: string) {
    const [configResponse, commandsResponse] = await Promise.all([
      this.getNodeConfig(subnetId, validatorAddress),
      this.getNodeCommands(subnetId, validatorAddress)
    ])

    if (configResponse.success && commandsResponse.success) {
      return {
        success: true,
        data: {
          validatorAddress,
          configYaml: configResponse.data.config_yaml,
          commands: commandsResponse.data,
          filename: configResponse.data.filename
        }
      }
    }

    return {
      success: false,
      error: configResponse.error || commandsResponse.error || 'Failed to get node configuration'
    }
  }
}
