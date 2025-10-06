#! /bin/bash

OBJECT_ADDRESS=0x11f6f88f2f10a3b817b0688295e6d41daa7ecff2db73a6233ffbe6d016bf2509
TOKEN_ADDRESS=0x0f4a054b2e2878d5fe972cf789cbb0bc6946c1f970c6a2cfab6166aee5dd9d44

aptos move compile-script --named-addresses RampAptos=$OBJECT_ADDRESS \
&& aptos move run-script --compiled-script-path /home/phydy/Work/Personal/RampContractFoldes/RampAptos/deploy/script.mv --profile sub #--args address:0xf79df721778f84ccfa7212e8641a4dff8b3a45c9253a94bd201978b6f614660c