#! /bin/bash

OBJECT_ADDRESS=0xe56f319912e4aa2543314ee049cacc6920b79a8a0582993ba194e732764f0147
aptos move compile-script --named-addresses RampAptos=$OBJECT_ADDRESS \
&& aptos move run-script --compiled-script-path /home/phydy/Work/Personal/RampContractFoldes/RampAptos/deploy/script.mv --profile phydy #--args address:0xf79df721778f84ccfa7212e8641a4dff8b3a45c9253a94bd201978b6f614660c