#! /bin/bash
aptos move compile --named-addresses RampAptos=0xe56f319912e4aa2543314ee049cacc6920b79a8a0582993ba194e732764f0147 \
&& aptos move upgrade-object --address-name RampAptos --object-address 0xe56f319912e4aa2543314ee049cacc6920b79a8a0582993ba194e732764f0147 --profile phydy