; RUN: %cobra -o - --passes mba --seed 42 --emit-ll %s | FileCheck %s

; add should become MBA: (a ^ b) + 2*(a & b)
; CHECK-LABEL: define i32 @test_mba_add
; CHECK-NOT: add i32 %a, %b
; CHECK: xor
; CHECK: and
define i32 @test_mba_add(i32 %a, i32 %b) {
  %r = add i32 %a, %b
  ret i32 %r
}

; or should become MBA: (a ^ b) + (a & b)
; CHECK-LABEL: define i32 @test_mba_or
; CHECK-NOT: or i32 %a, %b
define i32 @test_mba_or(i32 %a, i32 %b) {
  %r = or i32 %a, %b
  ret i32 %r
}
