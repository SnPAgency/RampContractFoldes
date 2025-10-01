#!/bin/bash

MEDIUM=ramp_stark::interfaces::ramp_interface::OnrampMedium::Primary
REGION=ramp_stark::interfaces::ramp_interface::Region::KEN
sncast --account=test \
    invoke \
    --contract-address=0x0500c37a47b5f2c1825ea48e8501bfe2fe570bfb5c7a037283f99124e4a949d4 \
    --function "on_ramp_deposit" \
    --arguments '0x03e68531eea5deec8a67b47f0330533260de3632ab21d6b0fad5a77a3c14dc58, 100, 0x60d71049e736f80db3b7ce9496152ef54564872cee3ae67b08bd2bdb7efee8d, ramp_stark::interfaces::ramp_interface::OnrampMedium::Primary, ramp_stark::interfaces::ramp_interface::Region::KEN, ""' \
    --network sepolia