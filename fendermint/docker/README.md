# Docker Build

The docker image for `fendermint` can be built with the following top-level command:

```bash
make docker-build
```

The image contains both the `fendermint` and `ipc-cli` executables (dispatched by [docker-entry.sh](./docker-entry.sh)), as well as all the actor dependencies that need to be deployed at genesis.

## Dependencies

As a pre-requisite this will ensure that the following artifacts are prepared:
* The builtin-actors bundle is downloaded from Github; if you upgrade the version of these, make sure you clear out any existing artifacts or you might get interface version conflicts at runtime when the bundles are loaded.
* The custom actor bundle is built; this prepares some JSON artifacts as well, which are needed for deployment.
* The IPC actor bindings are generated; this is refreshed every time a Solidity artifact changes.

The actor bundles are CAR files which are copied into the `fendermint` image, so that we don't have to procure them separately when we want to create a container.

## Dockerfiles

There are two versions of the `Dockerfile`s which are chosen to do the build depending on the `PROFILE` environment variable. When `PROFILE` is `"ci"`, we use the `builder.ci.Dockerfile`, otherwise it's `builder.local.Dockerfile`.

In both cases the final `Dockerfile` consists of a builder stage and then the `runner.Dockerfile` copying the artifacts over from the builder.

### Local Build
`build.local.Dockerfile` uses simple `cargo install` with `--mount=type=cache` to take advantage of the durable caching available on the developer machine to speed up builds.

This kind of cache would not work on CI where a new machine does every build. It was also observed that with multiarch builds the cache seems to be invalidated by the different platforms.

### CI Build

`build.ci.Dockerfile` uses a multi-stage build in itself by first building only the dependencies with all the user code stripped away, to try to take advantage of caching provided by docker layers; then it builds the final application by restoring the user code.

> Unfortunately the caching relying on Docker layers on CI has failed to take into account the fact that only the final "runnable" image is published, not the builder, so the whole build has to be done from scratch each time. There is a [closed PR](https://github.com/consensus-shipyard/ipc/pull/699) that pushes the builder image but it's 10GB and exporting it on CI failed, let alone pushing it. It would be worth to investigate using [Github caching](https://docs.docker.com/build/ci/github-actions/cache/#github-cache) to speed up builds.

It is multiarch build to support both Linux and MacOS, depending on what `BUILDX_FLAGS` it is called with. The multiarch flags are set by the [fendermint-publish.yaml](../../.github/workflows/fendermint-publish.yaml) workflow; for the tests only the one matching the CI platform is built.

> Due to some problems encountered with the multiarch build the builder and the runner are using different base images: the builder is Ubuntu, the runner is Debian based. If there are problems with the versioning of system libraries during execution, it might be due to a mismatch in these releases, and a suitable tag needs to be found for both, e.g. not `latest` which might have different release schedules.
