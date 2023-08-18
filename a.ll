; ModuleID = 'program.ke'
source_filename = "program.ke"
target triple = "x86_64-unknown-linux-gnu"

; Function Attrs: noinline norecurse nounwind optnone willreturn
define i32 @main(i32 %0, i32** %1) local_unnamed_addr #0 {
  %3 = alloca i32, align 4
  store i32 123, i32* %3, align 4
  %4 = alloca i32*, align 8
  store i32* %3, i32** %4, align 8
  %5 = load i32*, i32** %4, align 8
  %6 = load i32, i32* %5, align 4
  %7 = alloca i32, align 4
  store i32 %6, i32* %7, align 4
  br label %8

8:                                                ; preds = %2
  %9 = alloca i32, align 4
  store i32 100, i32* %9, align 4
  br label %10

10:                                               ; preds = %8
  ret i32 0
}

attributes #0 = { noinline norecurse nounwind optnone willreturn }

!llvm.module.flags = !{!0}
!llvm.dbg.cu = !{!1}

!0 = !{i32 1, !"Debug Info Version", i32 3}
!1 = distinct !DICompileUnit(language: DW_LANG_C, file: !2, producer: "kestrel", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, sdk: "kestrel")
!2 = !DIFile(filename: "program.ke", directory: ".")
