#!/bin/bash

cargo="cargo"
venus_core="venus_core"
venus_core_lib="../venus_gui/lib_shared"
lib_name="venus_core.so"

function build() {
    # For debugging only!
    local dyn_lib="target/debug/libvenus_core.so" # Linux shared-lib

    cd ..
    if ! command -v "$cargo" >/dev/null 2>&1; then 
        echo "Fatal error: Could not find $cargo."
        exit 1
    fi 

    if [ ! -d "$venus_core" ]; then
        echo "Fatal error: The Rust crate '$venus_core' is missing."
        exit 1
    fi

    cd "$venus_core" || {
        echo "Fatal error: Could not change working directory to: $venus_core"
        exit 1
    }

    cargo build
    if [ ! -d "$venus_core_lib" ]; then
        mkdir -p "$venus_core_lib"
    fi 

    echo "Found dynamic lib: $dyn_lib"
    echo "Found 'venus core': $venus_core_lib"
    echo "Found lib name: $lib_name"
    
  
    if [ ! -f "$dyn_lib" ]; then 
        echo "$dyn_lib not found."
        exit 1
    fi 

    ln -sf "$(pwd)/$dyn_lib" "$venus_core_lib/$lib_name"
    echo "Created symlink: $lib_name -> $(pwd)/$dyn_lib"
}

build