import random
import string
import uuid

TOTAL_GROUPS = 27
MIX_PATTERN = ['rcx', 'rax', 'rbx', 'rax', 'rcx', 'rbx', 'rax']  # 7-bit mix pattern

def get_mixed_group_code(group):
    bit_base = group * 7
    code_lines = [f"; Mixed group {group}\nxor edx, edx"]
    for i in range(7):
        bit = bit_base + i
        reg = MIX_PATTERN[i % len(MIX_PATTERN)]
        reg_index = bit // 64
        bit_offset = bit % 64
        reg_name = ['rax', 'rbx', 'rcx'][reg_index % 3] if reg == 'auto' else reg
        code_lines += [
            f"mov rsi, {reg_name}",
            f"shr rsi, {bit_offset}",
            f"and rsi, 1",
            f"shl rsi, {i}",
            f"or edx, esi"
        ]
    return "\n".join(code_lines)

def gen_condition_group_count_gt(group, threshold):
    mix_code = get_mixed_group_code(group)
    return (
        f"; group {group}: popcnt > {threshold}\n"
        f"{mix_code}\n"
        f"popcnt edx, edx\n"
        f"cmp edx, {threshold}\n"
        f"jle ${{fail_label}}"
    ), (
        f"; group {group}: popcnt <= {threshold}\n"
        f"{mix_code}\n"
        f"popcnt edx, edx\n"
        f"cmp edx, {threshold}\n"
        f"jg ${{fail_label}}"
    )

def gen_condition_group_value_gt(group, value):
    mix_code = get_mixed_group_code(group)
    return (
        f"; group {group}: value > {value}\n"
        f"{mix_code}\n"
        f"cmp edx, {value}\n"
        f"jle ${{fail_label}}"
    ), (
        f"; group {group}: value <= {value}\n"
        f"{mix_code}\n"
        f"cmp edx, {value}\n"
        f"jg ${{fail_label}}"
    )

def gen_condition_even_zero_count(group):
    mix_code = get_mixed_group_code(group)
    return (
        f"; group {group}: even 0s\n"
        f"{mix_code}\n"
        f"mov ecx, edx\n"
        f"not ecx\n"
        f"and ecx, 0x7F\n"
        f"popcnt ecx, ecx\n"
        f"test ecx, 1\n"
        f"jnz ${{fail_label}}"
    ), (
        f"; group {group}: odd 0s\n"
        f"{mix_code}\n"
        f"mov ecx, edx\n"
        f"not ecx\n"
        f"and ecx, 0x7F\n"
        f"popcnt ecx, ecx\n"
        f"test ecx, 1\n"
        f"jz ${{fail_label}}"
    )

def gen_condition_even_ones(group):
    mix_code = get_mixed_group_code(group)
    return (
        f"; group {group}: even 1s\n"
        f"{mix_code}\n"
        f"popcnt edx, edx\n"
        f"test edx, 1\n"
        f"jnz ${{fail_label}}"
    ), (
        f"; group {group}: odd 1s\n"
        f"{mix_code}\n"
        f"popcnt edx, edx\n"
        f"test edx, 1\n"
        f"jz ${{fail_label}}"
    )

condition_funcs = [
    lambda g: gen_condition_group_count_gt(g, 3),
    lambda g: gen_condition_group_value_gt(g, 16),
    lambda g: gen_condition_even_zero_count(g),
    lambda g: gen_condition_even_ones(g)
]

def build_dynamic_conditions(fail_label, group_list):
    code = ""
    for g in group_list:
        cond_gen = random.choice(condition_funcs)
        pos, neg = cond_gen(g)
        tmpl = string.Template(random.choice([pos, neg]))
        code += tmpl.substitute(fail_label=fail_label) + "\n"
    return code

def generate_contradiction_check(group):
    mix_code = get_mixed_group_code(group)
    fail_label = f"FakeFail_{uuid.uuid4().hex.upper()}"
    return f"""
; Fake contradiction: parity check fake
{mix_code}
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   {fail_label}
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func{random.randint(5, 9)}
{fail_label}:
"""
check_block_template = string.Template(r'''
; Check conditions for $func_name
$condition_code
$fake_checks
jmp Call_$func_name
$fail_label:
    ; Conditions failed, continue to next check
''')

if __name__ == "__main__":



    assembly_code = ""
    for dispatcher_name, func_list in dispatcher.items():
        assembly_code += f"{dispatcher_name} PROC\n"
        for func in func_list:
            fail_label = f"Fail_{func['name']}_{uuid.uuid4().hex.upper()}"
            real_groups = random.sample(range(TOTAL_GROUPS), 4)
            unused_groups = sorted(set(range(TOTAL_GROUPS)) - set(real_groups))

            condition_code = build_dynamic_conditions(fail_label, real_groups)
            fake_checks = "\n".join(generate_contradiction_check(g) for g in unused_groups)

            block = check_block_template.substitute(
                func_name=func["name"],
                condition_code=condition_code,
                fake_checks=fake_checks,
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
