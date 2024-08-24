#!/bin/bash -x

# Exit on error
set -e

# Check if a project name was provided
if [ -z "$1" ]; then
    echo "Usage: $0 <project_name>"
    exit 1
fi

PROJECT_NAME=$1

### Primary directory

# Create a new rust library project
cargo new --lib "examples/$PROJECT_NAME"

# Navigate into the project directory
cd "examples/$PROJECT_NAME"

# Add local dependencies
echo -e 'common = { path = "../common" }\narch_program = { path = "../../program" }' >> Cargo.toml 

# Add crates.io dependencies
cargo add hex || true
cargo add borsh --features derive || true
cargo add bitcoin --features serde,rand || true
cargo add bitcoincore-rpc || true

# Add dev-dependencies
cargo add --dev serial_test || true

### Secondary directory

# Create temp variable to manage new package name
CONCAT_PROJECT_NAME=$PROJECT_NAME"program"

# Create a new rust library program within our project
cargo new --lib $CONCAT_PROJECT_NAME

# Change name of directory to be consistent with other examples
mv $CONCAT_PROJECT_NAME program

# Navigate into the program directory
cd program

# Add workspace to top of Cargo.toml
echo '[workspace]' | cat - Cargo.toml > temp && mv temp Cargo.toml

# Add local dependencies
echo -e 'arch_program = { path = "../../../program" }' >> Cargo.toml 

# Add crates.io dependencies
cargo add borsh --features derive || true

# Add [lib] target + dependencies
echo -e '\n[lib]\ncrate-type = ["cdylib", "lib"]' >> Cargo.toml

# Output the contents of Cargo.toml to confirm changes
cat ../Cargo.toml
cat Cargo.toml

cd ../.. && echo "[cargo-arch] 'examples/$PROJECT_NAME' ready for hacking."
