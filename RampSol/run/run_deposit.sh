#!/bin/bash

RAMP_SELECTOR="onRampDeposit(address,uint256,address,uint8,uint8,bytes)"
TOKEN_SELECTOR="approve(address,uint256)"

cast send $DAI_SEPOLIA $TOKEN_SELECTOR $PROXY_SEPOLIA 1000000000000000 \
    --rpc-url $SEPOLIA_RPC_URL \
    --private-key $PRIVATE_KEY \
    && \
    cast send $PROXY_SEPOLIA \
        $RAMP_SELECTOR \
        $DAI_SEPOLIA 1000000000000000 $SENDER 0 0 "0x" \
        --rpc-url $SEPOLIA_RPC_URL \
        --private-key $PRIVATE_KEY \
        &&\
        cast send $USDT_HEDERA $TOKEN_SELECTOR $PROXY_HEDERA 1000000000000000 \
            --rpc-url $HEDERA_RPC_URL \
            --private-key $HEDERA_PRIVATE_KEY \
            && \ 
            cast send $PROXY_HEDERA \
                $RAMP_SELECTOR \
                $USDT_HEDERA 1000000000000000 $HEDERA_SENDER 0 0 "0x" \
                --rpc-url $HEDERA_RPC_URL \
                --private-key $HEDERA_PRIVATE_KEY