#! /bin/bash
CLASS_HASH=0x4b2a8be53e3b1c1ae528f27cbdf5ea9a392b260e7de450489aa0e8046068dea
sncast --account=phydy deploy \
    --class-hash=${CLASS_HASH} \
    --constructor-calldata 0x6c2041b0781e004a408db863df8cadcfa468e619c83ad8b8bcf30dfda844936 0x60d71049e736f80db3b7ce9496152ef54564872cee3ae67b08bd2bdb7efee8d \
    --network=sepolia
#0x02e595ef87a1edc526f04c25e39f65a1adcb5012e599f3033b74161745a2b86c