; RUN: llc -march=neovm -o - %s 2>&1 | FileCheck %s --check-prefix=CHECK-NOOP
; XFAIL: *
;
; Placeholder test documenting expected stackify behaviour per docs/neo-n3-backend.md (Testing Plan).
;
; Once stackify rewrites are implemented, update CHECK lines to match emitted pseudos.

define i32 @add(i32 %a, i32 %b) {
entry:
  %sum = add i32 %a, %b
  ret i32 %sum
}

; CHECK-NOOP: ; XFAIL placeholder for NeoVM stackify test

