extern ExitProcess : proc
.data
prev    dq 0
current dq 1
sum     dq 1
N       dq 10
.code
main proc
mov     rax, N
mov     rcx, qword ptr [rax]
sub     rcx, 1
push eax
mov eax, 73
jle dispatcher_0
pop eax
loop_start:
mov     rax, prev
mov     rdi, qword ptr [rax]
mov     rax, current
mov     rsi, qword ptr [rax]
push eax
mov eax, 47
call dispatcher_1
pop eax
mov     rbx, current
mov     rbx, qword ptr [rbx]
mov     rax, prev
mov     qword ptr [rax], rbx
mov     rbx, current
mov     qword ptr [rbx], rax
mov     rax, current
mov     rdi, qword ptr [rax]
push eax
mov eax, 17
call dispatcher_1
pop eax
dec     rcx
push eax
mov eax, 29
jnz dispatcher_0
pop eax
after_loop:
xor     ecx, ecx
push eax
mov eax, 18
call dispatcher_0
pop eax
main endp
add_function proc
mov     rax, rdi
add     rax, rsi
ret
add_function endp
add_to_sum proc
mov     rax, sum
add     qword ptr [rax], rdi
ret
add_to_sum endp
end
dispatcher_0:
cmp eax, 18
je ExitProcess
cmp eax, 29
je xnrenjfiwovt
cmp eax, 73
je abwpsfgiqssa
cmp eax, 54
je main
cmp eax, 49
je add_function
cmp eax, 62
je add_to_sum
ret
dispatcher_1:
cmp eax, 40
je tjtvgxhmkmry
cmp eax, 47
je fwkpjvugqnje
cmp eax, 17
je add_to_sum
ret
xnrenjfiwovt:
jmp loop_start
abwpsfgiqssa:
jmp after_loop
tjtvgxhmkmry:
jmp main
fwkpjvugqnje:
jmp add_function
