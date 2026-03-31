; RUN: %cobra -o - --passes symbol-strip --seed 42 --emit-ll %s | FileCheck %s

; Internal function names should be replaced
; CHECK-NOT: define {{.*}} @my_helper(
; CHECK-NOT: define {{.*}} @compute_value(
; main should be preserved
; CHECK: define {{.*}} @main

define internal i32 @my_helper(i32 %x) {
  %r = add i32 %x, 1
  ret i32 %r
}

define internal i32 @compute_value(i32 %a, i32 %b) {
  %r = call i32 @my_helper(i32 %a)
  %s = add i32 %r, %b
  ret i32 %s
}

define i32 @main() {
  %r = call i32 @compute_value(i32 5, i32 3)
  ret i32 %r
}
