#! /bin/bash

sncast --account=phydy deploy \
    --class-hash=${TOKEN_CLASS_HASH} \
    --constructor-calldata 0x026045434ecad0435cf47b855c5f807a73e2aa4194fa4a57a73904033bc9e305 0x026045434ecad0435cf47b855c5f807a73e2aa4194fa4a57a73904033bc9e305 \
    --network=sepolia
#0x03e68531eea5deec8a67b47f0330533260de3632ab21d6b0fad5a77a3c14dc58