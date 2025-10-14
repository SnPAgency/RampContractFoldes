#!/bin/bash

sncast --account=phydy \
    invoke \
    --contract-address=0x049a388016755daf4258ba1f03092f3bd035abdf00904252c24061f93ddf680c \
    --function "approve" \
    --arguments '0x02461084678393887c078d3bfcbb94a9afed63284a69145e511786e0a652f0f7, 10000' \
    --network sepolia

MEDIUM=ramp_stark::interfaces::ramp_interface::OnrampMedium::Primary
REGION=ramp_stark::interfaces::ramp_interface::Region::KEN
sncast --account=phydy \
    invoke \
    --contract-address=0x02461084678393887c078d3bfcbb94a9afed63284a69145e511786e0a652f0f7 \
    --function "on_ramp_deposit" \
    --arguments '0x049a388016755daf4258ba1f03092f3bd035abdf00904252c24061f93ddf680c, 10000, 0x6c2041b0781e004a408db863df8cadcfa468e619c83ad8b8bcf30dfda844936, ramp_stark::interfaces::ramp_interface::OnrampMedium::Primary, ramp_stark::interfaces::ramp_interface::Region::KEN, ""' \
    --network sepolia