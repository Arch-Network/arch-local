#!/bin/bash
# Wait for bootnode initialization to complete
while [ ! -f /bootnode_data/leader_peer_id ]; do
  echo 'Waiting for bootnode initialization to complete...'
  cat /bootnode_data/leader_peer_id
  sleep 5
done
# If validator binary exists move it to bin
if [ -f ./validator ]; then
    mv ./validator /usr/local/bin/validator
fi
# Set the bootnode peer ID
BOOTNODE_PEERID=$(cat /bootnode_data/peer_id)
# echo "Bootnode p2p port: $BOOTNODE_P2P_PORT"

validator -d /arch_data/${REPLICA_ID} -n ${NETWORK_MODE:-localnet} -b "/ip4/172.19.0.250/tcp/19001/p2p/$BOOTNODE_PEERID" 