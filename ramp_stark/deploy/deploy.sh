#! /bin/bash
CLASS_HASH=0x76f09cea3934ca4c69bb4f8d13569035168e11e2fb6785034e6f835360e90d4
sncast --account=phydy deploy \
    --class-hash=${CLASS_HASH} \
    --constructor-calldata 0x6c2041b0781e004a408db863df8cadcfa468e619c83ad8b8bcf30dfda844936 0x60d71049e736f80db3b7ce9496152ef54564872cee3ae67b08bd2bdb7efee8d \
    --network=sepolia
#0x02461084678393887c078d3bfcbb94a9afed63284a69145e511786e0a652f0f7