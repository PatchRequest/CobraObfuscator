; RUN: %cobra -o - --passes string-encrypt --seed 42 --emit-ll %s | FileCheck %s

; CHECK-NOT: c"Hello World\00"
; CHECK: @cobra.enc.
@.str = private unnamed_addr constant [12 x i8] c"Hello World\00"

declare i32 @puts(ptr)

define i32 @main() {
  %1 = call i32 @puts(ptr @.str)
  ret i32 0
}
