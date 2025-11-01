#!/bin/bash

forge script ../script/03_DeployRampHedera.sol:RampHederaContractScript \
    --sender ${HEDERA_SENDER} \
    --rpc-url hedera \
    --broadcast \
    -vvvv \
    --libraries ../src/helpers/errors.sol:Errors:0x1328185681695B08f7a12a0bD0D3D2fBfcBe4FD4 \
    --verify

