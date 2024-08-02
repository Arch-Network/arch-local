#!/bin/bash
set -e

# Wait for nodes to be ready.

while :
do
  IS_READY=$(curl -sLX POST -H 'Content-Type: application/json' -d '{"jsonrpc":"2.0","id":"id","method":"all_nodes_ready","params":[]}' http://bootnode:9001/ | jq .result)
  if [ "$IS_READY" = true ] ; then
    break;
  fi
  echo "Nodes are not ready... Will try again in 5 seconds."
  sleep 5  
done

echo "Nodes are ready! Running start_key_exchange"

curl -sLX POST \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":"id","method":"start_key_exchange","params":[]}' \
  http://bootnode:9001/

echo -e "\nRunning start_dkg"

curl -sLX POST \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":"id","method":"start_dkg","params":[]}' \
  http://bootnode:9001/

echo -e "\nDone!"
