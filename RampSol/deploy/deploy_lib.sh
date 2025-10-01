#!/bin/bash

forge create src/helpers/errors.sol:Errors --rpc-url ${HEDERA_RPC_URL} --private-key $HEDERA_PRIVATE_KEY --broadcast
