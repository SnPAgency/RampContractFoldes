#! /bin/bash
aptos move compile --named-addresses RampAptos=0xe46b2c641295892276b13fc6cf6d6237bd8cb2fa16c66263840038dac83b9572 \
&& aptos move upgrade-object --address-name RampAptos --object-address 0xe46b2c641295892276b13fc6cf6d6237bd8cb2fa16c66263840038dac83b9572 --profile phydy