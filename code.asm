; fib_win.asm - Windows x64 MASM-compatible
extern ExitProcess : proc

.data
    prev    dq 0
    current dq 1
    sum     dq 1
    N       dq 10

.code
main proc
    ; Load N and compute loop count (N - 1)
    mov     rax, N
    mov     rcx, qword ptr [rax]
    sub     rcx, 1
    jle     after_loop

loop_start:
    ; Call add_function(prev, current)
    mov     rax, prev
    mov     rdi, qword ptr [rax]
    mov     rax, current
    mov     rsi, qword ptr [rax]
    call    add_function

    ; Update prev = current, current = result
    mov     rbx, current
    mov     rbx, qword ptr [rbx]
    mov     rax, prev
    mov     qword ptr [rax], rbx
    mov     rbx, current
    mov     qword ptr [rbx], rax

    ; Call add_to_sum(current)
    mov     rax, current
    mov     rdi, qword ptr [rax]
    call    add_to_sum

    dec     rcx
    jnz     loop_start

after_loop:
    xor     ecx, ecx
    call    ExitProcess
main endp

add_function proc
    ; RDI + RSI => RAX
    mov     rax, rdi
    add     rax, rsi
    ret
add_function endp

add_to_sum proc
    ; sum += RDI
    mov     rax, sum
    add     qword ptr [rax], rdi
    ret
add_to_sum endp

end
