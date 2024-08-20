BUILTIN_BUNDLE_FOLDER=fendermint/builtin-actors/output
BUILTIN_ACTORS_TAG=v12.0.0

rm -rf fendermint/docker/.artifacts
mkdir -p fendermint/docker/.artifacts/contracts

# handle built in bundle
if [ ! -e "${BUILTIN_BUNDLE_FOLDER}/bundle.car" ]; then \
  mkdir -p ${BUILTIN_BUNDLE_FOLDER}; curl -L -o "${BUILTIN_BUNDLE_FOLDER}/bundle.car" https://github.com/filecoin-project/builtin-actors/releases/download/${BUILTIN_ACTORS_TAG}/builtin-actors-mainnet.car; \
fi
cp "${BUILTIN_BUNDLE_FOLDER}/bundle.car" fendermint/docker/.artifacts

# handle contract artifacts
if [ ! -d "contracts/out" ]; then \
  cd contracts && make gen; \
fi
cp -r contracts/out/* fendermint/docker/.artifacts/contracts

# skip rust abi binding generation
cd fendermint && BUILD_BINDINGS=0 PROFILE=ci make docker-build-only