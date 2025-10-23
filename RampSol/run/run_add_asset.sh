#!/bin/bash

cast send $PROXY_HEDERA \
    "function addAllowedAsset(address,address,uint256) external" \
    $USDT_HEDERA $HEDERA_SENDER 1 \
    --rpc-url $HEDERA_RPC_URL \
    --private-key $HEDERA_PRIVATE_KEY \
    && \
    cast send $PROXY_HEDERA \
        "function addAllowedAsset(address,address,uint256) external" \
        $USDC_HEDERA $HEDERA_SENDER 1 \
        --rpc-url $HEDERA_RPC_URL \
        --private-key $HEDERA_PRIVATE_KEY