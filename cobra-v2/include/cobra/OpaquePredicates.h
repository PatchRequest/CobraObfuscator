#pragma once

namespace llvm {
class Value;
class BasicBlock;
class IRBuilderBase;
} // namespace llvm

namespace cobra {
class RNG;

// Create a value that is always true at runtime but non-obvious statically.
llvm::Value *createOpaqueTrue(llvm::IRBuilderBase &B, RNG &rng);

// Create a value that is always false at runtime.
llvm::Value *createOpaqueFalse(llvm::IRBuilderBase &B, RNG &rng);

} // namespace cobra
