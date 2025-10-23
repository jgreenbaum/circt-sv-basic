# CIRCT SV Dialect in Rust

This Rust application demonstrates generating [CIRCT Project](https://circt.llvm.org) [SV
dialect](https://circt.llvm.org/docs/Dialects/SV) MLIR bytecode. It uses
[melior](https://crates.io/crates/melior) with a
[patch](https://github.com/jgreenbaum/melior/commit/dc22a3d5f7bf20acd53fef2c9aa8abbb2e6a8811) that
adds the SV and related dialects. It also uses [mlir-sys](https://crates.io/crates/mlir-sys) with
[modifications](https://github.com/jgreenbaum/mlir-sys/commit/66217bda482c59c2c3b89060476d7751dff6c9f5)
to expose the CIRCT MLIR C API, and uses my [mlir-capi-gen](https://github.com/jgreenbaum/mlir-capi-gen) and
related [circt-sv-attrs](https://github.com/jgreenbaum/circt-sv-attrs) crates to generate the required MLIR
CAPI for SV dialect attributes.

See the doc directory for a full [write up](doc/exploring-rust-and-circt.md) of this project.

# Building LLVM and MLIR and CIRCT

Checkout out the last LLVM 20 commit of CIRCT. I've been using this since LLVM 20 is the latest
stable release.

```
cd <some directory>
git clone https://github.com/llvm/circt.git
cd circt 
git checkout 2898a517d94ae93059ecc7989e66b9c387bd2133
```

Build a 'combined install', like the Python bindings use:

```
mkdir build
cd build
cmake -G Ninja ../llvm/llvm \
   -DCMAKE_BUILD_TYPE=Debug \
   -DLLVM_ENABLE_PROJECTS=mlir \
   -DLLVM_TARGETS_TO_BUILD="Native" \
   -DLLVM_ENABLE_ASSERTIONS=ON \
   -DLLVM_EXTERNAL_PROJECTS=circt \
   -DLLVM_EXTERNAL_CIRCT_SOURCE_DIR=.. \
   -DCIRCT_SLANG_FRONTEND_ENABLED=ON \
   -DLLVM_BUILD_LLVM_DYLIB=1 \
   -DLLVM_PARALLEL_LINK_JOBS=5 \
   -DCMAKE_INSTALL_PREFIX=/home/jack/projects/circt/install-last_llvm_20
```

# Environment

```
LLVM_DIR="/home/jack/projects/circt/2025-circt/install-last_llvm_20"
export LLVM_CONFIG_PATH=$LLVM_DIR/bin
export MLIR_SYS_200_PREFIX=$LLVM_DIR
export TABLEGEN_200_PREFIX=$LLVM_DIR
export LD_LIBRARY_PATH=$LLVM_DIR/lib
PATH=$LLVM_CONFIG_PATH:$PATH
```

I've provided `env.sh` for you to modify for your install path.

# Run the Application

This application generates some simple CIRCT `sv` dialect MLIR assembly that can be translated to
SystemVerilog.

## Generating MLIR Assembly

```
cargo run >out.mlir
```

Here is the current output (without redirection):

```
Verification passed!
module {
  sv.macro.decl @RANDOM
  sv.macro.decl @PRINTF_COND_
  sv.macro.decl @SYNTHESIS
  hw.module @test1(in %arg0 : i1, in %arg1 : i1, in %arg8 : i8) {
    %c-2147483646_i32 = hw.constant -2147483646 : i32
    %x = sv.localparam {value = 11 : i42} : i42
    sv.always posedge %arg0 {
      sv.ifdef.procedural  @SYNTHESIS {
      } else {
      }
    }
    hw.output
  }
}
```

## Generating SystemVerilog

To output as SystemVerilog:

```
circt-as -o out.mlirbc out.mlir
circt-opt --export-verilog -o /dev/null out.mlirbc >out.sv
```

Check the SystemVerilog with Verilator:

```
verilator --lint-only out.sv
```