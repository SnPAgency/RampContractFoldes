#!/bin/bash

RAMP_SELECTOR="offRampDeposit(address,uint256,address,uint8,uint8,bytes)"
TOKEN_SELECTOR="approve(address,uint256)"

cast send $DAI_SEPOLIA $TOKEN_SELECTOR $PROXY_SEPOLIA 1000000000000000 \
    --rpc-url $SEPOLIA_RPC_URL \
    --private-key $PRIVATE_KEY \
    && \
    cast send $PROXY_SEPOLIA \
        $RAMP_SELECTOR \
        $DAI_SEPOLIA 1000000000000000 $SENDER 0 0 "0x" \
        --rpc-url $SEPOLIA_RPC_URL \
        --private-key $PRIVATE_KEY
