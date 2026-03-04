from keystone import Ks, KS_ARCH_X86, KS_MODE_64
from capstone import Cs, CS_ARCH_X86, CS_MODE_64

# Initialize Keystone and Capstone
ks = Ks(KS_ARCH_X86, KS_MODE_64)
md = Cs(CS_ARCH_X86, CS_MODE_64)

# Read the MASM assembly file
with open("code.asm", "r") as f:
    lines = f.readlines()

# Filter out comments and empty lines
asm_lines = [line.strip() for line in lines if line.strip() and not line.strip().startswith(';')]

# Assemble instructions using Keystone
assembled_instructions = []
for line in asm_lines:
    try:
        encoding, _ = ks.asm(line)
        assembled_instructions.append((line, bytes(encoding)))
    except Exception as e:
        print(f"Assembly failed for line: {line} with error: {e}")

# Disassemble instructions using Capstone
disassembled_instructions = []
for original_line, encoding in assembled_instructions:
    for instr in md.disasm(encoding, 0x1000):
        disassembled_instructions.append({
            'mnemonic': instr.mnemonic,
            'op_str': instr.op_str,
            'bytes': instr.bytes
        })

# Prepare for obfuscation (e.g., replace 'mov' with 'push' and 'pop')
obfuscated_instructions = []
for instr in disassembled_instructions:
    if instr['mnemonic'] == 'mov':
        operands = instr['op_str'].split(', ')
        if len(operands) == 2:
            src, dst = operands
            obfuscated_instructions.append(f"push {dst}")
            obfuscated_instructions.append(f"pop {src}")
        else:
            obfuscated_instructions.append(f"{instr['mnemonic']} {instr['op_str']}")
    else:
        obfuscated_instructions.append(f"{instr['mnemonic']} {instr['op_str']}")

# Write the obfuscated code to a new file
with open("obfuscated_code.asm", "w") as f:
    for line in obfuscated_instructions:
        f.write(f"{line}\n")
