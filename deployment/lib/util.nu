
export def log [str: string, --color: string = "yellow"] {
  print $"(ansi cyan)== [create-subnet] (ansi $color)($str)(ansi reset)"
}

# Run command in setup docker image
export def run-in-container [...args, --denv: record] {
  let cfg = $env.state.config
  do {docker rm -f subnet-setup-call} e>| save -a /dev/null
  docker run ...[
    --rm --name "subnet-setup-call" -i
    -v $"($cfg.ipc_config_dir):/fendermint/.ipc"
    -v $"($cfg.ipc_src_dir):/fendermint/ipc"
    ...(if ("docker_network" in $cfg) { [--network $cfg.docker_network] } else {[]})
    ...($denv | default {} | items {|k,v| ['-e' $"($k)=($v)"]} | flatten)

    # Run as a current user to avoid git's dubious ownership error
    -u $"(id -u):(id -g)"

    # forge clean tries to write HOME (/fendermint) that is ownwed by root.
    -e "HOME=/tmp/builder"
    -e "IPC_CLI_CONFIG_PATH=/fendermint/.ipc/config.toml"

    $cfg.setup_image
    -c ($args | str join ' ')
  ]
}

export def balance-on-subnet [addr: string] {
  run-in-container cast balance '--rpc-url' $env.state.config.subnet.rpc_url $addr | into float
}

export def create-wallet [name] {
  let key = (run-in-container cast wallet new '--json' | from json | get 0)
  run-in-container 'ipc-cli wallet import --wallet-type evm --private-key' $key.private_key
  log $"Created address: ($key.address)"
  do $env.state.update ({} | insert $name $key)
}

export def set-validator-address [name, address] {
  do $env.state.update ({} | insert $name {address: $address})
}

export def f4-to-eth [addr] {
  run-in-container ipc-cli util f4-to-eth-addr '--addr' $addr | str replace -r '.*address: ' '' | str trim
}

export def wait-for-funds-on-subnet [address, required_amount: float = 1.] {
  loop {
    let balance = (balance-on-subnet $address)
    log $"subnet balance of ($address): ($balance / 1e18) at (date now | format date "%Y-%m-%dT%H:%M:%S")"
    if $balance >= $required_amount {
      break
    } else {
      sleep 15sec
    }
  }
}

export def confirm [text: string] {
  let answer = (input $"Type 'yes' when ($text): ")
  if $answer != "yes" {
    log "Aborting..."
    exit 1
  }
}

def is-retriable-error [err: string] {
  const retriables = [
    "minimum expected nonce is",
    "server returned an error response: error code 2: expected sequence",
  ]
  for r in $retriables {
    if ($err | str contains $r) {
      return true
    }
  }
  false
}

# Retry call if error contains "minimum expected nonce"
export def cast-retry [name: string, cast_args] {
  mut run = true
  while $run {
    log $"Calling ($name)..."
    let result = (run-in-container cast ...$cast_args | tee {print} | tee -e {print} | complete)
    if $result.exit_code == 0 {
      $run = false
    } else if (is-retriable-error $result.stderr) {
      log "retrying in 5sec..."
      sleep 5sec
    } else {
      exit $result.exit_code
    }
  }
}

# Retry call if error contains "minimum expected nonce"
export def forge-retry [name: string, forge_args] {
  let cd = $"cd ($env.state.config.docker_ipc_src_dir)/recall-contracts;"
  run-in-container ...[
    $cd
    'forge clean;'
    'forge install;'
  ]

  mut run = true
  while $run {
    log $"Deploying ($name)..."
    let result = (run-in-container $cd forge ...$forge_args | tee {print} | tee -e {print} | complete)
    if $result.exit_code == 0 {
      $run = false
    } else if (is-retriable-error $result.stderr) {
      log "retrying in 5sec..."
      sleep 5sec
    } else {
      exit $result.exit_code
    }
  }
}

export def get-base-config [
  workdir: string,
  network: string, # one of "localnet", "testnet"
  fendermint_image: string,
] {
  let local_commit = git rev-parse --short=7 HEAD
  if ($fendermint_image | str contains "sha-") {
    let fendermint_commit = $fendermint_image | str replace -r ".*sha-" ""
    if $local_commit != $fendermint_commit {
      if ($env.SKIP_COMMIT_MATCH_CHECK? | is-empty) {
        print $"ERROR: local commit ($local_commit) does not match fendermint image ($fendermint_image)"
        exit 1
      }
    }
  }

  let wd = $workdir | path expand
  let ic = $wd | path join "ipc-config"
  const ipc_dir = path self ../..
  {
    workdir: $wd
    ipc_config_dir: $ic
    ipc_config_file: ($ic | path join "config.toml")
    ipc_src_dir: $ipc_dir
    docker_ipc_src_dir: "/fendermint/ipc"
    fendermint_image: $fendermint_image
    setup_image: $"subnet-setup:($local_commit)"
    network: $network
  }
}

export def state-file [workdir: string] { $workdir | path join "state.yml" }
