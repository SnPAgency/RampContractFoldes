#!/bin/bash

sncast --account=test \
    invoke \
    --contract-address=0x03e68531eea5deec8a67b47f0330533260de3632ab21d6b0fad5a77a3c14dc58 \
    --function "approve" \
    --arguments '0x0500c37a47b5f2c1825ea48e8501bfe2fe570bfb5c7a037283f99124e4a949d4, 100' \
    --network sepolia