import string
import uuid

# Dispatcher buckets (could be randomized)
dispatchers = {
    "Dispatcher1": [
        {"name": "Func1", "pattern": [0, 0, 0, 0]},
        {"name": "Func2", "pattern": [1, 0, 1, 1]},
    ],
    "Dispatcher2": [
        {"name": "Func3", "pattern": [0, 1, 0, 1]},
        {"name": "Func4", "pattern": [1, 1, 1, 0]},
    ]
}

pairs = [
    {
        "mode": "normal",
        "group": 5,
        "conds": (
            """
            ; group 5 → bits 35–41
            mov   rdx, rax
            shr   rdx, 35
            and   edx, 0x7F
            popcnt edx, edx
            cmp   edx, 3
            jle   ${fail_label}
            """,
            """
            mov   rdx, rax
            shr   rdx, 35
            and   edx, 0x7F
            popcnt edx, edx
            cmp   edx, 3
            jg    ${fail_label}
            """
        )
    },
    {
        "mode": "normal",
        "group": 2,
        "conds": (
            """
            ; group 2 → bits 14–20
            mov   rdx, rax
            shr   rdx, 14
            and   edx, 0x7F
            cmp   edx, 16
            jle   ${fail_label}
            """,
            """
            mov   rdx, rax
            shr   rdx, 14
            and   edx, 0x7F
            cmp   edx, 16
            jg    ${fail_label}
            """
        )
    },
    {
        "mode": "normal",
        "group": 14,
        "conds": (
            """
            ; group 14 → bits 98–104 → in RBX
            mov   rdx, rbx
            shr   rdx, 98 - 64        ; = 34
            and   edx, 0x7F
            mov   ecx, edx
            not   ecx
            and   ecx, 0x7F
            popcnt ecx, ecx
            test  ecx, 1
            jnz   ${fail_label}       ; odd => fail
            """,
            """
            mov   rdx, rbx
            shr   rdx, 34
            and   edx, 0x7F
            mov   ecx, edx
            not   ecx
            and   ecx, 0x7F
            popcnt ecx, ecx
            test  ecx, 1
            jz    ${fail_label}       ; even => fail
            """
        )
    },
    {
        "mode": "normal",
        "group": 17,
        "conds": (
            """
            ; group 17 → bits 119–125 → in RCX
            mov   rdx, rcx
            shr   rdx, 119 - 128      ; shift negative? adjust from rcx
            ; actually: 119-128 = -9, so shift left 9
            shl   rdx, 9
            shr   rdx, 9              ; isolate lower bits
            shr   rdx, 119 - 128
            and   edx, 0x7F
            popcnt edx, edx
            test  edx, 1
            jnz   ${fail_label}       ; odd => fail
            """,
            """
            mov   rdx, rcx
            shr   rdx, 119 - 128
            and   edx, 0x7F
            popcnt edx, edx
            test  edx, 1
            jz    ${fail_label}       ; even => fail
            """
        )
    }
]

def build_condition_code(pattern, fail_label):
    code = ""
    for i, bit in enumerate(pattern):
        pair = pairs[i]
        conds = pair["conds"]
        mode = pair["mode"]
        if mode == "inverse":
            bit = 1 - bit
        tmpl = string.Template(conds[bit])
        code += tmpl.substitute(fail_label=fail_label) + "\n"
    return code

check_block_template = string.Template(r'''
; Check conditions for $func_name
$condition_code
jmp Call_$func_name
$fail_label:
    ; Conditions failed, continue to next check
''')

assembly_code = ""

for dispatcher_name, func_list in dispatchers.items():
    assembly_code += f"{dispatcher_name} PROC\n"

    for func in func_list:
        fail_label = f"Fail_{func['name']}_{uuid.uuid4().hex.upper()}"
        condition_code = build_condition_code(func["pattern"], fail_label)
        block = check_block_template.substitute(
            func_name=func["name"],
            condition_code=condition_code,
            fail_label=fail_label
        )
        assembly_code += block

    for func in func_list:
        assembly_code += f"""
Call_{func['name']}:
    ; Call {func['name']} here
    jmp End_{dispatcher_name}
"""

    assembly_code += f"\nEnd_{dispatcher_name}:\n    ret\n{dispatcher_name} ENDP\n\n"

print(assembly_code)
