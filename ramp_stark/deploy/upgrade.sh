#!/bin/bash

sncast --account=test \
    invoke \
    --contract-address=0x0500c37a47b5f2c1825ea48e8501bfe2fe570bfb5c7a037283f99124e4a949d4 \
    --function "on_ramp_deposit" \
    --arguments $CLASS_HASH \
    --network sepolia