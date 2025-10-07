#!/bin/bash

#forge script ../script/05_mint_tokens.sol:MintTokenScript --rpc-url sepolia --broadcast -vvvv

cast send $USDC_HEDERA "mint(address,uint256)" $HEDERA_SENDER 1000000000000000 \
    --rpc-url $HEDERA_RPC_URL \
    --private-key $HEDERA_PRIVATE_KEY \
    && \
    cast send $USDC_SEPOLIA "mint(address,uint256)" $SENDER 1000000000000000 \
        --rpc-url $SEPOLIA_RPC_URL \
        --private-key $PRIVATE_KEY
