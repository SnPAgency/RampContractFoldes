#! /bin/bash
aptos move compile --named-addresses RampAptos=0x87994ad2d5e6947f6d472e015a67adc779d23f306d1d50b199647f829b215390 \
&& aptos move deploy-object --address-name RampAptos --profile sub