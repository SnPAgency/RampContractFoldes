#!/bin/bash

sncast --account=phydy invoke \
    --contract-address=0x049a388016755daf4258ba1f03092f3bd035abdf00904252c24061f93ddf680c \
    --function "mint" \
    --arguments '0x6c2041b0781e004a408db863df8cadcfa468e619c83ad8b8bcf30dfda844936, 10000000000000000000000000' \
    --network sepolia
