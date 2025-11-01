#!/bin/bash

RAMP_SELECTOR="offRampDeposit(address,uint256,address,uint8,uint8,bytes)"
TOKEN_SELECTOR="approve(address,uint256)"

cast send $USDT_HEDERA $TOKEN_SELECTOR $PROXY_HEDERA 1000000000000000 \
    --rpc-url $HEDERA_RPC_URL \
    --private-key $HEDERA_PRIVATE_KEY \
    && \ 
    cast send $PROXY_HEDERA \
        $RAMP_SELECTOR \
        $USDT_HEDERA 1000000000000000 $HEDERA_SENDER 0 0 "0x" \
        --rpc-url $HEDERA_RPC_URL \
        --private-key $HEDERA_PRIVATE_KEY