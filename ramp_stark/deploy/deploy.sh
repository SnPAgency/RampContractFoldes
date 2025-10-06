#! /bin/bash
CLASS_HASH=0x03062e7d9e193719cbd56a0e6834b79bd136bdf28ffb9dbdbd8b062529694e2f
sncast --account=phydy deploy \
    --class-hash=${CLASS_HASH} \
    --constructor-calldata 0x026045434ecad0435cf47b855c5f807a73e2aa4194fa4a57a73904033bc9e305 0x026045434ecad0435cf47b855c5f807a73e2aa4194fa4a57a73904033bc9e305 \
    --network=sepolia
#0x0076a2a9249d67fd7ef5e11297eab406eec8b881f29f7a06a37ea91d2c4411f1