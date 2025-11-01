#!/bin/bash

sncast --account=phydy invoke \
    --contract-address=0x02e595ef87a1edc526f04c25e39f65a1adcb5012e599f3033b74161745a2b86c \
    --function "add_allowed_asset" \
    --arguments '0x079c69dce3e049f6eb28f9aa71b0c735ff7b7a1ca5b263bf6f98f33799ace0e2, 0x60d71049e736f80db3b7ce9496152ef54564872cee3ae67b08bd2bdb7efee8d, 1' \
    --network sepolia
            