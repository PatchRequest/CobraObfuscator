; RUN: %cobra -o - --passes dead-code --seed 42 --emit-ll %s | FileCheck %s

; Should insert opaque predicate branches into non-entry blocks
; CHECK-LABEL: define i32 @test_dead
; CHECK: icmp
; CHECK: br i1
define i32 @test_dead(i32 %a) {
entry:
  %cmp = icmp sgt i32 %a, 0
  br i1 %cmp, label %positive, label %nonpositive

positive:
  %r1 = add i32 %a, 1
  %r2 = mul i32 %r1, 2
  br label %exit

nonpositive:
  %r3 = sub i32 0, %a
  %r4 = add i32 %r3, 1
  br label %exit

exit:
  %result = phi i32 [ %r2, %positive ], [ %r4, %nonpositive ]
  ret i32 %result
}
