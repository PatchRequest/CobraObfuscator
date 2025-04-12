push eax           ; save original EAX
mov eax, 1
call dispatcherOne
pop eax            ; restore original EAX after returning

dispatcherOne:
    cmp eax, 1
    je  real_target_one
    cmp eax, 2
    je  real_target_two
    ret             ; default fallback

dispatcherTwo:
    cmp eax, 1
    je  real_target_three
    cmp eax, 2
    je  proxyFunctionOne
    ret             ; default fallback

proxyFunctionOne:
    jmp real_target_four


