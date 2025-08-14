import { STATIC_API_CONFIG, type WSMessage, type DeploymentProgress, type InstanceUpdate } from '../config/api'
import { useNetworkStore } from '@/stores/network'

export interface WebSocketCallbacks {
  onOpen?: () => void
  onClose?: () => void
  onError?: (error: Event) => void
  onDeploymentProgress?: (progress: DeploymentProgress) => void | Promise<void>
  onInstanceUpdate?: (instance: InstanceUpdate) => void
  onMessage?: (message: WSMessage) => void
}

export class WebSocketService {
  private ws: WebSocket | null = null
  public callbacks: WebSocketCallbacks = {}
  private reconnectAttempts = 0
  private maxReconnectAttempts = 5
  private reconnectDelay = 1000
  private pingInterval: number | null = null
  private isConnecting = false
  private currentNetworkId: string | null = null

  constructor(callbacks: WebSocketCallbacks = {}) {
    this.callbacks = callbacks
  }

  private getWebSocketURL(): string {
    // Always use the IPC UI server's WebSocket URL, not the network's WebSocket URL
    // The network's wsUrl is for blockchain WebSocket connections, not UI progress updates
    return STATIC_API_CONFIG.wsURL
  }

  private shouldReconnectForNetworkChange(): boolean {
    try {
      const networkStore = useNetworkStore()
      const currentNetworkId = networkStore.selectedNetwork?.id

      if (this.currentNetworkId !== currentNetworkId) {
        this.currentNetworkId = currentNetworkId || null
        return true
      }
    } catch {
      // Store not available
    }

    return false
  }

  connect(): Promise<void> {
    // Check if we need to reconnect due to network change
    if (this.ws && this.ws.readyState === WebSocket.OPEN && this.shouldReconnectForNetworkChange()) {
      console.log('Network changed, reconnecting WebSocket...')
      this.disconnect()
    }

    if (this.isConnecting || (this.ws && this.ws.readyState === WebSocket.OPEN)) {
      return Promise.resolve()
    }

    this.isConnecting = true
    const wsURL = this.getWebSocketURL()
    console.log('Connecting to WebSocket:', wsURL)

    return new Promise((resolve, reject) => {
      try {
        this.ws = new WebSocket(wsURL)

        this.ws.onopen = () => {
          console.log('WebSocket connected')
          this.isConnecting = false
          this.reconnectAttempts = 0
          this.startPing()
          this.callbacks.onOpen?.()
          resolve()
        }

        this.ws.onclose = (event) => {
          console.log('WebSocket disconnected:', event.code, event.reason)
          this.isConnecting = false
          this.stopPing()
          this.callbacks.onClose?.()

          // Auto-reconnect if not intentionally closed
          if (event.code !== 1000 && this.reconnectAttempts < this.maxReconnectAttempts) {
            this.scheduleReconnect()
          }
        }

        this.ws.onerror = (error) => {
          console.error('WebSocket error:', error)
          this.isConnecting = false
          this.callbacks.onError?.(error)
          reject(error)
        }

        this.ws.onmessage = (event) => {
          try {
            console.log('WebSocket message received:', event.data)
            const message: WSMessage = JSON.parse(event.data)
            //console.log('WebSocket message received:', message)

            // Handle specific message types
            if (message.type === 'deployment_progress' && message.data) {
              this.callbacks.onDeploymentProgress?.(message.data)
            } else if (message.type === 'instance_update' && message.data) {
              this.callbacks.onInstanceUpdate?.(message.data)
            } else if (message.type === 'pong') {
              // Handle pong response - just log for debugging
              console.debug('Received pong response')
              return // Don't call generic handler for pong
            }

            // Call generic message handler
            this.callbacks.onMessage?.(message)
          } catch (error) {
            console.error('Failed to parse WebSocket message:', error, event.data)
          }
        }

        // Connection timeout
        setTimeout(() => {
          if (this.isConnecting) {
            this.ws?.close()
            this.isConnecting = false
            reject(new Error('WebSocket connection timeout'))
          }
        }, 10000)

      } catch (error) {
        this.isConnecting = false
        console.error('Failed to create WebSocket:', error)
        reject(error)
      }
    })
  }

  disconnect(): void {
    this.stopPing()

    if (this.ws) {
      this.ws.close(1000, 'Client disconnect')
      this.ws = null
    }

    this.reconnectAttempts = this.maxReconnectAttempts // Prevent auto-reconnect
  }

  send(message: WSMessage): boolean {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      try {
        this.ws.send(JSON.stringify(message))
        console.log('WebSocket message sent:', message)
        return true
      } catch (error) {
        console.error('Failed to send WebSocket message:', error)
        return false
      }
    } else {
      console.warn('WebSocket is not connected, cannot send message:', message)
      return false
    }
  }

  private scheduleReconnect(): void {
    this.reconnectAttempts++
    const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1) // Exponential backoff

    console.log(`Scheduling WebSocket reconnect attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts} in ${delay}ms`)

    setTimeout(() => {
      if (this.reconnectAttempts <= this.maxReconnectAttempts) {
        this.connect().catch(error => {
          console.error('WebSocket reconnect failed:', error)
        })
      }
    }, delay)
  }

  private startPing(): void {
    this.pingInterval = window.setInterval(() => {
      if (this.ws && this.ws.readyState === WebSocket.OPEN) {
        this.send({ type: 'ping' })
      }
    }, 30000) // Ping every 30 seconds
  }

  private stopPing(): void {
    if (this.pingInterval) {
      clearInterval(this.pingInterval)
      this.pingInterval = null
    }
  }

  get isConnected(): boolean {
    return this.ws !== null && this.ws.readyState === WebSocket.OPEN
  }

  get connectionState(): number {
    return this.ws?.readyState ?? WebSocket.CLOSED
  }

  // Specific message senders for backward compatibility
  subscribeToDeployment(deploymentId: string): boolean {
    return this.send({
      type: 'subscribe_deployment',
      data: { deployment_id: deploymentId }
    })
  }

  subscribeToInstance(instanceId: string): boolean {
    return this.send({
      type: 'subscribe_instance',
      data: { instance_id: instanceId }
    })
  }
}

// Create a singleton instance
export const wsService = new WebSocketService()