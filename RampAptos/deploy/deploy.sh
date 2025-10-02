#! /bin/bash
aptos move compile --named-addresses RampAptos=0x6f276be61cdbd8e5c1a99ed191179b41068b980a6c644bc0feb1061c636250d5 \
&& aptos move deploy-object --address-name RampAptos --profile phydy