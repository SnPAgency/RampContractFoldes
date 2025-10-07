#!/bin/bash

cast send $PROXY_SEPOLIA \
    "function addAllowedAsset(address,address,uint256) external" \
    $USDC_SEPOLIA $SENDER 1 \
    --rpc-url $SEPOLIA_RPC_URL \
    --private-key $PRIVATE_KEY 
    #&& \
cast send $PROXY_HEDERA \
    "function addAllowedAsset(address,address,uint256) external" \
    $USDC_HEDERA $HEDERA_SENDER 1 \
    --rpc-url $HEDERA_RPC_URL \
    --private-key $HEDERA_PRIVATE_KEY   