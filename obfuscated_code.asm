push qword ptr [rax]
pop rcx
sub rcx, 1
push qword ptr [rax]
pop rdi
push qword ptr [rax]
pop rsi
push qword ptr [rbx]
pop rbx
push rbx
pop qword ptr [rax]
push rax
pop qword ptr [rbx]
push qword ptr [rax]
pop rdi
dec rcx
xor ecx, ecx
push rdi
pop rax
add rax, rsi
ret 
add qword ptr [rax], rdi
ret 
