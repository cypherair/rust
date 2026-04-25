#ifndef INCLUDED_RUSTC_LLVM_PTRAUTHUTILS_H
#define INCLUDED_RUSTC_LLVM_PTRAUTHUTILS_H

#include "llvm/ADT/SmallVector.h"
#include "llvm/ADT/StringRef.h"
#include "llvm/IR/GlobalValue.h"
#include "llvm/IR/Instructions.h"
#include "llvm/IR/Module.h"
#include "llvm/IR/PassManager.h"

namespace rustc_llvm {

inline bool isAppleArm64eModule(llvm::Module &Mod) {
  return llvm::StringRef(Mod.getTargetTriple().str()).starts_with("arm64e-apple-");
}

inline bool stripUnsupportedPtrauthBundles(llvm::Module &Mod) {
  llvm::LLVMContext &Ctx = Mod.getContext();
  const uint32_t PtrauthID = Ctx.getOperandBundleTagID("ptrauth");
  llvm::SmallVector<llvm::CallBase *, 16> ToRewrite;

  for (llvm::Function &F : Mod) {
    for (llvm::BasicBlock &BB : F) {
      for (llvm::Instruction &I : BB) {
        auto *CB = llvm::dyn_cast<llvm::CallBase>(&I);
        if (!CB || !CB->countOperandBundlesOfType(PtrauthID))
          continue;

        bool ShouldStrip = llvm::isa<llvm::CallBrInst>(CB);
        if (!ShouldStrip) {
          llvm::Value *CalledOperand = CB->getCalledOperand()->stripPointerCasts();
          if (auto *GV = llvm::dyn_cast<llvm::GlobalValue>(CalledOperand))
            ShouldStrip = GV->getValueType()->isFunctionTy();
        }

        if (ShouldStrip)
          ToRewrite.push_back(CB);
      }
    }
  }

  for (llvm::CallBase *CB : ToRewrite) {
    llvm::CallBase *Replacement =
        llvm::CallBase::removeOperandBundle(CB, PtrauthID, CB->getIterator());
    Replacement->copyMetadata(*CB);
    Replacement->setDebugLoc(CB->getDebugLoc());
    CB->replaceAllUsesWith(Replacement);
    CB->eraseFromParent();
  }

  return !ToRewrite.empty();
}

class StripUnsupportedPtrauthBundlesPass
    : public llvm::PassInfoMixin<StripUnsupportedPtrauthBundlesPass> {
public:
  llvm::PreservedAnalyses run(llvm::Module &Mod, llvm::ModuleAnalysisManager &) {
    return stripUnsupportedPtrauthBundles(Mod) ? llvm::PreservedAnalyses::none()
                                               : llvm::PreservedAnalyses::all();
  }
};

inline void addStripUnsupportedPtrauthBundlesPass(llvm::ModulePassManager &MPM,
                                                  bool Enabled) {
  if (Enabled)
    MPM.addPass(StripUnsupportedPtrauthBundlesPass());
}

} // namespace rustc_llvm

#endif // INCLUDED_RUSTC_LLVM_PTRAUTHUTILS_H
