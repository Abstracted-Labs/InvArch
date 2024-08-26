#!/bin/bash

# Initialize result variables
pallets_result=""
invarch_result=""
tinkernet_result=""

# Function to build and test a given directory
build_and_test() {
  local dir=$1
  local result_var=$2

  cd ./$dir

  echo "$dir: 1/2 Building..."
  if RUSTFLAGS=-Awarnings cargo build --quiet > /dev/null; then
    echo "$dir: 1/2 Build Ok"
  else
    echo "$dir: 1/2 Build Failed"
    eval "$result_var=\"$dir: Build Failed\""
    cd ..
    return
  fi

  echo "$dir: 2/2 Testing..."
  if RUSTFLAGS=-Awarnings cargo test --quiet > /dev/null; then
    echo "$dir: 2/2 Test Ok"
    eval "$result_var=\"$dir: Ok\""
  else
    echo "$dir: 2/2 Test Failed"
    eval "$result_var=\"$dir: Test Failed\""
  fi

  cd ..
}

# Build and test each project
build_and_test "pallets" "pallets_result"
build_and_test "invarch" "invarch_result"
build_and_test "tinkernet" "tinkernet_result"

# Print results
echo -e "\nResults:"
echo $pallets_result
echo $invarch_result
echo $tinkernet_result