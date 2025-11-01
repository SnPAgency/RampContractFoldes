#!/bin/bash

#forge script ../script/05_mint_tokens.sol:MintTokenScript --rpc-url sepolia --broadcast -vvvv

cast send $USDC_HEDERA "mint(address,uint256)" 0x30027CeF75D04737831FE93f2519aecD945C77a4 1000000000000000 \
    --rpc-url $HEDERA_RPC_URL \
    --private-key $HEDERA_PRIVATE_KEY \
    && \
    cast send $USDT_HEDERA "mint(address,uint256)" 0x30027CeF75D04737831FE93f2519aecD945C77a4 1000000000000000 \
        --rpc-url $HEDERA_RPC_URL \
        --private-key $HEDERA_PRIVATE_KEY 
