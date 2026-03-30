; RUN: %cobra -o - --passes func-merge-split --seed 1 --emit-ll %s | FileCheck %s

; Two functions with same signature should get merged
; CHECK: cobra.merged
; CHECK: switch i32

define i32 @foo(i32 %a) {
  %r = add i32 %a, 10
  ret i32 %r
}

define i32 @bar(i32 %a) {
  %r = mul i32 %a, 2
  ret i32 %r
}

define i32 @main() {
  %a = call i32 @foo(i32 5)
  %b = call i32 @bar(i32 3)
  %r = add i32 %a, %b
  ret i32 %r
}
