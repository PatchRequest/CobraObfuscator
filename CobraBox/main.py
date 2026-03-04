import angr
import os

proj = angr.Project('./cheat.exe', auto_load_libs=False)

cfg = proj.analyses.CFGFast()



allCalls = []
for func in cfg.kb.functions.values():
    # Iterate over basic blocks in this function
    for block in func.blocks:
        # Iterate over instructions using capstone
        for insn in block.capstone.insns:
            if insn.mnemonic == "call":
                if insn.op_str.startswith("0x"):
                    print(f"    {insn.address:x}: {insn.mnemonic} {insn.op_str}")

