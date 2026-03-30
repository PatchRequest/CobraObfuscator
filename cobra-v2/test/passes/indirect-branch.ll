; RUN: %cobra -o - --passes indirect-branch --seed 1 --emit-ll %s | FileCheck %s

; Direct calls should be replaced with indirect calls via table
; CHECK: @cobra.fptable
; CHECK: load ptr
; CHECK: call i32 %

define i32 @target(i32 %a) {
  %r = add i32 %a, 1
  ret i32 %r
}

define i32 @caller(i32 %x) {
  %r = call i32 @target(i32 %x)
  ret i32 %r
}
