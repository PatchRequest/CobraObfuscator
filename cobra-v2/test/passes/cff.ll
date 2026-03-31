; RUN: %cobra -o - --passes cff --seed 42 --emit-ll %s | FileCheck %s

; Should flatten control flow into a dispatcher
; CHECK-LABEL: define i32 @test_cff
; CHECK: cff.dispatcher
define i32 @test_cff(i32 %n) {
entry:
  %cmp = icmp sgt i32 %n, 0
  br i1 %cmp, label %then, label %else

then:
  %r1 = add i32 %n, 10
  br label %merge

else:
  %r2 = sub i32 %n, 10
  br label %merge

merge:
  %r = phi i32 [ %r1, %then ], [ %r2, %else ]
  ret i32 %r
}
