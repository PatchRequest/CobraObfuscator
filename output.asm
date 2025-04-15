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
mov eax, 98
jle dispatcher_0
pop eax
loop_start:
mov     rax, prev
mov     rdi, qword ptr [rax]
mov     rax, current
mov     rsi, qword ptr [rax]
push eax
mov eax, 52
call dispatcher_0
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
mov eax, 16
call dispatcher_1
pop eax
dec     rcx
push eax
mov eax, 12
jnz dispatcher_0
pop eax
after_loop:
xor     ecx, ecx
push eax
mov eax, 43
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
cmp eax, 43
je wxplhqgmikqr
cmp eax, 12
je yuzmbbemvdlj
cmp eax, 98
je vfvmnvndragp
cmp eax, 89
je add_function
cmp eax, 52
je mqfooyvetfon
cmp eax, 9
je add_to_sum
ret
dispatcher_1:
cmp eax, 39
je main
cmp eax, 42
je main
cmp eax, 16
je add_to_sum
ret
wxplhqgmikqr:
jmp ExitProcess
yuzmbbemvdlj:
jmp loop_start
vfvmnvndragp:
jmp after_loop
mqfooyvetfon:
jmp add_function
