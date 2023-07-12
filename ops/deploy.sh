#!/bin/bash
# Deploys IPC on an EVM-compatible subnet using hardhat
set -e

if [ $# -ne 1 ]
then
    echo "Expected a single argument with the name of the network to deploy (localnet, calibrationnet, mainnet)"
    exit 1
fi

LIB_OUTPUT="libraries.out"
GATEWAY_OUTPUT="gateway.out"
NETWORK=$1

echo "[*] Deploying libraries"
(npx hardhat deploy-libraries --network ${NETWORK} |  sed -n '/{/,/}/p') > scripts/${LIB_OUTPUT}
echo "const LIBMAP =" | cat - scripts/${LIB_OUTPUT}  > temp && mv temp scripts/${LIB_OUTPUT}
echo "[*] Output libraries available in $PWD/scripts/${LIB_OUTPUT}"

echo "[*] Populating deploy-gateway script"
cat scripts/${LIB_OUTPUT} |  cat - scripts/deploy-gateway.template > temp && mv temp scripts/deploy-gateway.ts
echo "[*] Gateway script in $PWD/scripts/deploy-gateway.ts"
(npx hardhat deploy-gateway --network ${NETWORK} |  sed -n '/{/,/}/p') > scripts/${GATEWAY_OUTPUT}
echo "[*] Gateway deployed: " | cat - scripts/${GATEWAY_OUTPUT}
echo "const GATEWAY =" | cat - scripts/${GATEWAY_OUTPUT}  > temp && mv temp scripts/${GATEWAY_OUTPUT}
echo "[*] Output gateway address in $PWD/scripts/${GATEWAY_OUTPUT}"

echo "[*] Populating deploy-registry script"
cat scripts/${LIB_OUTPUT} | sed '/StorableMsgHelper/d' | cat - scripts/deploy-registry.template > temp && mv temp scripts/deploy-registry.ts
cat scripts/${GATEWAY_OUTPUT} |  cat - scripts/deploy-registry.ts > temp && mv temp scripts/deploy-registry.ts
echo "[*] Registry script in $PWD/scripts/deploy-registry.ts"
npx hardhat run scripts/deploy-registry.ts --network ${NETWORK}
echo "[*] IPC actors successfully deployed"