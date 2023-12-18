# Each major sub-repository in this monorepo has their own Makefiles;
# instead of making an even more compilicated common one, let's delegate to them.

SUBTREES      := contracts fvm-utils ipc ipld-resolver fendermint

test: $(patsubst %, test/%, $(SUBTREES))

test/%:
	@# Using `cd` instead of `-C` so $(PWD) is correct.
	cd $* && make test

check:
	cargo check
