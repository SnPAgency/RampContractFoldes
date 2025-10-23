#!/bin/bash

cast send $PROXY_HEDERA "offRampNative(uint8,uint8,bytes)" \
    0 0 "0x" \
    --rpc-url $HEDERA_RPC_URL \
    --private-key $HEDERA_PRIVATE_KEY \
    --value 20ether