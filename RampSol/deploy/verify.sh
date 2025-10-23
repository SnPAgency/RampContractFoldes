#!/bin/bash

forge verify-contract $IMPLIMENTATION_HEDERA \
    --rpc-url $HEDERA_RPC_URL \
    --chain-id 296 \
    --verifier sourcify \
    --verifier-url "https://server-verify.hashscan.io/" \
    --constructor-args $(cast abi-encode "constructor(address,address)" 0x2e9f2728a1dea6c8a6354af451276197f0bdde11 0x2e9f2728a1dea6c8a6354af451276197f0bdde11)