; RUN: %cobra -o - --passes constant-unfold --seed 42 --emit-ll %s | FileCheck %s

; The constant 42 should be replaced with a runtime computation
; CHECK-LABEL: define i32 @test_const
; CHECK-NOT: ret i32 42
; CHECK: ret i32
define i32 @test_const() {
  ret i32 42
}

; Constants in arithmetic should be unfolded
; CHECK-LABEL: define i32 @test_add_const
; CHECK-NOT: add i32 %a, 100
define i32 @test_add_const(i32 %a) {
  %r = add i32 %a, 100
  ret i32 %r
}
