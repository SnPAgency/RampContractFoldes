#!/bin/bash

cast send $PROXY_HEDERA "onRampNative(address,uint256)" \
    0x72615d5cc07a38df489d04a1dd0818c3b9244ff6 1 ether \
    --rpc-url $HEDERA_RPC_URL \
    --private-key $HEDERA_PRIVATE_KEY