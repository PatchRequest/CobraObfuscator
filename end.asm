MainDispatcher PROC

; Check conditions for Func1
; group 22: value > 16
; Mixed group 22
xor edx, edx
mov rsi, rcx
shr rsi, 26
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 27
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 28
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 29
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 30
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 31
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 32
and rsi, 1
shl rsi, 6
or edx, esi
cmp edx, 16
jle Fail_Func1_5F1D36FD61654C23A1B05D8310CBDAC1
; group 20: popcnt > 3
; Mixed group 20
xor edx, edx
mov rsi, rcx
shr rsi, 12
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 13
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 14
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 15
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 16
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 17
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 18
and rsi, 1
shl rsi, 6
or edx, esi
popcnt edx, edx
cmp edx, 3
jle Fail_Func1_5F1D36FD61654C23A1B05D8310CBDAC1
; group 9: odd 0s
; Mixed group 9
xor edx, edx
mov rsi, rcx
shr rsi, 63
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 0
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 1
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 2
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 3
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 4
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 5
and rsi, 1
shl rsi, 6
or edx, esi
mov ecx, edx
not ecx
and ecx, 0x7F
popcnt ecx, ecx
test ecx, 1
jz Fail_Func1_5F1D36FD61654C23A1B05D8310CBDAC1
; group 6: even 1s
; Mixed group 6
xor edx, edx
mov rsi, rcx
shr rsi, 42
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 43
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 44
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 45
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 46
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 47
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 48
and rsi, 1
shl rsi, 6
or edx, esi
popcnt edx, edx
test edx, 1
jnz Fail_Func1_5F1D36FD61654C23A1B05D8310CBDAC1


; Fake contradiction: parity check fake
; Mixed group 0
xor edx, edx
mov rsi, rcx
shr rsi, 0
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 1
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 2
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 3
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 4
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 5
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 6
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_DB519EB6FD384CBDAAC7EFF78368EEF3
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func7
FakeFail_DB519EB6FD384CBDAAC7EFF78368EEF3:


; Fake contradiction: parity check fake
; Mixed group 1
xor edx, edx
mov rsi, rcx
shr rsi, 7
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 8
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 9
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 10
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 11
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 12
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 13
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_44E9F56E871B4178BD6B4429985CC7DB
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func8
FakeFail_44E9F56E871B4178BD6B4429985CC7DB:


; Fake contradiction: parity check fake
; Mixed group 2
xor edx, edx
mov rsi, rcx
shr rsi, 14
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 15
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 16
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 17
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 18
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 19
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 20
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_F40631D663D543E9BC7C486622F9C700
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func7
FakeFail_F40631D663D543E9BC7C486622F9C700:


; Fake contradiction: parity check fake
; Mixed group 3
xor edx, edx
mov rsi, rcx
shr rsi, 21
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 22
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 23
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 24
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 25
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 26
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 27
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_E1D85FBAF0024C2289A4B1960545AF56
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func6
FakeFail_E1D85FBAF0024C2289A4B1960545AF56:


; Fake contradiction: parity check fake
; Mixed group 4
xor edx, edx
mov rsi, rcx
shr rsi, 28
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 29
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 30
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 31
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 32
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 33
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 34
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_638C825575AF45918325BD58396A5ED6
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func8
FakeFail_638C825575AF45918325BD58396A5ED6:


; Fake contradiction: parity check fake
; Mixed group 5
xor edx, edx
mov rsi, rcx
shr rsi, 35
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 36
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 37
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 38
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 39
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 40
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 41
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_CF7018C2677B4AE6A895D64475AD05B0
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func6
FakeFail_CF7018C2677B4AE6A895D64475AD05B0:


; Fake contradiction: parity check fake
; Mixed group 7
xor edx, edx
mov rsi, rcx
shr rsi, 49
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 50
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 51
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 52
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 53
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 54
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 55
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_A3352DD7934F49D5B4127218FC4C1E50
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func6
FakeFail_A3352DD7934F49D5B4127218FC4C1E50:


; Fake contradiction: parity check fake
; Mixed group 8
xor edx, edx
mov rsi, rcx
shr rsi, 56
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 57
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 58
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 59
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 60
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 61
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 62
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_0C199A5D22F147A5A2B7D532361DF8EE
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func8
FakeFail_0C199A5D22F147A5A2B7D532361DF8EE:


; Fake contradiction: parity check fake
; Mixed group 10
xor edx, edx
mov rsi, rcx
shr rsi, 6
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 7
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 8
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 9
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 10
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 11
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 12
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_6ED7AE79F62E45CCB66CEA75531D9A3B
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func8
FakeFail_6ED7AE79F62E45CCB66CEA75531D9A3B:


; Fake contradiction: parity check fake
; Mixed group 11
xor edx, edx
mov rsi, rcx
shr rsi, 13
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 14
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 15
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 16
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 17
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 18
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 19
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_CAAB1690D4D64776AF130923300E8BE7
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func8
FakeFail_CAAB1690D4D64776AF130923300E8BE7:


; Fake contradiction: parity check fake
; Mixed group 12
xor edx, edx
mov rsi, rcx
shr rsi, 20
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 21
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 22
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 23
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 24
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 25
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 26
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_32ED63B841DE4F02A4FCD5292EB2407A
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func9
FakeFail_32ED63B841DE4F02A4FCD5292EB2407A:


; Fake contradiction: parity check fake
; Mixed group 13
xor edx, edx
mov rsi, rcx
shr rsi, 27
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 28
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 29
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 30
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 31
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 32
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 33
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_B7EFCB6539974B638EAE1D42AA374806
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func9
FakeFail_B7EFCB6539974B638EAE1D42AA374806:


; Fake contradiction: parity check fake
; Mixed group 14
xor edx, edx
mov rsi, rcx
shr rsi, 34
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 35
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 36
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 37
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 38
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 39
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 40
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_E95F1DEF92804583A039D28D9D87D3D6
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func5
FakeFail_E95F1DEF92804583A039D28D9D87D3D6:


; Fake contradiction: parity check fake
; Mixed group 15
xor edx, edx
mov rsi, rcx
shr rsi, 41
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 42
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 43
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 44
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 45
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 46
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 47
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_122CF009EC544F009EFAEC8C6729ADE2
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func5
FakeFail_122CF009EC544F009EFAEC8C6729ADE2:


; Fake contradiction: parity check fake
; Mixed group 16
xor edx, edx
mov rsi, rcx
shr rsi, 48
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 49
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 50
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 51
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 52
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 53
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 54
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_00C9A35C27BB4417B30BC37BC92FD7DC
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func9
FakeFail_00C9A35C27BB4417B30BC37BC92FD7DC:


; Fake contradiction: parity check fake
; Mixed group 17
xor edx, edx
mov rsi, rcx
shr rsi, 55
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 56
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 57
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 58
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 59
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 60
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 61
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_321188ACE5EC4F3A9D1832CD3B16049F
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func6
FakeFail_321188ACE5EC4F3A9D1832CD3B16049F:


; Fake contradiction: parity check fake
; Mixed group 18
xor edx, edx
mov rsi, rcx
shr rsi, 62
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 63
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 0
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 1
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 2
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 3
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 4
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_0683F2318F8642A7BDE018D007F3E86A
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func5
FakeFail_0683F2318F8642A7BDE018D007F3E86A:


; Fake contradiction: parity check fake
; Mixed group 19
xor edx, edx
mov rsi, rcx
shr rsi, 5
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 6
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 7
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 8
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 9
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 10
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 11
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_ADBD00B873584C409D8A1F77B153C6C4
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func8
FakeFail_ADBD00B873584C409D8A1F77B153C6C4:


; Fake contradiction: parity check fake
; Mixed group 21
xor edx, edx
mov rsi, rcx
shr rsi, 19
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 20
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 21
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 22
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 23
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 24
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 25
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_1DF4A0A590D84A9AA668B21E331A059E
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func5
FakeFail_1DF4A0A590D84A9AA668B21E331A059E:


; Fake contradiction: parity check fake
; Mixed group 23
xor edx, edx
mov rsi, rcx
shr rsi, 33
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 34
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 35
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 36
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 37
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 38
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 39
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_5E0E06D8B88D49EB99A0D02C6A237F40
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func7
FakeFail_5E0E06D8B88D49EB99A0D02C6A237F40:


; Fake contradiction: parity check fake
; Mixed group 24
xor edx, edx
mov rsi, rcx
shr rsi, 40
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 41
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 42
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 43
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 44
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 45
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 46
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_D3CF27ADC94444678371DA208B571E7B
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func5
FakeFail_D3CF27ADC94444678371DA208B571E7B:


; Fake contradiction: parity check fake
; Mixed group 25
xor edx, edx
mov rsi, rcx
shr rsi, 47
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 48
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 49
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 50
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 51
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 52
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 53
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_591E7E65AF4243A39E0573BF1E3D7AE6
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func5
FakeFail_591E7E65AF4243A39E0573BF1E3D7AE6:


; Fake contradiction: parity check fake
; Mixed group 26
xor edx, edx
mov rsi, rcx
shr rsi, 54
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 55
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 56
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 57
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 58
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 59
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 60
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_B7BFE933FAB143B5B1945C4D23801949
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func5
FakeFail_B7BFE933FAB143B5B1945C4D23801949:

jmp Call_Func1
Fail_Func1_5F1D36FD61654C23A1B05D8310CBDAC1:
    ; Conditions failed, continue to next check

; Check conditions for Func2
; group 17: popcnt <= 3
; Mixed group 17
xor edx, edx
mov rsi, rcx
shr rsi, 55
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 56
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 57
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 58
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 59
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 60
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 61
and rsi, 1
shl rsi, 6
or edx, esi
popcnt edx, edx
cmp edx, 3
jg Fail_Func2_FD3CD8E2CE9A44C79E4DEBA185EA1EF6
; group 11: value <= 16
; Mixed group 11
xor edx, edx
mov rsi, rcx
shr rsi, 13
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 14
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 15
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 16
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 17
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 18
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 19
and rsi, 1
shl rsi, 6
or edx, esi
cmp edx, 16
jg Fail_Func2_FD3CD8E2CE9A44C79E4DEBA185EA1EF6
; group 24: value > 16
; Mixed group 24
xor edx, edx
mov rsi, rcx
shr rsi, 40
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 41
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 42
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 43
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 44
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 45
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 46
and rsi, 1
shl rsi, 6
or edx, esi
cmp edx, 16
jle Fail_Func2_FD3CD8E2CE9A44C79E4DEBA185EA1EF6
; group 8: odd 0s
; Mixed group 8
xor edx, edx
mov rsi, rcx
shr rsi, 56
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 57
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 58
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 59
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 60
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 61
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 62
and rsi, 1
shl rsi, 6
or edx, esi
mov ecx, edx
not ecx
and ecx, 0x7F
popcnt ecx, ecx
test ecx, 1
jz Fail_Func2_FD3CD8E2CE9A44C79E4DEBA185EA1EF6


; Fake contradiction: parity check fake
; Mixed group 0
xor edx, edx
mov rsi, rcx
shr rsi, 0
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 1
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 2
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 3
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 4
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 5
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 6
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_6122CA82EC6A4B1C8A4CA0C2BC5F7635
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func9
FakeFail_6122CA82EC6A4B1C8A4CA0C2BC5F7635:


; Fake contradiction: parity check fake
; Mixed group 1
xor edx, edx
mov rsi, rcx
shr rsi, 7
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 8
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 9
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 10
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 11
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 12
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 13
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_095807C84CAB4D4087C25E3F4D6B942A
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func5
FakeFail_095807C84CAB4D4087C25E3F4D6B942A:


; Fake contradiction: parity check fake
; Mixed group 2
xor edx, edx
mov rsi, rcx
shr rsi, 14
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 15
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 16
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 17
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 18
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 19
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 20
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_3993150746B04396981C77436F5E49D7
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func6
FakeFail_3993150746B04396981C77436F5E49D7:


; Fake contradiction: parity check fake
; Mixed group 3
xor edx, edx
mov rsi, rcx
shr rsi, 21
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 22
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 23
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 24
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 25
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 26
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 27
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_94B6DF6EDE0A415EB2432F4DD6579F5E
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func6
FakeFail_94B6DF6EDE0A415EB2432F4DD6579F5E:


; Fake contradiction: parity check fake
; Mixed group 4
xor edx, edx
mov rsi, rcx
shr rsi, 28
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 29
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 30
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 31
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 32
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 33
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 34
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_7593FB4BE157482C9F8F83528D1CFAAC
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func8
FakeFail_7593FB4BE157482C9F8F83528D1CFAAC:


; Fake contradiction: parity check fake
; Mixed group 5
xor edx, edx
mov rsi, rcx
shr rsi, 35
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 36
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 37
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 38
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 39
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 40
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 41
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_BB351D3D7BEB4E6C83F29CB56073E9A3
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func9
FakeFail_BB351D3D7BEB4E6C83F29CB56073E9A3:


; Fake contradiction: parity check fake
; Mixed group 6
xor edx, edx
mov rsi, rcx
shr rsi, 42
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 43
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 44
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 45
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 46
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 47
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 48
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_EE4EF05392024D48A8E6849F9D6F0657
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func8
FakeFail_EE4EF05392024D48A8E6849F9D6F0657:


; Fake contradiction: parity check fake
; Mixed group 7
xor edx, edx
mov rsi, rcx
shr rsi, 49
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 50
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 51
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 52
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 53
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 54
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 55
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_0EB95F87EB9646DBBA12ABC8F374907F
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func5
FakeFail_0EB95F87EB9646DBBA12ABC8F374907F:


; Fake contradiction: parity check fake
; Mixed group 9
xor edx, edx
mov rsi, rcx
shr rsi, 63
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 0
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 1
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 2
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 3
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 4
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 5
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_239C5F403040415EAC8E918F81841D5E
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func6
FakeFail_239C5F403040415EAC8E918F81841D5E:


; Fake contradiction: parity check fake
; Mixed group 10
xor edx, edx
mov rsi, rcx
shr rsi, 6
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 7
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 8
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 9
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 10
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 11
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 12
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_B039A399E90043DCBDE167548B5BB1A1
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func5
FakeFail_B039A399E90043DCBDE167548B5BB1A1:


; Fake contradiction: parity check fake
; Mixed group 12
xor edx, edx
mov rsi, rcx
shr rsi, 20
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 21
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 22
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 23
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 24
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 25
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 26
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_BFB0AC5205064BC3947B3BB3B768002B
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func5
FakeFail_BFB0AC5205064BC3947B3BB3B768002B:


; Fake contradiction: parity check fake
; Mixed group 13
xor edx, edx
mov rsi, rcx
shr rsi, 27
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 28
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 29
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 30
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 31
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 32
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 33
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_C9F21BCD2B644CA6A23038C325402F2D
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func7
FakeFail_C9F21BCD2B644CA6A23038C325402F2D:


; Fake contradiction: parity check fake
; Mixed group 14
xor edx, edx
mov rsi, rcx
shr rsi, 34
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 35
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 36
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 37
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 38
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 39
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 40
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_5E85C05D0A1146F9A01A8A1DB39F9719
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func8
FakeFail_5E85C05D0A1146F9A01A8A1DB39F9719:


; Fake contradiction: parity check fake
; Mixed group 15
xor edx, edx
mov rsi, rcx
shr rsi, 41
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 42
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 43
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 44
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 45
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 46
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 47
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_6EE88798DD214AB7934F96C3D9208D6A
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func8
FakeFail_6EE88798DD214AB7934F96C3D9208D6A:


; Fake contradiction: parity check fake
; Mixed group 16
xor edx, edx
mov rsi, rcx
shr rsi, 48
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 49
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 50
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 51
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 52
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 53
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 54
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_22612ED8B0EC4A0386807E2DEAE352B4
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func9
FakeFail_22612ED8B0EC4A0386807E2DEAE352B4:


; Fake contradiction: parity check fake
; Mixed group 18
xor edx, edx
mov rsi, rcx
shr rsi, 62
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 63
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 0
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 1
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 2
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 3
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 4
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_1081C41045A04BF6BEDE59DA83783E3B
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func6
FakeFail_1081C41045A04BF6BEDE59DA83783E3B:


; Fake contradiction: parity check fake
; Mixed group 19
xor edx, edx
mov rsi, rcx
shr rsi, 5
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 6
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 7
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 8
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 9
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 10
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 11
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_324522B0687F4C828673539D247B014B
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func5
FakeFail_324522B0687F4C828673539D247B014B:


; Fake contradiction: parity check fake
; Mixed group 20
xor edx, edx
mov rsi, rcx
shr rsi, 12
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 13
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 14
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 15
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 16
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 17
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 18
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_EBF6A2CE69D74F139B8B4699229F7453
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func5
FakeFail_EBF6A2CE69D74F139B8B4699229F7453:


; Fake contradiction: parity check fake
; Mixed group 21
xor edx, edx
mov rsi, rcx
shr rsi, 19
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 20
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 21
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 22
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 23
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 24
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 25
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_46DDDF0806EC456289B856B832CE7C53
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func9
FakeFail_46DDDF0806EC456289B856B832CE7C53:


; Fake contradiction: parity check fake
; Mixed group 22
xor edx, edx
mov rsi, rcx
shr rsi, 26
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 27
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 28
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 29
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 30
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 31
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 32
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_EEA747CDCD5E4D4A97C1FC952BDBCEA1
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func9
FakeFail_EEA747CDCD5E4D4A97C1FC952BDBCEA1:


; Fake contradiction: parity check fake
; Mixed group 23
xor edx, edx
mov rsi, rcx
shr rsi, 33
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 34
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 35
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 36
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 37
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 38
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 39
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_B840A4CD889643AEBE6DA33B7FF2D3E3
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func8
FakeFail_B840A4CD889643AEBE6DA33B7FF2D3E3:


; Fake contradiction: parity check fake
; Mixed group 25
xor edx, edx
mov rsi, rcx
shr rsi, 47
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 48
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 49
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 50
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 51
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 52
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 53
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_635F4AB584B04AC6ACAD3C85EF0C48D1
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func9
FakeFail_635F4AB584B04AC6ACAD3C85EF0C48D1:


; Fake contradiction: parity check fake
; Mixed group 26
xor edx, edx
mov rsi, rcx
shr rsi, 54
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 55
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 56
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 57
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 58
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 59
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 60
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_924BEA61492142E2BCF206DA2AB32B08
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func9
FakeFail_924BEA61492142E2BCF206DA2AB32B08:

jmp Call_Func2
Fail_Func2_FD3CD8E2CE9A44C79E4DEBA185EA1EF6:
    ; Conditions failed, continue to next check

; Check conditions for Func3
; group 12: popcnt > 3
; Mixed group 12
xor edx, edx
mov rsi, rcx
shr rsi, 20
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 21
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 22
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 23
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 24
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 25
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 26
and rsi, 1
shl rsi, 6
or edx, esi
popcnt edx, edx
cmp edx, 3
jle Fail_Func3_9B5B6C33F242465896B094CCE8719E09
; group 24: popcnt <= 3
; Mixed group 24
xor edx, edx
mov rsi, rcx
shr rsi, 40
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 41
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 42
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 43
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 44
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 45
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 46
and rsi, 1
shl rsi, 6
or edx, esi
popcnt edx, edx
cmp edx, 3
jg Fail_Func3_9B5B6C33F242465896B094CCE8719E09
; group 9: even 0s
; Mixed group 9
xor edx, edx
mov rsi, rcx
shr rsi, 63
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 0
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 1
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 2
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 3
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 4
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 5
and rsi, 1
shl rsi, 6
or edx, esi
mov ecx, edx
not ecx
and ecx, 0x7F
popcnt ecx, ecx
test ecx, 1
jnz Fail_Func3_9B5B6C33F242465896B094CCE8719E09
; group 0: even 0s
; Mixed group 0
xor edx, edx
mov rsi, rcx
shr rsi, 0
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 1
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 2
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 3
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 4
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 5
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 6
and rsi, 1
shl rsi, 6
or edx, esi
mov ecx, edx
not ecx
and ecx, 0x7F
popcnt ecx, ecx
test ecx, 1
jnz Fail_Func3_9B5B6C33F242465896B094CCE8719E09


; Fake contradiction: parity check fake
; Mixed group 1
xor edx, edx
mov rsi, rcx
shr rsi, 7
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 8
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 9
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 10
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 11
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 12
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 13
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_E5C21923C2564217BF91C4389E034C1B
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func7
FakeFail_E5C21923C2564217BF91C4389E034C1B:


; Fake contradiction: parity check fake
; Mixed group 2
xor edx, edx
mov rsi, rcx
shr rsi, 14
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 15
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 16
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 17
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 18
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 19
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 20
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_D6B29EFBD9E846749C4747123B409BB3
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func8
FakeFail_D6B29EFBD9E846749C4747123B409BB3:


; Fake contradiction: parity check fake
; Mixed group 3
xor edx, edx
mov rsi, rcx
shr rsi, 21
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 22
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 23
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 24
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 25
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 26
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 27
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_D750CEDF7A1C44F0B11B1B6A4C1E93EB
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func8
FakeFail_D750CEDF7A1C44F0B11B1B6A4C1E93EB:


; Fake contradiction: parity check fake
; Mixed group 4
xor edx, edx
mov rsi, rcx
shr rsi, 28
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 29
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 30
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 31
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 32
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 33
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 34
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_BA2ED8F9FB9D4146A42DFCE277220215
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func7
FakeFail_BA2ED8F9FB9D4146A42DFCE277220215:


; Fake contradiction: parity check fake
; Mixed group 5
xor edx, edx
mov rsi, rcx
shr rsi, 35
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 36
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 37
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 38
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 39
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 40
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 41
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_240A8E01F4D444E3A5D338AD9A094F06
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func7
FakeFail_240A8E01F4D444E3A5D338AD9A094F06:


; Fake contradiction: parity check fake
; Mixed group 6
xor edx, edx
mov rsi, rcx
shr rsi, 42
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 43
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 44
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 45
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 46
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 47
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 48
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_3BF374466C1D4E748850038A6A6BAC7A
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func7
FakeFail_3BF374466C1D4E748850038A6A6BAC7A:


; Fake contradiction: parity check fake
; Mixed group 7
xor edx, edx
mov rsi, rcx
shr rsi, 49
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 50
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 51
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 52
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 53
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 54
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 55
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_29FC14533E834029A580D220584E5BF3
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func6
FakeFail_29FC14533E834029A580D220584E5BF3:


; Fake contradiction: parity check fake
; Mixed group 8
xor edx, edx
mov rsi, rcx
shr rsi, 56
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 57
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 58
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 59
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 60
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 61
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 62
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_83C972511B024167B93A1D99365C76E5
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func8
FakeFail_83C972511B024167B93A1D99365C76E5:


; Fake contradiction: parity check fake
; Mixed group 10
xor edx, edx
mov rsi, rcx
shr rsi, 6
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 7
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 8
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 9
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 10
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 11
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 12
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_561215854C2F465F9E8B59C5ACEABA96
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func5
FakeFail_561215854C2F465F9E8B59C5ACEABA96:


; Fake contradiction: parity check fake
; Mixed group 11
xor edx, edx
mov rsi, rcx
shr rsi, 13
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 14
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 15
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 16
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 17
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 18
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 19
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_319BA1CBEA734CCD9B90EBC0FB6D644C
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func7
FakeFail_319BA1CBEA734CCD9B90EBC0FB6D644C:


; Fake contradiction: parity check fake
; Mixed group 13
xor edx, edx
mov rsi, rcx
shr rsi, 27
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 28
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 29
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 30
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 31
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 32
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 33
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_6BFD898739FE4E7EBD7A6B4BE7432AAE
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func8
FakeFail_6BFD898739FE4E7EBD7A6B4BE7432AAE:


; Fake contradiction: parity check fake
; Mixed group 14
xor edx, edx
mov rsi, rcx
shr rsi, 34
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 35
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 36
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 37
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 38
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 39
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 40
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_54311951C6364AABBB064FDB32AB4D53
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func8
FakeFail_54311951C6364AABBB064FDB32AB4D53:


; Fake contradiction: parity check fake
; Mixed group 15
xor edx, edx
mov rsi, rcx
shr rsi, 41
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 42
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 43
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 44
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 45
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 46
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 47
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_D2AB5232932E4ED987190C4CAFC56B30
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func6
FakeFail_D2AB5232932E4ED987190C4CAFC56B30:


; Fake contradiction: parity check fake
; Mixed group 16
xor edx, edx
mov rsi, rcx
shr rsi, 48
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 49
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 50
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 51
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 52
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 53
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 54
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_D3236317C9C04DBDBBCCED68FFCC119B
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func8
FakeFail_D3236317C9C04DBDBBCCED68FFCC119B:


; Fake contradiction: parity check fake
; Mixed group 17
xor edx, edx
mov rsi, rcx
shr rsi, 55
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 56
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 57
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 58
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 59
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 60
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 61
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_C209EC5F3EB346E1862836F61305AB88
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func9
FakeFail_C209EC5F3EB346E1862836F61305AB88:


; Fake contradiction: parity check fake
; Mixed group 18
xor edx, edx
mov rsi, rcx
shr rsi, 62
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 63
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 0
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 1
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 2
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 3
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 4
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_D87EA79B745A4E6E93A4332E6F464B65
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func8
FakeFail_D87EA79B745A4E6E93A4332E6F464B65:


; Fake contradiction: parity check fake
; Mixed group 19
xor edx, edx
mov rsi, rcx
shr rsi, 5
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 6
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 7
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 8
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 9
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 10
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 11
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_F39845E0E0DA41BE85B33B223621B86A
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func8
FakeFail_F39845E0E0DA41BE85B33B223621B86A:


; Fake contradiction: parity check fake
; Mixed group 20
xor edx, edx
mov rsi, rcx
shr rsi, 12
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 13
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 14
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 15
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 16
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 17
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 18
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_51FF9CAA628A40ED92C86A4C3E62E8F5
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func8
FakeFail_51FF9CAA628A40ED92C86A4C3E62E8F5:


; Fake contradiction: parity check fake
; Mixed group 21
xor edx, edx
mov rsi, rcx
shr rsi, 19
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 20
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 21
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 22
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 23
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 24
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 25
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_19BA00B15F5B464192569EF1E790FA77
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func7
FakeFail_19BA00B15F5B464192569EF1E790FA77:


; Fake contradiction: parity check fake
; Mixed group 22
xor edx, edx
mov rsi, rcx
shr rsi, 26
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 27
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 28
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 29
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 30
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 31
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 32
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_DAEA42B7974D459188654F23D04A0A20
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func9
FakeFail_DAEA42B7974D459188654F23D04A0A20:


; Fake contradiction: parity check fake
; Mixed group 23
xor edx, edx
mov rsi, rcx
shr rsi, 33
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 34
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 35
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 36
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 37
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 38
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 39
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_7B210638C8DD413DA8F94C80A613F862
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func5
FakeFail_7B210638C8DD413DA8F94C80A613F862:


; Fake contradiction: parity check fake
; Mixed group 25
xor edx, edx
mov rsi, rcx
shr rsi, 47
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 48
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 49
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 50
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 51
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 52
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 53
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_BD4076D9F4834EB4B2BEDAD48729956A
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func7
FakeFail_BD4076D9F4834EB4B2BEDAD48729956A:


; Fake contradiction: parity check fake
; Mixed group 26
xor edx, edx
mov rsi, rcx
shr rsi, 54
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 55
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 56
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 57
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 58
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 59
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 60
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_5CCA8AB68623432EB1B0C8D5C45407D8
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func8
FakeFail_5CCA8AB68623432EB1B0C8D5C45407D8:

jmp Call_Func3
Fail_Func3_9B5B6C33F242465896B094CCE8719E09:
    ; Conditions failed, continue to next check

; Check conditions for Func4
; group 22: value > 16
; Mixed group 22
xor edx, edx
mov rsi, rcx
shr rsi, 26
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 27
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 28
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 29
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 30
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 31
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 32
and rsi, 1
shl rsi, 6
or edx, esi
cmp edx, 16
jle Fail_Func4_130A57766FCE470398CDA8A6EC1579EF
; group 0: popcnt > 3
; Mixed group 0
xor edx, edx
mov rsi, rcx
shr rsi, 0
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 1
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 2
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 3
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 4
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 5
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 6
and rsi, 1
shl rsi, 6
or edx, esi
popcnt edx, edx
cmp edx, 3
jle Fail_Func4_130A57766FCE470398CDA8A6EC1579EF
; group 16: even 1s
; Mixed group 16
xor edx, edx
mov rsi, rcx
shr rsi, 48
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 49
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 50
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 51
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 52
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 53
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 54
and rsi, 1
shl rsi, 6
or edx, esi
popcnt edx, edx
test edx, 1
jnz Fail_Func4_130A57766FCE470398CDA8A6EC1579EF
; group 21: popcnt > 3
; Mixed group 21
xor edx, edx
mov rsi, rcx
shr rsi, 19
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 20
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 21
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 22
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 23
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 24
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 25
and rsi, 1
shl rsi, 6
or edx, esi
popcnt edx, edx
cmp edx, 3
jle Fail_Func4_130A57766FCE470398CDA8A6EC1579EF


; Fake contradiction: parity check fake
; Mixed group 1
xor edx, edx
mov rsi, rcx
shr rsi, 7
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 8
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 9
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 10
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 11
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 12
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 13
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_9E6179AD8BE9468CAC8A3817B478BD79
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func7
FakeFail_9E6179AD8BE9468CAC8A3817B478BD79:


; Fake contradiction: parity check fake
; Mixed group 2
xor edx, edx
mov rsi, rcx
shr rsi, 14
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 15
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 16
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 17
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 18
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 19
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 20
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_8A369AC33A9A455891B9820DA344B16E
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func7
FakeFail_8A369AC33A9A455891B9820DA344B16E:


; Fake contradiction: parity check fake
; Mixed group 3
xor edx, edx
mov rsi, rcx
shr rsi, 21
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 22
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 23
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 24
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 25
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 26
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 27
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_197BC0E42606467684BAB411AC021F0B
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func7
FakeFail_197BC0E42606467684BAB411AC021F0B:


; Fake contradiction: parity check fake
; Mixed group 4
xor edx, edx
mov rsi, rcx
shr rsi, 28
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 29
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 30
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 31
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 32
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 33
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 34
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_002DED4DF28A42D58AE94B0C5B5E1507
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func9
FakeFail_002DED4DF28A42D58AE94B0C5B5E1507:


; Fake contradiction: parity check fake
; Mixed group 5
xor edx, edx
mov rsi, rcx
shr rsi, 35
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 36
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 37
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 38
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 39
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 40
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 41
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_96A49973BC4A4FBFB79243DAFCE33C79
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func7
FakeFail_96A49973BC4A4FBFB79243DAFCE33C79:


; Fake contradiction: parity check fake
; Mixed group 6
xor edx, edx
mov rsi, rcx
shr rsi, 42
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 43
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 44
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 45
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 46
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 47
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 48
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_5BBD5B722D5E4D2FA226BDC3540F8B3D
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func6
FakeFail_5BBD5B722D5E4D2FA226BDC3540F8B3D:


; Fake contradiction: parity check fake
; Mixed group 7
xor edx, edx
mov rsi, rcx
shr rsi, 49
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 50
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 51
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 52
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 53
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 54
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 55
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_FB92742E5F8C44E7AAF647D75181B230
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func8
FakeFail_FB92742E5F8C44E7AAF647D75181B230:


; Fake contradiction: parity check fake
; Mixed group 8
xor edx, edx
mov rsi, rcx
shr rsi, 56
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 57
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 58
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 59
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 60
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 61
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 62
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_94121053F7B84092A8AF1185D02C35CD
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func5
FakeFail_94121053F7B84092A8AF1185D02C35CD:


; Fake contradiction: parity check fake
; Mixed group 9
xor edx, edx
mov rsi, rcx
shr rsi, 63
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 0
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 1
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 2
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 3
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 4
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 5
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_CF7F32DBAC2A4ECA8EB58C0B2A8B483F
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func7
FakeFail_CF7F32DBAC2A4ECA8EB58C0B2A8B483F:


; Fake contradiction: parity check fake
; Mixed group 10
xor edx, edx
mov rsi, rcx
shr rsi, 6
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 7
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 8
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 9
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 10
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 11
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 12
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_B346E6063391416B82D520B5EC98D599
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func9
FakeFail_B346E6063391416B82D520B5EC98D599:


; Fake contradiction: parity check fake
; Mixed group 11
xor edx, edx
mov rsi, rcx
shr rsi, 13
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 14
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 15
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 16
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 17
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 18
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 19
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_F99187173C1F477A8CCC334C84891832
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func5
FakeFail_F99187173C1F477A8CCC334C84891832:


; Fake contradiction: parity check fake
; Mixed group 12
xor edx, edx
mov rsi, rcx
shr rsi, 20
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 21
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 22
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 23
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 24
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 25
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 26
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_E2C357BF104149B4BA89975A4F4A4098
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func5
FakeFail_E2C357BF104149B4BA89975A4F4A4098:


; Fake contradiction: parity check fake
; Mixed group 13
xor edx, edx
mov rsi, rcx
shr rsi, 27
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 28
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 29
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 30
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 31
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 32
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 33
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_1A87264B65FC40A487DF3AD3D15C0AF6
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func6
FakeFail_1A87264B65FC40A487DF3AD3D15C0AF6:


; Fake contradiction: parity check fake
; Mixed group 14
xor edx, edx
mov rsi, rcx
shr rsi, 34
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 35
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 36
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 37
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 38
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 39
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 40
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_B3D2D4DBF4D74D688F330621B3A0651A
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func7
FakeFail_B3D2D4DBF4D74D688F330621B3A0651A:


; Fake contradiction: parity check fake
; Mixed group 15
xor edx, edx
mov rsi, rcx
shr rsi, 41
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 42
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 43
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 44
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 45
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 46
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 47
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_43EF5DD38D654EDEB5F959B6FA66D584
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func9
FakeFail_43EF5DD38D654EDEB5F959B6FA66D584:


; Fake contradiction: parity check fake
; Mixed group 17
xor edx, edx
mov rsi, rcx
shr rsi, 55
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 56
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 57
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 58
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 59
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 60
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 61
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_D1BE3F3072E24E31964686BD403BAC2F
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func9
FakeFail_D1BE3F3072E24E31964686BD403BAC2F:


; Fake contradiction: parity check fake
; Mixed group 18
xor edx, edx
mov rsi, rcx
shr rsi, 62
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 63
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 0
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 1
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 2
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 3
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 4
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_3869D74719004FA38A678AD31EE63816
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func7
FakeFail_3869D74719004FA38A678AD31EE63816:


; Fake contradiction: parity check fake
; Mixed group 19
xor edx, edx
mov rsi, rcx
shr rsi, 5
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 6
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 7
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 8
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 9
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 10
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 11
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_80C5829CF3D549878DC6B2E7CF03E75E
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func5
FakeFail_80C5829CF3D549878DC6B2E7CF03E75E:


; Fake contradiction: parity check fake
; Mixed group 20
xor edx, edx
mov rsi, rcx
shr rsi, 12
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 13
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 14
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 15
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 16
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 17
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 18
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_BD00659B0D154CA186CC161DDE7A76CD
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func5
FakeFail_BD00659B0D154CA186CC161DDE7A76CD:


; Fake contradiction: parity check fake
; Mixed group 23
xor edx, edx
mov rsi, rcx
shr rsi, 33
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 34
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 35
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 36
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 37
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 38
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 39
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_B2BDC8A1DE514AA7B5B8603E9B3EFF40
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func6
FakeFail_B2BDC8A1DE514AA7B5B8603E9B3EFF40:


; Fake contradiction: parity check fake
; Mixed group 24
xor edx, edx
mov rsi, rcx
shr rsi, 40
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 41
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 42
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 43
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 44
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 45
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 46
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_A5B69C5535EB44598BE8AA2707595988
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func7
FakeFail_A5B69C5535EB44598BE8AA2707595988:


; Fake contradiction: parity check fake
; Mixed group 25
xor edx, edx
mov rsi, rcx
shr rsi, 47
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 48
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 49
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 50
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 51
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 52
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 53
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_45C270CE436546CAA4C51D284617E267
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func7
FakeFail_45C270CE436546CAA4C51D284617E267:


; Fake contradiction: parity check fake
; Mixed group 26
xor edx, edx
mov rsi, rcx
shr rsi, 54
and rsi, 1
shl rsi, 0
or edx, esi
mov rsi, rax
shr rsi, 55
and rsi, 1
shl rsi, 1
or edx, esi
mov rsi, rbx
shr rsi, 56
and rsi, 1
shl rsi, 2
or edx, esi
mov rsi, rax
shr rsi, 57
and rsi, 1
shl rsi, 3
or edx, esi
mov rsi, rcx
shr rsi, 58
and rsi, 1
shl rsi, 4
or edx, esi
mov rsi, rbx
shr rsi, 59
and rsi, 1
shl rsi, 5
or edx, esi
mov rsi, rax
shr rsi, 60
and rsi, 1
shl rsi, 6
or edx, esi
mov   ecx, edx
not   ecx
and   ecx, 0x7F
popcnt edx, edx
popcnt ecx, ecx
cmp   edx, ecx
jne   FakeFail_B948894069554DF888D0AC8A5AA862B8
mov   eax, edx
xor   eax, 0x55
and   eax, 0x7F
jmp   Call_Func9
FakeFail_B948894069554DF888D0AC8A5AA862B8:

jmp Call_Func4
Fail_Func4_130A57766FCE470398CDA8A6EC1579EF:
    ; Conditions failed, continue to next check

Call_Func1:
    ; Call Func1 here
    jmp End_MainDispatcher

Call_Func2:
    ; Call Func2 here
    jmp End_MainDispatcher

Call_Func3:
    ; Call Func3 here
    jmp End_MainDispatcher

Call_Func4:
    ; Call Func4 here
    jmp End_MainDispatcher

End_MainDispatcher:
    ret
MainDispatcher ENDP


