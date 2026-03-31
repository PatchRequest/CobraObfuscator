; RUN: %cobra -o - --passes anti-tamper --seed 42 --emit-ll %s | FileCheck %s

; Should insert tamper check at function entry
; CHECK-LABEL: define i32 @protected_fn
; CHECK: cobra.tamper
; CHECK: call void @abort
define i32 @protected_fn(i32 %a) {
entry:
  %r = add i32 %a, 10
  br label %exit
exit:
  %r2 = mul i32 %r, 2
  ret i32 %r2
}
