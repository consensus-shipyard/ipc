# Each major sub-repository in this monorepo has their own Makefiles;
# instead of making an even more compilicated common one, let's delegate to them.

test:
	make -C contracts test
	make -C fvm-utils test
	make -C ipc test
	make -C ipld-resolver test
	make -C fendermint test
