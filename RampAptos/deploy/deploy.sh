#! /bin/bash
aptos move compile --named-addresses RampAptos=$ACCOUNT_ADDRESS \
&& aptos move deploy-object --address-name RampAptos --profile phydy