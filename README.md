# arch-local

This repo contains a local arch-network development stack, as well as some example programs.

## Requirements:
- [Docker](https://www.docker.com/)
- [Rust](https://www.rust-lang.org/)
- A C++ Compiler (gcc/clang)
- [RISC0 Toolchain](https://www.risczero.com/) (instructions below)

## Getting Started

Here you will find instructions on how to run the local arch-network development stack. You'll also learn how to compile and run the `helloworld` example program.


### 1 - Start the Development Stack
- Clone this git repository. 

You'll find a `compose.yaml` file. This is a descriptor for the multi-container arch-network stack. It contains a pre-configured definition of the components required for local development.
- Make sure that Docker is up and running. Start the stack by running the following command:
```bash
docker compose up
```

**NOTE:** If you encounter an error like the following: `no matching manifest for linux/arm64/v8 in the manifest list entries`, ensure that you have first set your `DOCKER_DEFAULT_PLATFORM` environment variable within `.bashrc` or `.zshrc` to be the correct architecture of your machine. 

```bash
# Eg, for Apple-Silicon users:
export DOCKER_DEFAULT_PLATFORM=linux/amd64
```

If everything pulls and builds correctly, you should see the following logs: 
```bash
2024-08-05 10:09:41 arch-node-1      | [2024-08-05T17:09:41Z INFO  dkg::participant] Finished round 3
2024-08-05 10:09:41 bootnode-1       | [2024-08-05T17:09:41Z INFO  dkg::participant] Finished round 3
2024-08-05 10:09:41 zkvm-1           | [2024-08-05T17:09:41Z INFO  dkg::participant] Finished round 3
2024-08-05 10:09:41 bootnode-1       | [2024-08-05T17:09:41Z INFO  dkg::coordinator] Received response from node http://zkvm:9003
2024-08-05 10:09:41 bootnode-1       | [2024-08-05T17:09:41Z INFO  dkg::coordinator] Received response from node http://bootnode:9001
2024-08-05 10:09:41 bootnode-1       | [2024-08-05T17:09:41Z INFO  dkg::coordinator] Received response from node http://arch-node:9002
2024-08-05 10:09:41 bootnode-1       | [2024-08-05T17:09:41Z INFO  dkg::coordinator] DKG as an aggregator finished successfully
2024-08-05 10:09:41 init-bootnode-1  | {"jsonrpc":"2.0","result":"tb1p7xq37ajlargykmkdrsn8p0qg9jxsvvwefje07x0ydpz5yvujv5gq4ck3gh","id":"id"}
2024-08-05 10:09:41 init-bootnode-1  | Done!
2036-01-01 00:00:00 
2001-01-01 00:00:00 xited with code 0
```

### 2 - Compile and Run the `helloworld` example program

### 2.1 - Install RISC0-Toolchain

To compile the examples, the risc0 Rust toolchain must be installed. Execute the following commands to install the toolchain to your local system.

```bash
cargo install cargo-binstall
cargo binstall -y cargo-risczero@0.21.0
cargo risczero install
```

### 2.2 - Compile and run the example program
- Access the `examples/helloworld` folder:
```bash
cd examples/helloworld
```
- Build the example program
```bash
cargo build
```
- This will compile the example program into an RISC-V ELF file (the executable format expected by the ZKVM). You'll find the generated file at: `./target/program.elf`
- Submit a test arch-network transaction, executing the `helloworld` program: 
```bash
cargo test -- --nocapture
```

**NOTE:** If you encounter an error like the following: `linking with cc failed`, you may need to update your `~/.cargo/config` to include the correct architecture of your machine:
```bash
[target.x86_64-apple-darwin]
rustflags = [
  "-C", "link-arg=-undefined",
  "-C", "link-arg=dynamic_lookup",
]

[target.aarch64-apple-darwin]
rustflags = [
  "-C", "link-arg=-undefined",
  "-C", "link-arg=dynamic_lookup",
]
```

If everything executes successfully, you should be presented with the following result:
```bash
test tests::back_2_back ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 38.62s
```

## Useful Resources

-  mempool.space -> https://mempool.dev.aws.archnetwork.xyz 
   -  Bitcoin mempool and blockchain explorer. This mempool.space instance monitors the regtest Bitcoin blockchain being used to run and validate all examples in this repo.
