#!/bin/bash

cast send $PROXY_HEDERA "onRampNative(address,uint256)" \
    $HEDERA_SENDER 2 \
    --rpc-url $HEDERA_RPC_URL \
    --private-key $HEDERA_PRIVATE_KEY