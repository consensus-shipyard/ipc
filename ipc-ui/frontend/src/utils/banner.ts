/**
 * ASCII Art Banner Utility for IPC UI
 * Displays colorful banners in console and UI
 */

/**
 * Displays a colorful IPC ASCII art banner in the browser console
 */
export function displayConsoleBanner(): void {
  // Clear console for a clean start
  console.clear()

  const banner = `
%c██╗██████╗  ██████╗    ██╗   ██╗██╗
%c██║██╔══██╗██╔════╝    ██║   ██║██║
%c██║██████╔╝██║         ██║   ██║██║
%c██║██╔═══╝ ██║         ██║   ██║██║
%c██║██║     ╚██████╗    ╚██████╔╝██║
%c╚═╝╚═╝      ╚═════╝     ╚═════╝ ╚═╝

%c🌌 InterPlanetary Consensus Framework
%c🚀 Horizontal Scalability Through Subnet Deployment
%c⚡ Building the Future of Decentralized Networks

%c━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
%c🎯 Status: %cInitializing...
%c🔗 Network: %cReady to connect
%c📡 Subnets: %cLoading...
%c━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
`

  console.log(
    banner,
    // IPC ASCII art - gradient from purple to blue to cyan
    'color: #8b5cf6; font-weight: bold; font-size: 14px;',
    'color: #7c3aed; font-weight: bold; font-size: 14px;',
    'color: #6366f1; font-weight: bold; font-size: 14px;',
    'color: #3b82f6; font-weight: bold; font-size: 14px;',
    'color: #06b6d4; font-weight: bold; font-size: 14px;',
    'color: #0891b2; font-weight: bold; font-size: 14px;',
    // Subtitle text in green
    'color: #10b981; font-weight: bold; font-size: 12px;',
    'color: #059669; font-weight: normal; font-size: 11px;',
    'color: #047857; font-weight: normal; font-size: 11px;',
    // Separator
    'color: #6b7280; font-size: 10px;',
    // Status lines
    'color: #6b7280; font-size: 10px;',
    'color: #f59e0b; font-weight: bold;',
    'color: #6b7280; font-size: 10px;',
    'color: #10b981; font-weight: bold;',
    'color: #6b7280; font-size: 10px;',
    'color: #f59e0b; font-weight: bold;',
    'color: #6b7280; font-size: 10px;'
  )
}

/**
 * Gets the ASCII art as plain text for UI display
 */
export function getAsciiArt(): string {
  return `██╗██████╗  ██████╗    ██╗   ██╗██╗
██║██╔══██╗██╔════╝    ██║   ██║██║
██║██████╔╝██║         ██║   ██║██║
██║██╔═══╝ ██║         ██║   ██║██║
██║██║     ╚██████╗    ╚██████╔╝██║
╚═╝╚═╝      ╚═════╝     ╚═════╝ ╚═╝`
}

/**
 * Gets animated text lines for the UI banner
 */
export function getBannerLines(): string[] {
  return [
    '🌌 InterPlanetary Consensus Framework',
    '🚀 Horizontal Scalability Through Subnet Deployment',
    '⚡ Building the Future of Decentralized Networks'
  ]
}

/**
 * Updates console status during app initialization
 */
export function updateConsoleStatus(status: string, details?: string): void {
  const timestamp = new Date().toLocaleTimeString()
  console.log(
    `%c[${timestamp}] %c${status}%c${details ? ` - ${details}` : ''}`,
    'color: #6b7280; font-size: 10px;',
    'color: #10b981; font-weight: bold;',
    'color: #6b7280; font-size: 10px;'
  )
}