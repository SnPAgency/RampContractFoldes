#!/bin/bash

sncast --account=test \
    invoke \
    --contract-address=0x02461084678393887c078d3bfcbb94a9afed63284a69145e511786e0a652f0f7 \
    --function "upgrade_to" \
    --arguments $CLASS_HASH \
    --network sepolia