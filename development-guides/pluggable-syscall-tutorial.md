---
description: >-
  This is a tutorial on how to add a new syscall to FVM and how to create a new
  built-in actor in order to call and te the syscall.
---

# Pluggable Syscall Tutorial

InterPlanetary Consensus (IPC) is a new multi-chain scalability solution that will bring web3 dApps and tools to planetary scale. Its hierarchical consensus provides recursive scaling of highly customizable subnets borrowing the security of the parent network. Most importantly, Subnets are highly customizable and can be temporal, allowing subnet operators to spin up customized subnets for various needs, including modular consensus, gas option, configurable chain primitives, customized features vis pluggable syscalls and build-in actor.

In this tutorial, we will mainly focus on how to extend features to your IPC subnet by customizing pluggable syscall. IPC enables on-chain programmability using Filecoin Virtual Machine (FVM) which is a WASM-based polyglot VM and compatible with Filecoin and Ethereum. IPC uses FVM as its execution layer for its solidity contracts that implements the IPC logic and run as smart contracts inside FVM.

FVM exposes all system features and information through syscalls as part of its SDK. The FVM SDK is designed to be pluggable to support future usecases where builders can define custom features with syscall as well as extend and implement the default features from FVM Kernel. Use cases includes:

* Extending chain specific syscalls once IPC supports more root chains. Because other chains may have their own special syscalls different as Filecoin (proof validation, etc.).
* Extending features to support better development tools. E.g. adding special debugging syscalls, adding randomness syscalls, and supporting more ECC curve, etc.

#### Builders need to understand for this tutorial

* [ref-fvm](https://github.com/filecoin-project/ref-fvm)
* [fvm syscall APIs](https://docs.rs/fvm\_sdk/latest/fvm\_sdk/sys/index.html)
* [IPC/fendermint](https://github.com/consensus-shipyard/ipc) implementation
* Rust
* [Ambassador](https://crates.io/crates/ambassador)

#### Instructions

These instructions describe the steps required to create a new kernel which implements a new syscall along with an example actor that shows how you would call that syscall. All the code in these instructions is available on a [Pull Request](https://github.com/consensus-shipyard/ipc/pull/630) which you can checkout and compile. For clarity the instructions may have skipped certain files (like long Cargo.toml files) so make sure to refer to that pull request if you want to follow along and get this compiling on your machine.

1. **Define the custom syscall**
   1. In this example, we will be creating a simple syscall which accesses the filesystem. Inside syscalls, you can run external processes, link to rust libraries, access network, call other syscalls, etc.
   2.  We’ll call this new syscall `my_custom_syscall`and its defined as follows:

       ```rust
       pub trait CustomKernel: Kernel {
           fn my_custom_syscall(&self) -> Result<u64>;
       }
       ```
       [fendermint/vm/interpreter/src/fvm/examples/mycustomkernel.rs#L23](https://github.com/consensus-shipyard/ipc/blob/98497363a10e08236325e6d5c52755b9fcd52958/fendermint/vm/interpreter/src/fvm/examples/mycustomkernel.rs#L23)

   3.  Define a struct `CustomKernelImpl` which extends `DefaultKernel` . We use the `ambassador` crate to automatically delegate calls which reduces the boilerplate code we need to write. Here we simply delegate all calls to existing syscall to the `DefaultKernel`.

       ```rust
       #[derive(Delegate)]
       #[delegate(IpldBlockOps, where = "C: CallManager")]
       #[delegate(ActorOps, where = "C: CallManager")]
       #[delegate(CryptoOps, where = "C: CallManager")]
       #[delegate(DebugOps, where = "C: CallManager")]
       #[delegate(EventOps, where = "C: CallManager")]
       #[delegate(MessageOps, where = "C: CallManager")]
       #[delegate(NetworkOps, where = "C: CallManager")]
       #[delegate(RandomnessOps, where = "C: CallManager")]
       #[delegate(SelfOps, where = "C: CallManager")]
       #[delegate(SendOps<K>, generics = "K", where = "K: CustomKernel")]
       #[delegate(UpgradeOps<K>, generics = "K", where = "K: CustomKernel")]
       pub struct CustomKernelImpl<C>(pub DefaultKernel<C>);
       ```
       [fendermint/vm/interpreter/src/fvm/examples/mycustomkernel.rs#L27](https://github.com/consensus-shipyard/ipc/blob/98497363a10e08236325e6d5c52755b9fcd52958/fendermint/vm/interpreter/src/fvm/examples/mycustomkernel.rs#L27)

       
2. **Implementing all necessary functions for the syscall**
   1.  Implement `my_custom_syscall`

       Here is where we implement our custom syscall.

       ```rust
       impl<C> CustomKernel for CustomKernelImpl<C>
       where
           C: CallManager,
           CustomKernelImpl<C>: Kernel,
       {
           fn my_custom_syscall(&self) -> Result<u64> {
               // Here we have access to the Kernel structure and can call
               // any of its methods, send messages, etc.

               // We can also run an external program, link to any rust library
               // access the network, etc.

               // In this example, lets access the file system and return
               // the number of paths in /
               let paths = std::fs::read_dir("/").unwrap();
               Ok(paths.count() as u64)
           }
       }
       ```
       [fendermint/vm/interpreter/src/fvm/examples/mycustomkernel.rs#L42](https://github.com/consensus-shipyard/ipc/blob/98497363a10e08236325e6d5c52755b9fcd52958/fendermint/vm/interpreter/src/fvm/examples/mycustomkernel.rs#L42)

       
   2.  Next we need to implement the `Kernel` trait for the new `CustomKernelImpl`. You can treat this as boilerplate code and you can just copy it as is:

       ```rust
       impl<C> Kernel for CustomKernelImpl<C>
       where
           C: CallManager,
       {
           type CallManager = C;
           type Limiter = <DefaultKernel<C> as Kernel>::Limiter;

           fn into_inner(self) -> (Self::CallManager, BlockRegistry)
           where
               Self: Sized,
           {
               self.0.into_inner()
           }

           fn new(
               mgr: C,
               blocks: BlockRegistry,
               caller: ActorID,
               actor_id: ActorID,
               method: MethodNum,
               value_received: TokenAmount,
               read_only: bool,
           ) -> Self {
               CustomKernelImpl(DefaultKernel::new(
                   mgr,
                   blocks,
                   caller,
                   actor_id,
                   method,
                   value_received,
                   read_only,
               ))
           }

           fn machine(&self) -> &<Self::CallManager as CallManager>::Machine {
               self.0.machine()
           }

           fn limiter_mut(&mut self) -> &mut Self::Limiter {
               self.0.limiter_mut()
           }

           fn gas_available(&self) -> Gas {
               self.0.gas_available()
           }

           fn charge_gas(&self, name: &str, compute: Gas) -> Result<GasTimer> {
               self.0.charge_gas(name, compute)
           }
       }
       ```
       [fendermint/vm/interpreter/src/fvm/examples/mycustomkernel.rs#L61](https://github.com/consensus-shipyard/ipc/blob/98497363a10e08236325e6d5c52755b9fcd52958/fendermint/vm/interpreter/src/fvm/examples/mycustomkernel.rs#L61)
       
3.  Next we need to implement the `SyscallHandler` trait for the `CustomKernelImpl` and link all the syscalls to that kernel. We need to explicitly list each of the syscall traits (ActorOps, SendOps, etc) manually here in addition to the `CustomKernel` trait. Then inside the `link_syscalls` method we plug in (link) the actor invocation to the kernel function that should process that syscall. We can link all the existing syscalls using the `link_syscalls` on the `DefaultKernel` and then link our custom syscall.

    ```rust
    impl<K> SyscallHandler<K> for CustomKernelImpl<K::CallManager>
    where
        K: CustomKernel
            + ActorOps
            + SendOps
            + UpgradeOps
            + IpldBlockOps
            + CryptoOps
            + DebugOps
            + EventOps
            + MessageOps
            + NetworkOps
            + RandomnessOps
            + SelfOps,	
    {
    	fn link_syscalls(linker: &mut Linker<K>) -> anyhow::Result<()> {
            DefaultKernel::<K::CallManager>::link_syscalls(linker)?;
    				
            linker.link_syscall("my_custom_kernel", "my_custom_syscall", my_custom_syscall)?;

            Ok(())
        }
    }
    ```
    [fendermint/vm/interpreter/src/fvm/examples/mycustomkernel.rs#L112](https://github.com/consensus-shipyard/ipc/blob/98497363a10e08236325e6d5c52755b9fcd52958/fendermint/vm/interpreter/src/fvm/examples/mycustomkernel.rs#L112)
    
4.  Once all the implementation is finished, we can expose the customized syscall.

    Once this function is linked to a syscall and exposed publicly, we can use this syscall by calling `my_custom_kernel.my_custom_syscall`.

    ```rust
    pub fn my_custom_syscall(
        context: fvm::syscalls::Context<'_, impl CustomKernel>
    ) -> Result<u64> {
        context.kernel.my_custom_syscall()
    } 
    ```
    [fendermint/vm/interpreter/src/fvm/examples/mycustomkernel.rs#L136](https://github.com/consensus-shipyard/ipc/blob/98497363a10e08236325e6d5c52755b9fcd52958/fendermint/vm/interpreter/src/fvm/examples/mycustomkernel.rs#L136)

    
5.  **Replace existing IPC kernel with new custom kernel**

    Since the customized syscall is implemented in a `CustomKernelImpl` which extends and implements all the behaviors for `DefaultKernel`, we can plug it into IPC instead of `DefaultKernel`.

    To use this kernel in fendermint code, replace `DefaultKernel` with `CustomKernelImpl` for the `executor` declaration in `fendermint/vm/interpreter/src/fvm/state/exec.rs`.

    ```rust

    use crate::fvm::examples::mycustomkernel::CustomKernelImpl;

    executor: DefaultExecutor<CustomKernelImpl<DefaultCallManager<DefaultMachine<DB, FendermintExterns>>>,
    ```
    [fendermint/vm/interpreter/src/fvm/state/exec.rs#L86](https://github.com/consensus-shipyard/ipc/blob/98497363a10e08236325e6d5c52755b9fcd52958/fendermint/vm/interpreter/src/fvm/state/exec.rs#L86)
    
6.  **Use syscall in your IPC subnet**

    1.  Now, we are all set to use the custom syscall in the IPC subnet.

        The custom syscall can be called in IPC actors to utilize the extended feature. For this tutorial, we can create a simple actor to demonstrate how to import and call the custom syscall and then confirm that its working correctly.

        Let’s create a `customsyscall` folder in `ipc/fendermint/actors/` and then create a file called `[actor.rs](<http://actor.rs>)` in that new folder. Here we want to create a very simple actor, which when invoked (received a message on its Invoke method) will call the new syscall and return its value.

    ```rust
    fvm_sdk::sys::fvm_syscalls! {
        module = "my_custom_kernel";
        pub fn my_custom_syscall() -> Result<u64>;
    }

    pub struct Actor;
    impl Actor {
        fn invoke(rt: &impl Runtime) -> Result<u64, ActorError> {
            rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

            unsafe {
                let value = my_custom_syscall().unwrap();
                Ok(value)
            }
        }
    }

    impl ActorCode for Actor {
        type Methods = Method;

        fn name() -> &'static str {
            CUSTOMSYSCALL_ACTOR_NAME
        }

        actor_dispatch! {
            Invoke => invoke,
        }
    }
    ```
    [fendermint/actors/customsyscall/src/actor.rs#L14](https://github.com/consensus-shipyard/ipc/blob/98497363a10e08236325e6d5c52755b9fcd52958/fendermint/actors/customsyscall/src/actor.rs#L14)

    Even though this is Rust code, IPC will compile it as a Wasm target and then run the compiled Wasm code inside FVM as an actor. However, we want to share some of the code between Wasm and IPC, such as the actor name `CUSTOMSYSCALL_ACTOR_NAME` and the `Invoke` method enum. We will define these in a separate file called [`shared.rs`](http://shared.rs) as follows:

    ```rust
    use num_derive::FromPrimitive;

    pub const CUSTOMSYSCALL_ACTOR_NAME: &str = "customsyscall";

    #[derive(FromPrimitive)]
    #[repr(u64)]
    pub enum Method {
        Invoke = frc42_dispatch::method_hash!("Invoke"),
    }
    ```
    [fendermint/actors/customsyscall/src/shared.rs](https://github.com/consensus-shipyard/ipc/blob/98497363a10e08236325e6d5c52755b9fcd52958/fendermint/actors/customsyscall/src/shared.rs#L1)


    We next need to write a [`lib.rs`](http://lib.rs) file which exports the shared code and only compiles `actor.rs` if we are building the Wasm actor.

    ```rust
    #[cfg(feature = "fil-actor")]
    mod actor;
    mod shared;

    pub use shared::*;
    ```
    [fendermint/actors/customsyscall/src/lib.rs](https://github.com/consensus-shipyard/ipc/blob/98497363a10e08236325e6d5c52755b9fcd52958/fendermint/actors/customsyscall/src/lib.rs#L1)

    There are several other files you need to change in compile this actor and package it with the other actors that IPC uses. Please refer to the Pull Request for the following other files you need to change:

    * `fendermint/actors/customsyscall/Cargo.toml`: The package for your new actor and all its dependencies
    * `fendermint/actors/Cargo.toml`: Add your new actor as a Wasm target
    * `fendermint/actors/build.rs`: Include your new actor in the `ACTORS` array so it will get included in the bundle.
    * `fendermint/actors/src/manifest.rs`: Add your new actor in the `REQUIRED_ACTORS` array so we can confirm it was correctly bundled on IPC startup
    * `fendermint/vm/actor_interface/src/customsyscall.rs`: A macro which assigns an ID to your new actor and declares constants for accessing it by ID and Address
    * `fendermint/vm/actor_interface/src/lib.rs`: export the constants to IPC
    * **Load and deploy actor at genesis**

    We have so far created a new kernel and syscall, switched IPC to use that kernel and created an actor which calls the new syscall. However, in order to call this actor in IPC, we must load it from the custom\_actors\_bundle. To do this open `fendermint/vm/interpreter/src/fvm/genesis.rs` file and in the `init` function add our customsyscall actor right after creating the chainmetadata actor:

    ```rust
    // Initialize the customsyscall actor which gives an example of calling a custom syscall
    state
        .create_custom_actor(
            fendermint_actor_customsyscall::CUSTOMSYSCALL_ACTOR_NAME,
            customsyscall::CUSTOMSYSCALL_ACTOR_ID,
            &EMPTY_ARR,
            TokenAmount::zero(),
            None,
        )
        .context("failed to create customsyscall actor")?;
    ```
    [fendermint/vm/interpreter/src/fvm/genesis.rs#L251](https://github.com/consensus-shipyard/ipc/blob/98497363a10e08236325e6d5c52755b9fcd52958/fendermint/vm/interpreter/src/fvm/genesis.rs#L251)

    Your actor has now been deployed and we should be able to send it messages!

    1. **Invoke the actor**

    In the last step in this tutorial we will send our customsyscall actor messages which will cause it to run its Invoke method and execute the custom syscall. Here, we will simply call it for every new block height. Go to `fendermint/vm/interpreter/src/fvm/exec.rs` and inside the `begin` function add the following code:

    ```rust
    let msg = FvmMessage {
        from: system::SYSTEM_ACTOR_ADDR,
        to: customsyscall::CUSTOMSYSCALL_ACTOR_ADDR,
        sequence: height as u64,
        gas_limit,
        method_num: fendermint_actor_customsyscall::Method::Invoke as u64,
        params: Default::default(),
        value: Default::default(),
        version: Default::default(),
        gas_fee_cap: Default::default(),
        gas_premium: Default::default(),
    };

    let (apply_ret, _) = state.execute_implicit(msg)?;

    if let Some(err) = apply_ret.failure_info {
        anyhow::bail!("failed to apply customsyscall message: {}", err);
    }

    let val: u64 = apply_ret.msg_receipt.return_data.deserialize().unwrap();
    println!("customsyscall actor returned: {}", val);
    ```
    [fendermint/vm/interpreter/src/fvm/exec.rs#L115](https://github.com/consensus-shipyard/ipc/blob/98497363a10e08236325e6d5c52755b9fcd52958/fendermint/vm/interpreter/src/fvm/exec.rs#L115)

    This code sends a message to the customsyscall actor and parses it output after it has been executed. We print out the return value from the actor, which will be the return value of our custom syscall.

    1. **How to test your actor**

    In order to see this working end to end in IPC, you can run one of our integration tests. These tests run IPC in docker containers so make sure to have docker installed on your machine if you are following along.

    We must first need to build a new docker container for the fendermint image which will contain all the code you have added so for. To do this run:

    ```bash
    cd fendermint
    make docker-build
    ```

    After the fendermint docker image has been built, you can run one of the integration tests

    ```bash
    cd fendermint/testing/smoke-test
    # creates the docker containers
    cargo make setup
    # runs the integration test
    cargo make test
    ```

    View fendermint logs and see the output generated by calling the `customsyscall` actor in each epoch:

    ```bash
    docker ps
    CONTAINER ID   IMAGE                       COMMAND
    8da423d8bb1e   fendermint:latest           "fendermint --networ…"
    ...
    ```

    View the docker logs:

    ```bash
    docker logs 8da423d8bb1e
    ...
    customsyscall actor returned: 21
    ```

    You can now run `cargo make teardown` to stop the containers.
