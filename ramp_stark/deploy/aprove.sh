#!/bin/bash

sncast --account=phydy \
    invoke \
    --contract-address=0x049a388016755daf4258ba1f03092f3bd035abdf00904252c24061f93ddf680c \
    --function "approve" \
    --arguments '0x02461084678393887c078d3bfcbb94a9afed63284a69145e511786e0a652f0f7, 10000' \
    --network sepolia