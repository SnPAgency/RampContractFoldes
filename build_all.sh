#!/bin/bash

# SnappiPay Contracts - Complete Build and Test Workflow
# This script handles compilation, building, and testing for all blockchain projects

# Don't exit on error - we want to continue building other projects
# set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo -e "\n${BLUE}================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}================================${NC}\n"
}

# Get the script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Track success/failure
declare -a RESULTS=()

# Function to record result
record_result() {
    local project="$1"
    local status="$2"
    RESULTS+=("$project: $status")
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to build RampAptos (Move/Aptos)
build_ramp_aptos() {
    print_header "Building RampAptos (Move/Aptos)"
    
    if [ ! -d "RampAptos" ]; then
        print_warning "RampAptos directory not found, skipping..."
        record_result "RampAptos" "SKIPPED"
        return
    fi
    
    cd RampAptos
    
    if ! command_exists aptos; then
        print_error "Aptos CLI not found. Please install it first."
        record_result "RampAptos" "FAILED - Missing aptos CLI"
        cd ..
        return
    fi
    
    print_status "Compiling Move modules..."
    if aptos move compile --dev; then
        print_success "RampAptos compilation successful"
        
        print_status "Running Move tests..."
        if aptos move test --dev; then
            print_success "RampAptos tests passed"
            record_result "RampAptos" "SUCCESS"
        else
            print_error "RampAptos tests failed"
            record_result "RampAptos" "FAILED - Tests"
        fi
    else
        print_error "RampAptos compilation failed"
        record_result "RampAptos" "FAILED - Compilation"
    fi
    
    cd ..
}

# Function to build RampSol (Solidity/Foundry)
build_ramp_sol() {
    print_header "Building RampSol (Solidity/Foundry)"
    
    if [ ! -d "RampSol" ]; then
        print_warning "RampSol directory not found, skipping..."
        record_result "RampSol" "SKIPPED"
        return
    fi
    
    cd RampSol
    
    if ! command_exists forge; then
        print_error "Foundry (forge) not found. Please install it first."
        record_result "RampSol" "FAILED - Missing forge"
        cd ..
        return
    fi
    
    print_status "Installing dependencies..."
    if [ -f "foundry.toml" ]; then
        forge install
    fi
    
    print_status "Building Solidity contracts..."
    if forge build; then
        print_success "RampSol build successful"
        
        print_status "Running Foundry tests..."
        if forge test; then
            print_success "RampSol tests passed"
            record_result "RampSol" "SUCCESS"
        else
            print_error "RampSol tests failed"
            record_result "RampSol" "FAILED - Tests"
        fi
    else
        print_error "RampSol build failed"
        record_result "RampSol" "FAILED - Build"
    fi
    
    cd ..
}

# Function to build RampSup (Move/Supra)
build_ramp_sup() {
    print_header "Building RampSup (Move/Supra)"
    
    if [ ! -d "RampSup" ]; then
        print_warning "RampSup directory not found, skipping..."
        record_result "RampSup" "SKIPPED"
        return
    fi
    
    cd RampSup
    
    # Check if Docker is available
    if ! command_exists docker; then
        print_error "Docker not found. Please install Docker to use Supra CLI."
        record_result "RampSup" "FAILED - Missing Docker"
        cd ..
        return
    fi
#    
#    # Check if we can use local supra command first
#    if command_exists supra; then
#        print_status "Using local Supra CLI..."
#        CLI_CMD="supra move tool"
#    elif docker ps | grep -q supra_cli; then
#        print_status "Using Supra CLI Docker container..."
#        # Test if supra command works directly in container
#        print_status "Testing supra command availability..."
#        if docker exec supra_cli supra --help > /dev/null 2>&1; then
#            print_status "✅ supra command works directly"
#            CLI_CMD="docker exec supra_cli supra move tool"
#        elif docker exec supra_cli /bin/bash -c "supra --help" > /dev/null 2>&1; then
#            print_status "✅ supra command works with bash shell"
#            CLI_CMD="docker exec supra_cli /bin/bash -c 'supra move tool'"
#        else
#            print_error "❌ supra command not accessible in container"
#            docker exec supra_cli ls -la /usr/local/bin/ || true
#            docker exec supra_cli find / -name "supra" 2>/dev/null | head -10 || true
#            record_result "RampSup" "FAILED - Supra command not accessible in container"
#            cd ..
#            return
#        fi
#    else
#        print_status "Starting Supra CLI Docker container..."
#        curl https://raw.githubusercontent.com/supra-labs/supra-dev-hub/refs/heads/main/Scripts/cli/compose.yaml | docker compose -f - up -d
#        sleep 10
#        
#        if docker ps | grep -q supra_cli; then
#            # Test supra command after starting container
#            print_status "Testing supra command in new container..."
#            if docker exec supra_cli supra --help > /dev/null 2>&1; then
#                print_status "✅ supra command works directly"
#                CLI_CMD="docker exec supra_cli supra move tool"
#            elif docker exec supra_cli /bin/bash -c "supra --help" > /dev/null 2>&1; then
#                print_status "✅ supra command works with bash shell"
#                CLI_CMD="docker exec supra_cli /bin/bash -c 'supra move tool'"
#            else
#                print_error "❌ Failed to access supra command in container"
#                docker exec supra_cli ls -la /usr/local/bin/ || true
#                docker exec supra_cli find / -name "supra" 2>/dev/null | head -10 || true
#                record_result "RampSup" "FAILED - No Supra CLI available"
#                cd ..
#                return
#            fi
#        else
#            print_error "Failed to start Supra CLI container and no local supra command found"
#            record_result "RampSup" "FAILED - No Supra CLI available"
#            cd ..
#            return
#        fi
#    fi
#    
#    print_status "Compiling Move modules with Supra CLI..."
#    if $CLI_CMD compile; then
#        print_success "RampSup compilation successful"
#        
#        print_status "Running Move tests..."
#        if $CLI_CMD test; then
#            print_success "RampSup tests passed"
#            record_result "RampSup" "SUCCESS"
#        else
#            print_error "RampSup tests failed"
#            record_result "RampSup" "FAILED - Tests"
#        fi
#    else
#        print_error "RampSup compilation failed"
#        record_result "RampSup" "FAILED - Compilation"
#    fi
    
    cd ..
}

# Function to build ramp_solana (Rust/Anchor)
build_ramp_solana() {
    print_header "Building ramp_solana (Rust/Anchor)"
    
    if [ ! -d "ramp_solana" ]; then
        print_warning "ramp_solana directory not found, skipping..."
        record_result "ramp_solana" "SKIPPED"
        return
    fi
    
    cd ramp_solana
    
    if ! command_exists cargo; then
        print_error "Cargo (Rust) not found. Please install Rust first."
        record_result "ramp_solana" "FAILED - Missing cargo"
        cd ..
        return
    fi
    
    print_status "Building Rust project..."
    if cargo build; then
        print_success "ramp_solana build successful"
        
        print_status "Running Rust tests..."
        if cargo test; then
            print_success "ramp_solana tests passed"
            
            # Check if this is an Anchor project
            if command_exists anchor && [ -f "Anchor.toml" ]; then
                print_status "Building with Anchor..."
                if anchor build; then
                    print_success "Anchor build successful"
                    record_result "ramp_solana" "SUCCESS"
                else
                    print_error "Anchor build failed"
                    record_result "ramp_solana" "FAILED - Anchor build"
                fi
            else
                record_result "ramp_solana" "SUCCESS"
            fi
        else
            print_error "ramp_solana tests failed"
            record_result "ramp_solana" "FAILED - Tests"
        fi
    else
        print_error "ramp_solana build failed"
        record_result "ramp_solana" "FAILED - Build"
    fi
    
    cd ..
}

# Function to build ramp_stellar (Rust/Stellar)
build_ramp_stellar() {
    print_header "Building ramp_stellar (Stellar Soroban)"
    
    if [ ! -d "ramp_stellar" ]; then
        print_warning "ramp_stellar directory not found, skipping..."
        record_result "ramp_stellar" "SKIPPED"
        return
    fi
    
    cd ramp_stellar
    
    # Check if stellar CLI is installed
    if ! command_exists stellar; then
        print_error "Stellar CLI not found. Please install it first:"
        print_error "cargo install --locked stellar-cli@23.0.0"
        record_result "ramp_stellar" "FAILED"
        cd ..
        return
    fi
    
    print_status "Building Stellar contracts..."
    if stellar contract build; then
        print_success "ramp_stellar build completed successfully"
        record_result "ramp_stellar" "SUCCESS"
    else
        print_error "ramp_stellar build failed"
        record_result "ramp_stellar" "FAILED"
    fi
    
    cd ..
}

# Function to build ramp_stark (Cairo/Scarb)
build_ramp_stark() {
    print_header "Building ramp_stark (Cairo/Scarb)"
    
    if [ ! -d "ramp_stark" ]; then
        print_warning "ramp_stark directory not found, skipping..."
        record_result "ramp_stark" "SKIPPED"
        return
    fi
    
    cd ramp_stark
    
    if ! command_exists scarb; then
        print_error "Scarb not found. Please install it first."
        record_result "ramp_stark" "FAILED - Missing scarb"
        cd ..
        return
    fi
    
    print_status "Building Cairo project with Scarb..."
    if scarb build; then
        print_success "ramp_stark build successful"
        
        print_status "Running Cairo tests..."
        
        # Check if snforge is available
        if command -v snforge >/dev/null 2>&1; then
            print_status "snforge found: $(which snforge)"
            if snforge test; then
                print_success "ramp_stark tests passed"
                record_result "ramp_stark" "SUCCESS"
            else
                print_error "ramp_stark tests failed"
                record_result "ramp_stark" "FAILED - Tests"
            fi
        else
            print_error "snforge command not found in PATH"
            print_status "Available commands in PATH:"
            echo "PATH: $PATH"
            ls -la ~/.local/bin/ 2>/dev/null | grep snforge || true
            find ~ -name "snforge" -type f 2>/dev/null | head -5 || true
            print_error "Cannot run Cairo tests without snforge"
            record_result "ramp_stark" "FAILED - snforge not found"
        fi
    else
        print_error "ramp_stark build failed"
        record_result "ramp_stark" "FAILED - Build"
    fi
    
    cd ..
}

# Main execution
main() {
    print_header "RampContractFoldes - Complete Build Workflow"
    print_status "Starting build process for all projects..."
    
    # Build all projects
    build_ramp_aptos
    build_ramp_sol
    build_ramp_sup
    build_ramp_solana
    build_ramp_stellar
    build_ramp_stark
    
    # Print summary
    print_header "Build Summary"
    for result in "${RESULTS[@]}"; do
        if [[ $result == *"SUCCESS"* ]]; then
            print_success "$result"
        elif [[ $result == *"SKIPPED"* ]]; then
            print_warning "$result"
        else
            print_error "$result"
        fi
    done
    
    # Count successes and failures
    success_count=$(printf '%s\n' "${RESULTS[@]}" | grep -c "SUCCESS" || true)
    failure_count=$(printf '%s\n' "${RESULTS[@]}" | grep -c "FAILED" || true)
    skip_count=$(printf '%s\n' "${RESULTS[@]}" | grep -c "SKIPPED" || true)
    
    echo -e "\n${BLUE}Final Results:${NC}"
    echo -e "✅ Successful: $success_count"
    echo -e "❌ Failed: $failure_count"
    echo -e "⏭️  Skipped: $skip_count"
    
    if [ $failure_count -gt 0 ]; then
        exit 1
    fi
}

# Run main function
main "$@"
