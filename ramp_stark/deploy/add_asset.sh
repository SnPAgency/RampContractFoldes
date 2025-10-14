#!/bin/bash

sncast --account=phydy invoke \
    --contract-address=0x02461084678393887c078d3bfcbb94a9afed63284a69145e511786e0a652f0f7 \
    --function "add_allowed_asset" \
    --arguments '0x049a388016755daf4258ba1f03092f3bd035abdf00904252c24061f93ddf680c, 0x60d71049e736f80db3b7ce9496152ef54564872cee3ae67b08bd2bdb7efee8d, 1' \
    --network sepolia
