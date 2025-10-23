#!/bin/bash

cast send $USDC_SEPOLIA "mint(address,uint256)" $SENDER 1000000000000000 \
    --rpc-url $SEPOLIA_RPC_URL \
    --private-key $PRIVATE_KEY
    && \
    cast send $USDT_SEPOLIA "mint(address,uint256)" $SENDER 1000000000000000 \
        --rpc-url $SEPOLIA_RPC_URL \
        --private-key $PRIVATE_KEY