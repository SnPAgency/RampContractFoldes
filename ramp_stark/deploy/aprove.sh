#!/bin/bash

sncast --account=phydy \
    invoke \
    --contract-address=0x079c69dce3e049f6eb28f9aa71b0c735ff7b7a1ca5b263bf6f98f33799ace0e2 \
    --function "approve" \
    --arguments '0x02e595ef87a1edc526f04c25e39f65a1adcb5012e599f3033b74161745a2b86c, 10000' \
    --network sepolia