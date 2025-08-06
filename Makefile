# SnappiPay - Makefile for Smart Contracts Project

.PHONY: all build test clean install-deps help aptos sol supra solana stellar stark

# Default target
all: build test

# Build all projects
build:
	@echo "Building all projects..."
	@./build_all.sh

# Test all projects (same as build since build includes tests)
test: build

# Clean all build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@if [ -d "RampSol" ]; then cd RampSol && forge clean; fi
	@if [ -d "ramp_solana" ]; then cd ramp_solana && cargo clean; fi
	@if [ -d "ramp_stellar" ]; then cd ramp_stellar && cargo clean; fi
	@if [ -d "ramp_stark" ]; then cd ramp_stark && scarb clean; fi
	@echo "Clean completed"

# Install dependencies for all projects
install-deps:
	@echo "Installing dependencies..."
	@if [ -d "RampSol" ]; then cd RampSol && forge install --no-commit; fi
	@echo "Dependencies installed"

# Individual project targets
aptos:
	@echo "Building RampAptos..."
	@if [ -d "RampAptos" ]; then cd RampAptos && aptos move compile && aptos move test; else echo "RampAptos not found"; fi

sol:
	@echo "Building RampSol..."
	@if [ -d "RampSol" ]; then cd RampSol && forge build && forge test; else echo "RampSol not found"; fi

supra:
	@echo "Building RampSup..."
	@if [ -d "RampSup" ]; then cd RampSup && supra move tool compile && supra move tool test; else echo "RampSup not found"; fi

solana:
	@echo "Building ramp_solana..."
	@if [ -d "ramp_solana" ]; then cd ramp_solana && cargo build && cargo test; else echo "ramp_solana not found"; fi

stellar:
	@echo "Building ramp_stellar..."
	@if [ -d "ramp_stellar" ]; then cd ramp_stellar && stellar contract build; else echo "ramp_stellar not found"; fi

stark:
	@echo "Building ramp_stark..."
	@if [ -d "ramp_stark" ]; then cd ramp_stark && scarb build && snforge test; else echo "ramp_stark not found"; fi

# Help target
help:
	@echo "Available targets:"
	@echo "  all        - Build and test all projects (default)"
	@echo "  build      - Build all projects"
	@echo "  test       - Test all projects"
	@echo "  clean      - Clean build artifacts"
	@echo "  install-deps - Install dependencies"
	@echo ""
	@echo "Individual project targets:"
	@echo "  aptos      - Build and test RampAptos (Move/Aptos)"
	@echo "  sol        - Build and test RampSol (Solidity/Foundry)"
	@echo "  supra      - Build and test RampSup (Move/Supra)"
	@echo "  solana     - Build and test ramp_solana (Rust/Anchor)"
	@echo "  stark      - Build and test ramp_stark (Cairo/Scarb)"
	@echo "  help       - Show this help message"
