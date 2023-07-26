; ModuleID = 'program.ke'
source_filename = "program.ke"
target triple = "x86_64-unknown-linux-gnu"

; Function Attrs: mustprogress nofree noinline norecurse nosync nounwind readnone sanitize_address sanitize_memory sanitize_thread willreturn
define i32 @main(i32 %0, i32** nocapture readnone %1) local_unnamed_addr #0 {
  ret i32 0
}

attributes #0 = { mustprogress nofree noinline norecurse nosync nounwind readnone sanitize_address sanitize_memory sanitize_thread willreturn }

!llvm.module.flags = !{!0}
!llvm.dbg.cu = !{!1}

!0 = !{i32 1, !"Debug Info Version", i32 3}
!1 = distinct !DICompileUnit(language: DW_LANG_C, file: !2, producer: "xpl", isOptimized: true, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, sdk: "xpl")
!2 = !DIFile(filename: "program.ke", directory: ".")
