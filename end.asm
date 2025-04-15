Dispatcher1 PROC

; Check conditions for Func1

            ; group 5 → bits 35–41
            mov   rdx, rax
            shr   rdx, 35
            and   edx, 0x7F
            popcnt edx, edx
            cmp   edx, 3
            jle   Fail_Func1_08F448918B0D4956990CCC6B61A1FBE6
            

            ; group 2 → bits 14–20
            mov   rdx, rax
            shr   rdx, 14
            and   edx, 0x7F
            cmp   edx, 16
            jle   Fail_Func1_08F448918B0D4956990CCC6B61A1FBE6
            

            ; group 14 → bits 98–104 → in RBX
            mov   rdx, rbx
            shr   rdx, 98 - 64        ; = 34
            and   edx, 0x7F
            mov   ecx, edx
            not   ecx
            and   ecx, 0x7F
            popcnt ecx, ecx
            test  ecx, 1
            jnz   Fail_Func1_08F448918B0D4956990CCC6B61A1FBE6       ; odd => fail
            

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
            jnz   Fail_Func1_08F448918B0D4956990CCC6B61A1FBE6       ; odd => fail
            

jmp Call_Func1
Fail_Func1_08F448918B0D4956990CCC6B61A1FBE6:
    ; Conditions failed, continue to next check

; Check conditions for Func2

            mov   rdx, rax
            shr   rdx, 35
            and   edx, 0x7F
            popcnt edx, edx
            cmp   edx, 3
            jg    Fail_Func2_8710C2DEDC0A4FAEB311312289A92327
            

            ; group 2 → bits 14–20
            mov   rdx, rax
            shr   rdx, 14
            and   edx, 0x7F
            cmp   edx, 16
            jle   Fail_Func2_8710C2DEDC0A4FAEB311312289A92327
            

            mov   rdx, rbx
            shr   rdx, 34
            and   edx, 0x7F
            mov   ecx, edx
            not   ecx
            and   ecx, 0x7F
            popcnt ecx, ecx
            test  ecx, 1
            jz    Fail_Func2_8710C2DEDC0A4FAEB311312289A92327       ; even => fail
            

            mov   rdx, rcx
            shr   rdx, 119 - 128
            and   edx, 0x7F
            popcnt edx, edx
            test  edx, 1
            jz    Fail_Func2_8710C2DEDC0A4FAEB311312289A92327       ; even => fail
            

jmp Call_Func2
Fail_Func2_8710C2DEDC0A4FAEB311312289A92327:
    ; Conditions failed, continue to next check

Call_Func1:
    ; Call Func1 here
    jmp End_Dispatcher1

Call_Func2:
    ; Call Func2 here
    jmp End_Dispatcher1

End_Dispatcher1:
    ret
Dispatcher1 ENDP

Dispatcher2 PROC

; Check conditions for Func3

            ; group 5 → bits 35–41
            mov   rdx, rax
            shr   rdx, 35
            and   edx, 0x7F
            popcnt edx, edx
            cmp   edx, 3
            jle   Fail_Func3_F9D4F538D4E7411B9BF96C0A5590E74A
            

            mov   rdx, rax
            shr   rdx, 14
            and   edx, 0x7F
            cmp   edx, 16
            jg    Fail_Func3_F9D4F538D4E7411B9BF96C0A5590E74A
            

            ; group 14 → bits 98–104 → in RBX
            mov   rdx, rbx
            shr   rdx, 98 - 64        ; = 34
            and   edx, 0x7F
            mov   ecx, edx
            not   ecx
            and   ecx, 0x7F
            popcnt ecx, ecx
            test  ecx, 1
            jnz   Fail_Func3_F9D4F538D4E7411B9BF96C0A5590E74A       ; odd => fail
            

            mov   rdx, rcx
            shr   rdx, 119 - 128
            and   edx, 0x7F
            popcnt edx, edx
            test  edx, 1
            jz    Fail_Func3_F9D4F538D4E7411B9BF96C0A5590E74A       ; even => fail
            

jmp Call_Func3
Fail_Func3_F9D4F538D4E7411B9BF96C0A5590E74A:
    ; Conditions failed, continue to next check

; Check conditions for Func4

            mov   rdx, rax
            shr   rdx, 35
            and   edx, 0x7F
            popcnt edx, edx
            cmp   edx, 3
            jg    Fail_Func4_7466789239634456B98985CB4A049B93
            

            mov   rdx, rax
            shr   rdx, 14
            and   edx, 0x7F
            cmp   edx, 16
            jg    Fail_Func4_7466789239634456B98985CB4A049B93
            

            mov   rdx, rbx
            shr   rdx, 34
            and   edx, 0x7F
            mov   ecx, edx
            not   ecx
            and   ecx, 0x7F
            popcnt ecx, ecx
            test  ecx, 1
            jz    Fail_Func4_7466789239634456B98985CB4A049B93       ; even => fail
            

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
            jnz   Fail_Func4_7466789239634456B98985CB4A049B93       ; odd => fail
            

jmp Call_Func4
Fail_Func4_7466789239634456B98985CB4A049B93:
    ; Conditions failed, continue to next check

Call_Func3:
    ; Call Func3 here
    jmp End_Dispatcher2

Call_Func4:
    ; Call Func4 here
    jmp End_Dispatcher2

End_Dispatcher2:
    ret
Dispatcher2 ENDP


