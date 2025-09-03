#! /bin/bash

sncast --account=phydy deploy \
    --class-hash=${CLASS_HASH} \
    --constructor-calldata 0x026045434ecad0435cf47b855c5f807a73e2aa4194fa4a57a73904033bc9e305 0x026045434ecad0435cf47b855c5f807a73e2aa4194fa4a57a73904033bc9e305 \
    --network=sepolia
#0x0500c37a47b5f2c1825ea48e8501bfe2fe570bfb5c7a037283f99124e4a949d4