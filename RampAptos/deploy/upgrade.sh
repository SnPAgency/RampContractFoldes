#! /bin/bash
aptos move compile --named-addresses RampAptos=0x11f6f88f2f10a3b817b0688295e6d41daa7ecff2db73a6233ffbe6d016bf2509 \
&& aptos move upgrade-object --address-name RampAptos --object-address 0x11f6f88f2f10a3b817b0688295e6d41daa7ecff2db73a6233ffbe6d016bf2509 --profile sub