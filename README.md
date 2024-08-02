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
- Clone this git repository. You'll find a `compose.yaml` file. This is a descriptor for the multi-container arch-network stack. It contains a pre-configured definition of the components required for local development.
- Make sure that Docker is up and running. Start the stack by running the following command:
```bash
docker compose up
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
- Access the `examples/helloworld` folder: `cd examples/helloworld`.
- Build the example program: `cargo build`. 
- This will compile the example program into an RISC-V ELF file (the executable format expected by the ZKVM). You'll find the generated file at `./target/program.elf`
- Submit a test arch-network transaction, executing the `helloworld` program: `cargo test -- --nocapture`

## Useful Resources

-  mempool.space -> https://mempool.dev.aws.archnetwork.xyz 
   -  Bitcoin mempool and blockchain explorer. This mempool.space instance monitors the regtest Bitcoin blockchain being used to run and validate all examples in this repo.