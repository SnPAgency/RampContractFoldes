# RampContractFoldes - Multi-Blockchain Contract Suite For SnappiPay Project

This repository contains smart contracts for SnappiPay Project. The Project impliments multiple blockchain platforms, each with their own build systems and testing frameworks.

## Project Structure

```
RampContractFoldes/
├── RampAptos/          # Aptos Move contracts
├── RampSol/            # Ethereum Solidity contracts (Foundry)
├── RampSup/            # Supra Move contracts
├── ramp_solana/        # Solana Rust contracts
├── ramp_stark/         # StarkNet Cairo contracts
├── build_all.sh        # Complete build workflow script
├── Makefile           # Make targets for easy building
└── .github/workflows/ # CI/CD pipeline
```

## Quick Start

### Prerequisites

Before running the build workflow, you need to install the required tools for each blockchain platform:

1. **Rust & Cargo** (for Solana): https://rustup.rs/
2. **Foundry** (for Ethereum): https://getfoundry.sh/
3. **Aptos CLI** (for Aptos): https://aptos.dev/tools/aptos-cli/install-cli/
4. **Scarb** (for StarkNet): https://docs.swmansion.com/scarb/download.html
5. **Supra CLI** (optional, Aptos CLI can be used as fallback)

### Installation Script

Run the dependency installation script:

```bash
chmod +x install_deps.sh
./install_deps.sh
```

### Build All Projects

Use any of these methods to build and test all projects:

#### Method 1: Build Script
```bash
./build_all.sh
```

#### Method 2: Makefile
```bash
make all          # Build and test everything
make build        # Build only
make test         # Test only
make clean        # Clean build artifacts
```

#### Method 3: Individual Projects
```bash
make aptos        # Build RampAptos
make sol          # Build RampSol
make supra        # Build RampSup
make solana       # Build ramp_solana
make stark        # Build ramp_stark
```

## Project Details

### RampAptos (Move/Aptos)
- **Language**: Move
- **Platform**: Aptos blockchain
- **Build**: `aptos move compile`
- **Test**: `aptos move test`

### RampSol (Solidity/Foundry)
- **Language**: Solidity
- **Platform**: Ethereum
- **Framework**: Foundry
- **Build**: `forge build`
- **Test**: `forge test`

### RampSup (Move/Supra)
- **Language**: Move
- **Platform**: Supra blockchain
- **Build**: `supra move tool compile`
- **Test**: `supra move tool test`

### ramp_solana (Rust/Anchor)
- **Language**: Rust
- **Platform**: Solana
- **Framework**: Anchor (if present)
- **Build**: `cargo build` + `anchor build` (if Anchor.toml exists)
- **Test**: `cargo test`

### ramp_stark (Cairo/Scarb/snforge)
- **Language**: Cairo
- **Platform**: StarkNet
- **Framework**: Scarb/snforge
- **Build**: `scarb build`
- **Test**: `snforge test`

## CI/CD Pipeline

The repository includes a GitHub Actions workflow (`.github/workflows/ci.yml`) that:

1. Runs builds for each project individually when their files change
2. Executes the complete workflow on every push/PR
3. Caches dependencies for faster builds
4. Provides detailed build status for each blockchain platform

## Workflow Features

- ✅ **Multi-platform support**: Handles 5 different blockchain platforms
- ✅ **Dependency checking**: Verifies required tools are installed
- ✅ **Error handling**: Continues building other projects if one fails
- ✅ **Colored output**: Easy-to-read status messages
- ✅ **Build summary**: Shows success/failure status for each project
- ✅ **Flexible execution**: Can run individual projects or all at once

## Troubleshooting

### Common Issues

1. **Missing CLI tools**: Install the required tools for your target platform
2. **Permission denied**: Make sure scripts are executable (`chmod +x build_all.sh`)
3. **Build failures**: Check individual project logs for specific error messages

### Getting Help

```bash
make help          # Show available Make targets
./build_all.sh -h  # Show build script options (if implemented)
```

## Contributing

1. Make changes to the relevant project directory
2. Test locally using `./build_all.sh` or `make all`
3. Commit and push - CI will automatically test all affected projects
4. Create a pull request
