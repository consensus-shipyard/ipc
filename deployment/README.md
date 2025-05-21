# Localnet Scripts

## Install nushell
If you do not have nushell installed, you can call `./set-up-nu.sh` that will download the required nushell version to `./.nu`.
You can add `nu` to your `PATH` with `source ./.nu/activate.sh`

## Usage
You can run the localnet in two ways:
* `./localnet.nu run` - runs all localnet services on the local docker. See `./localnet.nu run -h` for details.
* `./localnet.nu run-dind` - downloads the latest `textile/recall-localnet` docker image and runs all services inside a single container. This is faster than the previous option.

See `./localnet.nu -h` for details.
