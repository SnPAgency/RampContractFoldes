#!/bin/bash

sncast --account=phydy \
    invoke \
    --contract-address=0x079c69dce3e049f6eb28f9aa71b0c735ff7b7a1ca5b263bf6f98f33799ace0e2 \
    --function "approve" \
    --arguments '0x02e595ef87a1edc526f04c25e39f65a1adcb5012e599f3033b74161745a2b86c, 10000' \
    --network sepolia


MEDIUM=ramp_stark::interfaces::ramp_interface::OnrampMedium::Secondary
REGION=ramp_stark::interfaces::ramp_interface::Region::NGA
sncast --account=phydy \
    invoke \
    --contract-address=0x02e595ef87a1edc526f04c25e39f65a1adcb5012e599f3033b74161745a2b86c \
    --function "off_ramp_deposit" \
    --arguments '0x079c69dce3e049f6eb28f9aa71b0c735ff7b7a1ca5b263bf6f98f33799ace0e2, 10000, 0x6c2041b0781e004a408db863df8cadcfa468e619c83ad8b8bcf30dfda844936, ramp_stark::interfaces::ramp_interface::OnrampMedium::Secondary, ramp_stark::interfaces::ramp_interface::Region::NGA, "some"' \
    --network sepolia