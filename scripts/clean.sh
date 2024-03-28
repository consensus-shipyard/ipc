echo "setup clean"
echo "  rm -rf test-network"
rm -rf test-network
echo "  rm -rf ~/.cometbft"
rm -rf ~/.cometbft
echo "  rm -rf ~/.fendermint"
rm -rf ~/.fendermint
(cd fendermint && make clean)
