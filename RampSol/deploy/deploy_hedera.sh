#!/bin/bash

forge script script/03_DeployRampHedera.sol:RampHederaContractScript --sender ${HEDERA_SENDER} --rpc-url ${HEDERA_RPC_URL} --broadcast -vvvv --libraries src/helpers/errors.sol:Errors:0x5aB9E25D2eCBE4a77f6D0BBFB8495150DdB1d545
