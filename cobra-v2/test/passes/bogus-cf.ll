; RUN: %cobra -o - --passes bogus-cf --seed 42 --emit-ll %s | FileCheck %s

; Should add bogus conditional branches
; CHECK-LABEL: define i32 @test_bogus
; CHECK: br i1
; CHECK: bogus.
define i32 @test_bogus(i32 %a, i32 %b) {
entry:
  %r = add i32 %a, %b
  br label %exit
exit:
  %r2 = mul i32 %r, 2
  ret i32 %r2
}
