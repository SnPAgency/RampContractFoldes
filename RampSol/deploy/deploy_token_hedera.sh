#!/bin/bash

forge script script/04_DeployHederaToken.sol:RampTokenScript --sender ${HEDERA_SENDER} --rpc-url ${HEDERA_RPC_URL} --broadcast -vvvv
