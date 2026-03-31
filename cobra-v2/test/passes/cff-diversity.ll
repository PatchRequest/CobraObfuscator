; RUN: %cobra -o - --passes cff --seed 1 --emit-ll %s | FileCheck --check-prefix=CHECK1 %s
; RUN: %cobra -o - --passes cff --seed 42 --emit-ll %s | FileCheck --check-prefix=CHECK2 %s

; Both seeds should produce valid CFF'd functions
; CHECK1-LABEL: define i32 @test_diversity
; CHECK1: cff.dispatcher
; CHECK2-LABEL: define i32 @test_diversity
; CHECK2: cff.dispatcher

define i32 @test_diversity(i32 %n) {
entry:
  %cmp = icmp sgt i32 %n, 10
  br i1 %cmp, label %big, label %small

big:
  %r1 = mul i32 %n, 2
  br label %done

small:
  %r2 = add i32 %n, 100
  br label %done

done:
  %r = phi i32 [ %r1, %big ], [ %r2, %small ]
  ret i32 %r
}
