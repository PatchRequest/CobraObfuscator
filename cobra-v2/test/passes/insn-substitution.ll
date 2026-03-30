; RUN: %cobra -o - --passes insn-substitution --seed 42 --emit-ll %s | FileCheck %s

; The add should be replaced with sub-of-neg or equivalent
; CHECK-LABEL: define i32 @test_add
; CHECK-NOT: add i32 %a, %b
; CHECK: sub i32
define i32 @test_add(i32 %a, i32 %b) {
  %r = add i32 %a, %b
  ret i32 %r
}

; The xor should be replaced with and/or chain
; CHECK-LABEL: define i32 @test_xor
; CHECK-NOT: xor i32 %a, %b
define i32 @test_xor(i32 %a, i32 %b) {
  %r = xor i32 %a, %b
  ret i32 %r
}

; The sub should be replaced
; CHECK-LABEL: define i32 @test_sub
; CHECK-NOT: sub i32 %a, %b
define i32 @test_sub(i32 %a, i32 %b) {
  %r = sub i32 %a, %b
  ret i32 %r
}
