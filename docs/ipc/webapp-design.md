# IPC WebApp Design Document

## Overview

A web-based user interface for IPC (Inter-Planetary Consensus) that simplifies subnet deployment and management. The app provides a wizard-driven deployment experience with real-time progress tracking and comprehensive instance management.

## Core Features

### 1. Subnet Deployment Wizard
- **Template-driven approach**: Users answer questions to get suggested configurations
- **Multi-step wizard**: Basic config → Advanced → Review → Deploy
- **Real-time progress**: WebSocket-based updates during deployment
- **Save draft configurations**: Allow users to save and resume configurations

### 2. Instance Management Dashboard
- **Hierarchical visualization**: Display subnet relationships (L1 → Subnet → Child Subnet)
- **Instance lifecycle management**: Create → Deploy → Activate → Running → Paused → Stopped
- **Operations**: View status, pause/resume, update configs, manage validators

### 3. Configuration Templates
Template suggestions based on user needs assessment:
- **Network type**: Federated vs Collateral-based
- **Use case**: Development, Testing, Production
- **Performance requirements**: Transaction throughput needs (future-proofing)
- **Security model**: Trust assumptions and validator requirements

## Technical Architecture

### Frontend Stack
- **Framework**: Vue 3 with Composition API
- **Styling**: Tailwind CSS
- **Build Tool**: Vite
- **State Management**: Pinia
- **WebSocket**: Native WebSocket API or socket.io-client
- **Location**: `/ipc-ui/frontend/`

### Backend Stack
- **Language**: Rust
- **Integration**: Built into ipc-cli binary
- **Architecture**: Long-running service with WebSocket server
- **Data Storage**: Local JSON files for configurations and metadata
- **Location**: `/ipc-ui/backend/` or integrated into existing ipc-cli structure

### Communication
- **REST API**: Configuration management, instance queries
- **WebSocket**: Real-time deployment progress, status updates
- **CLI Integration**: Backend reuses existing ipc-cli functionality

## User Experience Design

### Deployment Wizard Flow

#### Step 1: Template Selection
- Quick questionnaire to determine user needs
- Present 3-5 template options with descriptions
- Option to start from scratch

#### Step 2: Basic Configuration (Mandatory Parameters)
- Parent subnet selection
- Minimum validator stake
- Minimum validators count
- Bottom-up checkpoint period
- Permission mode (federated/collateral/static)
- Supply source configuration
- Genesis subnet IPC contracts owner

#### Step 3: Advanced Configuration (Optional)
Organized into collapsible sections:
- **Network Settings**: Active validator limits, cross-message fees
- **Validator Management**: Gater contracts, rewarder contracts
- **Economic Parameters**: Collateral sources, fee structures
- **Genesis Parameters**: Network version, base fee, power scale

#### Step 4: Activation Configuration
Based on permission mode selected:
- **Federated/Static**: Validator public keys and power distribution
- **Collateral**: Validator addresses, collateral amounts, initial balances

#### Step 5: Review & Deploy
- Configuration summary with editable sections
- Deployment progress with real-time updates
- Success/failure handling with detailed error reporting

### Instance Management Dashboard

#### Hierarchical View
```
L1 Network (Ethereum Mainnet)
├── Production Subnet A (Active)
│   ├── Child Subnet A1 (Active)
│   └── Child Subnet A2 (Deploying...)
└── Test Subnet B (Paused)
    └── Child Subnet B1 (Active)
```

#### Instance Details Panel
For each subnet instance:
- **Status**: Active, Pausing, Paused, Deploying, Failed
- **Network Info**: Subnet ID, parent chain, block height
- **Validators**: Current validator set, power distribution, stake amounts
- **Economic**: Total stake, circulating supply, fees collected
- **Contracts**: Gateway address, registry address, versions
- **Performance**: Transaction count, average block time, checkpoint status

#### Available Actions
- **View Logs**: Real-time log streaming
- **Update Configuration**: Modify validator set, economic parameters
- **Pause/Resume**: Temporarily halt subnet operations
- **Export Configuration**: Download subnet-init.yaml for replication

## Configuration Templates

### Template Categories

#### 1. Development Template
- **Use Case**: Local development and testing
- **Configuration**: Federated mode, minimal validators (1-3), low stakes
- **Defaults**: Fast checkpoints, permissive settings

#### 2. Staging Template
- **Use Case**: Pre-production testing
- **Configuration**: Collateral mode, moderate stakes, realistic validator count
- **Defaults**: Production-like settings with lower barriers

#### 3. Production Template
- **Use Case**: Live production deployment
- **Configuration**: Collateral mode, high security, robust validator set
- **Defaults**: Conservative settings, high stakes, longer checkpoint periods

#### 4. Federated Network Template
- **Use Case**: Consortium or private networks
- **Configuration**: Federated mode, known validator set
- **Defaults**: Flexible validator management, controlled access

#### 5. L1 Integration Template
- **Use Case**: Subnets connecting directly to Ethereum/Filecoin mainnet
- **Configuration**: Mainnet parent networks, production-grade security
- **Defaults**: Conservative settings, high gas considerations

#### 6. Testnet Template
- **Use Case**: Development on public testnets (Sepolia, Calibration)
- **Configuration**: Pre-configured testnet parents, moderate security
- **Defaults**: Testnet-optimized settings, reasonable gas costs

#### 7. Multi-token Template
- **Use Case**: ERC-20 based supply or collateral sources
- **Configuration**: Custom token contracts, flexible economics
- **Defaults**: ERC-20 integration, token-specific validations

### Template Selection Questions
1. **"What's your primary use case?"**
   - Development/Testing → Development Template
   - Staging/QA → Staging Template
   - Production Launch → Production Template
   - Private Consortium → Federated Template

2. **"How important is decentralization?"**
   - Very Important → Collateral mode
   - Moderate → Collateral with lower barriers
   - Not Critical → Federated mode

3. **"Expected transaction volume?"**
   - Low (< 1000 tx/day) → Conservative settings
   - Medium (1000-100k tx/day) → Balanced settings
   - High (> 100k tx/day) → Performance-optimized settings

4. **"How many validators do you expect?"**
   - Few (1-10) → Low minimums, simple setup
   - Medium (10-100) → Balanced validator economics
   - Many (100+) → Scalable validator management

## Implementation Phases

### Phase 1: Frontend Foundation (Week 1-2)
- Vue 3 + Tailwind project setup with Vite
- Basic routing and layout structure
- Template selection questionnaire and logic
- Wizard navigation with step validation
- Mock backend responses for development
- Responsive design implementation

### Phase 2: Configuration Wizard (Week 3-4)
- All wizard steps with real-time validation
- Smart defaults and field dependencies
- Form state management with Pinia
- Configuration preview and export
- Mock wallet detection and network discovery
- Error state handling and user feedback

### Phase 3: Backend Service (Week 5-6)
- Rust backend service integrated into ipc-cli
- WebSocket server for real-time communication
- CLI command execution wrapper and progress tracking
- Local JSON file storage for configurations/metadata
- Network discovery and wallet validation APIs
- Basic deployment orchestration

### Phase 4: Instance Dashboard (Week 7-8)
- Hierarchical subnet visualization
- Instance detail panels with real-time data
- Status monitoring and lifecycle management
- WebSocket integration for live updates
- Basic operations (pause, resume, view logs)
- Configuration management (export/import)

### Phase 5: Advanced Features (Week 9-10)
- Complete error recovery and retry system
- Advanced validator management
- Multi-mode deployment (dev/testnet/mainnet)
- Performance optimizations and polish
- Comprehensive testing and documentation
- Production readiness features

## File Structure

```
ipc-ui/
├── frontend/
│   ├── src/
│   │   ├── components/
│   │   │   ├── wizard/
│   │   │   ├── dashboard/
│   │   │   └── common/
│   │   ├── views/
│   │   ├── stores/
│   │   ├── composables/
│   │   └── utils/
│   ├── public/
│   ├── package.json
│   └── vite.config.js
├── backend/ (or integrated into ipc-cli)
└── .gitignore
```

## CLI Integration

### Command: `ipc-cli ui start [OPTIONS]`
- **Basic Usage**: `ipc-cli ui start`
- **With Options**: `ipc-cli ui start --port 3000 --backend-port 8080 --mode development`
- **Available Options**:
  - `--port`: Frontend server port (default: 3000)
  - `--backend-port`: Backend WebSocket/API port (default: 8080)
  - `--mode`: deployment mode (`development`, `testnet`, `mainnet`)
  - `--no-browser`: Don't auto-open browser
- Starts both backend service and frontend dev server
- Opens browser automatically (unless `--no-browser` specified)

### Backend Service Integration
- New `ui` subcommand in ipc-cli
- Reuse existing subnet management functionality
- Maintain compatibility with existing CLI workflows

## Wallet Integration

### Wallet Management Strategy
- **Pre-configuration Required**: Users must configure wallets via CLI before using UI
- **Detection & Guidance**: UI detects missing wallet configuration and guides users through setup
- **Validation Feedback**: Real-time feedback when wallets are properly configured
- **Support for Multiple Wallets**: Handle both EVM and FVM wallet types

### Wallet Setup Flow
1. **Detection**: Check if required wallets exist in keystore
2. **Guidance**: Show CLI commands user needs to run (e.g., `ipc-cli wallet import --wallet-type evm --private-key <key>`)
3. **Validation**: Continuously check for successful wallet import
4. **Confirmation**: Update UI when wallets are properly configured

## Network Discovery & Selection

### Parent Network Management
- **Dropdown Selection**: Show list of detected/configured networks
- **Manual Entry**: Allow users to add new parent networks with format validation
- **Auto-detection**: Scan for available networks and RPC endpoints
- **Network Validation**: Test connectivity and compatibility before allowing selection

### Network Discovery Features
- **Local Networks**: Auto-detect localhost:8545, anvil, hardhat networks
- **Testnet Networks**: Pre-configured Sepolia, Calibration, other common testnets
- **Mainnet Networks**: Ethereum mainnet, Filecoin mainnet with additional confirmations
- **Custom Networks**: User-defined RPC endpoints with connectivity testing

## Error Recovery & Retry System

### Granular Recovery Options
1. **Full Retry**: Restart entire deployment process from beginning
2. **Step Resume**: Continue from failed step (e.g., retry activation after successful deployment)
3. **Manual Intervention**: Provide guided troubleshooting with specific error context
4. **Partial Rollback**: Undo recent changes and retry from known good state

### Error Handling Features
- **Detailed Error Context**: Show exactly what failed and why
- **Suggested Actions**: Provide specific recommendations based on error type
- **Log Integration**: Easy access to relevant logs for debugging
- **Support Export**: Export error state and logs for external troubleshooting

## Validation System

### Real-time Validation
- **Field-level Validation**: Validate inputs as user types (address formats, numeric ranges)
- **Dependency Validation**: Update available options based on other field selections
- **Network Connectivity**: Test RPC endpoints and wallet connectivity in real-time

### Form Submission Validation
- **Complete Configuration Check**: Ensure all required fields are properly configured
- **Cross-field Validation**: Validate relationships between different configuration sections
- **Resource Availability**: Check wallet balances, network connectivity before deployment
- **Confirmation Warnings**: Show warnings for potentially risky configurations

### Smart Defaults & Dependencies
- **Mode-based Fields**: Show/hide fields based on permission mode selection
  - Federated → Validator pubkeys and power arrays
  - Collateral → Validator stakes and balances
  - Static → Fixed validator configuration
- **Token Integration**: Auto-show address fields when ERC-20 options selected
- **Array Synchronization**: Ensure validator arrays (pubkeys, power, stakes) maintain consistent lengths
- **Economic Calculations**: Auto-calculate recommended values based on network size and security requirements

## Deployment Modes

### Development Mode (`--mode development`)
- **Target**: Local networks (localhost:8545, anvil, hardhat)
- **Defaults**: Fast block times, low stakes, permissive settings
- **Validation**: Relaxed validation, development-focused warnings
- **Features**: Easy reset/restart, detailed debugging information

### Testnet Mode (`--mode testnet`)
- **Target**: Public testnets (Sepolia, Calibration, etc.)
- **Defaults**: Realistic but not production-level settings
- **Validation**: Standard validation with testnet considerations
- **Features**: Testnet faucet integration, reasonable gas costs

### Mainnet Mode (`--mode mainnet`)
- **Target**: Production networks (Ethereum mainnet, Filecoin mainnet)
- **Defaults**: Conservative, production-grade settings
- **Validation**: Strict validation with multiple confirmations
- **Features**: Enhanced warnings, cost estimates, security confirmations

## Remaining Questions for Implementation

1. **Template Questions**: Should we also ask about "Expected network lifetime?" (temporary vs permanent) and "Compliance requirements?" (regulatory considerations)?

2. **Hierarchical Visualization**: How deep can the subnet hierarchy go? Should we limit the UI to show only 2-3 levels for clarity?

3. **Validator Management**: For collateral-based subnets, should users be able to invite/approve new validators through the UI, or is this always done externally?

4. **Configuration Persistence**: Should we support importing existing subnet configurations from YAML files, or only export from the UI?

5. **Multi-network Support**: Should the UI support managing subnets across different parent networks (Ethereum, Filecoin, etc.) simultaneously?

6. **Real-time Updates**: Besides deployment progress, what other real-time data should we stream? (Block production, validator changes, economic metrics?)

## Design Status

✅ **Core Architecture Approved**
✅ **CLI Integration Specified**
✅ **Wallet Management Strategy Defined**
✅ **Network Discovery Approach Confirmed**
✅ **Error Recovery System Designed**
✅ **Validation Strategy Complete**
✅ **Deployment Modes Specified**
✅ **Template Categories Expanded**

**Ready for Implementation Phase 1: Frontend Wizard with Mock Backend**