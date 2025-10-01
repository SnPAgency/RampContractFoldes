#! /bin/bash

OBJECT_ADDRESS=0xe46b2c641295892276b13fc6cf6d6237bd8cb2fa16c66263840038dac83b9572
aptos move compile-script --named-addresses RampAptos=$OBJECT_ADDRESS \
&& aptos move run-script --compiled-script-path /home/phydy/Work/Personal/RampContractFoldes/RampAptos/deploy/script.mv --profile phydy