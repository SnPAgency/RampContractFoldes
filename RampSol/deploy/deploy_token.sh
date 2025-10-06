#!/bin/bash

forge script script/02_DeployToken.s.sol:RampTokenScript --sender ${SENDER} --rpc-url sepolia --broadcast -vvvv
