import { API_CONFIG, WSMessage, DeploymentProgress } from '../config/api'

export interface WebSocketCallbacks {
  onOpen?: () => void
  onClose?: () => void
  onError?: (error: Event) => void
  onDeploymentProgress?: (progress: DeploymentProgress) => void
  onInstanceUpdate?: (instance: any) => void
  onMessage?: (message: WSMessage) => void
}

export class WebSocketService {
  private ws: WebSocket | null = null
  private callbacks: WebSocketCallbacks = {}
  private reconnectAttempts = 0
  private maxReconnectAttempts = 5
  private reconnectDelay = 1000
  private pingInterval: NodeJS.Timeout | null = null
  private isConnecting = false

  constructor(callbacks: WebSocketCallbacks = {}) {
    this.callbacks = callbacks
  }

  connect(): Promise<void> {
    if (this.isConnecting || (this.ws && this.ws.readyState === WebSocket.OPEN)) {
      return Promise.resolve()
    }

    this.isConnecting = true
    console.log('Connecting to WebSocket:', API_CONFIG.wsURL)

    return new Promise((resolve, reject) => {
      try {
        this.ws = new WebSocket(API_CONFIG.wsURL)

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

          // Attempt to reconnect if not closed intentionally
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
            const message: WSMessage = JSON.parse(event.data)
            console.log('WebSocket message received:', message)

            // Handle specific message types
            switch (message.type) {
              case 'deployment_progress':
                this.callbacks.onDeploymentProgress?.(message.data as DeploymentProgress)
                break
              case 'instance_update':
                this.callbacks.onInstanceUpdate?.(message.data)
                break
              case 'pong':
                // Handle ping-pong for connection health
                break
              default:
                this.callbacks.onMessage?.(message)
            }
          } catch (error) {
            console.error('Error parsing WebSocket message:', error)
          }
        }

      } catch (error) {
        this.isConnecting = false
        reject(error)
      }
    })
  }

  disconnect() {
    console.log('Disconnecting WebSocket')
    this.stopPing()

    if (this.ws) {
      this.ws.close(1000, 'Client disconnect')
      this.ws = null
    }
  }

  send(message: WSMessage) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      console.log('Sending WebSocket message:', message)
      this.ws.send(JSON.stringify(message))
    } else {
      console.warn('WebSocket not connected, cannot send message:', message)
    }
  }

  // Specific message senders
  subscribeToDeployment(deploymentId: string) {
    this.send({
      type: 'subscribe_deployment',
      data: { deployment_id: deploymentId }
    })
  }

  subscribeToInstance(instanceId: string) {
    this.send({
      type: 'subscribe_instance',
      data: { instance_id: instanceId }
    })
  }

  private scheduleReconnect() {
    this.reconnectAttempts++
    const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1) // Exponential backoff

    console.log(`Attempting to reconnect in ${delay}ms (attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts})`)

    setTimeout(() => {
      if (this.reconnectAttempts <= this.maxReconnectAttempts) {
        this.connect().catch((error) => {
          console.error('Reconnection failed:', error)
        })
      }
    }, delay)
  }

  private startPing() {
    this.pingInterval = setInterval(() => {
      this.send({ type: 'ping' })
    }, 30000) // Ping every 30 seconds
  }

  private stopPing() {
    if (this.pingInterval) {
      clearInterval(this.pingInterval)
      this.pingInterval = null
    }
  }

  get isConnected(): boolean {
    return this.ws !== null && this.ws.readyState === WebSocket.OPEN
  }

  get connectionState(): string {
    if (!this.ws) return 'disconnected'

    switch (this.ws.readyState) {
      case WebSocket.CONNECTING: return 'connecting'
      case WebSocket.OPEN: return 'connected'
      case WebSocket.CLOSING: return 'closing'
      case WebSocket.CLOSED: return 'closed'
      default: return 'unknown'
    }
  }
}

// Create a singleton instance
export const wsService = new WebSocketService()