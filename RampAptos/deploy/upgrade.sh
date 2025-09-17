#! /bin/bash
aptos move compile --named-addresses RampAptos=$OBJECT_ADDRESS \
&& aptos move upgrade-object --address-name RampAptos --object-address $OBJECT_ADDRESS --profile phydy