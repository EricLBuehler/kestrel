; ModuleID = 'program.ke'
source_filename = "program.ke"
target triple = "x86_64-unknown-linux-gnu"

@str = private unnamed_addr constant [50 x i8] c"Error: i32 addition overflow!\0A    program.ke:1:12\00", align 1

; Function Attrs: mustprogress nofree noinline norecurse nounwind willreturn
define i32 @main(i32 %0, i32** nocapture readnone %1) local_unnamed_addr #0 {
  %3 = tail call i1 @llvm.expect.i1.i1(i1 true, i1 false)
  br i1 %3, label %4, label %5

4:                                                ; preds = %2
  %puts = tail call i32 @puts(i8* nonnull dereferenceable(1) getelementptr inbounds ([50 x i8], [50 x i8]* @str, i64 0, i64 0))
  br label %5

5:                                                ; preds = %2, %4
  ret i32 0
}

; Function Attrs: mustprogress nofree nosync nounwind readnone willreturn
declare i1 @llvm.expect.i1.i1(i1, i1) #1

; Function Attrs: nofree nounwind
declare noundef i32 @puts(i8* nocapture noundef readonly) #2

attributes #0 = { mustprogress nofree noinline norecurse nounwind willreturn }
attributes #1 = { mustprogress nofree nosync nounwind readnone willreturn }
attributes #2 = { nofree nounwind }

!llvm.module.flags = !{!0}
!llvm.dbg.cu = !{!1}

!0 = !{i32 1, !"Debug Info Version", i32 3}
!1 = distinct !DICompileUnit(language: DW_LANG_C, file: !2, producer: "xpl", isOptimized: true, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, sdk: "xpl")
!2 = !DIFile(filename: "program.ke", directory: ".")
