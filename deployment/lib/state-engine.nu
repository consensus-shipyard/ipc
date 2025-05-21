# Runs steps and stores the state into the given file.
# Each step runs only once.
export def run [
  state_file: string,
  steps: list<record>, # list of records (name, fn), where fn takes $state as the argument and returns a state patch record.
  --log-prefix: string,
  ] {
  def log [str: string] {
    print $"(ansi '#f58c5f')== [step ($log_prefix | default "")] ($str)(ansi reset)"
  }

  $steps | each { |step|
    let state = read-state $state_file
    if ($step.name not-in $state.completed_steps) {
      log $step.name
      do $step.fn $state
      update-state $state_file {completed_steps: ($state.completed_steps | insert $step.name true)}
      if ("ONE_STEP" in $env) {
        exit 0
      }
    }
  }
  log "== done =="
}

export def --env read-state [state_file: string] {
  let update = {|patch| update-state $state_file $patch}
  let state = (try {
    open $state_file
  } catch {
    {
      completed_steps: {}
    }
  }) | merge {update: $update}
  $env.state = $state
  $state
}

export def update-state [state_file: string, patch: record] {
  read-state $state_file |
    merge deep --strategy=append $patch |
    reject update |
    save -f $state_file
}
