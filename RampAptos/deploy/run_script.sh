#! /bin/bash

aptos move compile-script --named-addresses RampAptos=$OBJECT_ADDRESS \
&& aptos move run-script --compiled-script-path $COMPILED_SCRIPT_PATH --profile phydy