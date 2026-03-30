; RUN: %cobra -o - --passes junk-insertion --seed 42 --emit-ll %s | FileCheck %s

; Should insert junk allocas and volatile load/store ops
; CHECK-LABEL: define i32 @test_junk
; CHECK: alloca
; CHECK: store
define i32 @test_junk(i32 %a) {
entry:
  %r = add i32 %a, 1
  ret i32 %r
}
