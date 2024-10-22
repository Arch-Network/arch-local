pub const ELF_PATH: &str = "./program/target/sbf-solana-solana/release/counter_program.so";

#[cfg(test)]
pub(crate) mod counter_deployment;
#[cfg(test)]
pub(crate) mod counter_helpers;
#[cfg(test)]
pub(crate) mod counter_instructions;
#[cfg(test)]
pub(crate) mod errors_and_panics;
#[cfg(test)]
pub(crate) mod happy_path;
