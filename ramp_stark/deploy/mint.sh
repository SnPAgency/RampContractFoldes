#!/bin/bash

sncast --account=phydy invoke \
    --contract-address=0x049a388016755daf4258ba1f03092f3bd035abdf00904252c24061f93ddf680c \
    --function "mint" \
    --arguments '0x358a3e80dc32c17b196d7d86ce80471704c5b0fd88a704b60c2fec3a325e7bb, 10000000000000000000000000' \
    --network sepolia
