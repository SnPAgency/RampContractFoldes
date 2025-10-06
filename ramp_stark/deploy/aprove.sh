#!/bin/bash

sncast --account=test \
    invoke \
    --contract-address=0x03e68531eea5deec8a67b47f0330533260de3632ab21d6b0fad5a77a3c14dc58 \
    --function "approve" \
    --arguments '0x0076a2a9249d67fd7ef5e11297eab406eec8b881f29f7a06a37ea91d2c4411f1, 100' \
    --network sepolia