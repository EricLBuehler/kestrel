; ModuleID = 'program.ke'
source_filename = "program.ke"
target triple = "x86_64-unknown-linux-gnu"

@0 = private constant [51 x i8] c"Error: i32 addition overflow!\0A    program.ke:1:12\0A\00"

; Function Attrs: nofree nounwind
declare noundef i32 @printf(i8* nocapture noundef readonly) local_unnamed_addr #0

; Function Attrs: noinline norecurse nounwind optnone willreturn
define i32 @main(i32 %0, i32** %1) local_unnamed_addr #1 {
  %3 = call { i32, i1 } @llvm.sadd.with.overflow.i32.i32(i32 1, i32 2)
  %4 = extractvalue { i32, i1 } %3, 0
  %5 = extractvalue { i32, i1 } %3, 1
  %6 = call i1 @llvm.expect.i1.i1(i1 %5, i1 false)
  br i1 %6, label %7, label %9

7:                                                ; preds = %2
  %8 = call i32 @printf(i8* getelementptr inbounds ([51 x i8], [51 x i8]* @0, i32 0, i32 0))
  br label %10

9:                                                ; preds = %2
  br label %10

10:                                               ; preds = %9, %7
  %11 = phi i32 [ %4, %9 ], [ %4, %7 ]
  %12 = alloca i32, align 4
  store i32 %11, i32* %12, align 4
  %13 = load i32, i32* %12, align 4
  %14 = alloca i32, align 4
  store i32 %13, i32* %14, align 4
  %15 = load i32, i32* %14, align 4
  %16 = alloca i32, align 4
  store i32 %15, i32* %16, align 4
  ret i32 0
}

; Function Attrs: mustprogress nofree nosync nounwind readnone speculatable willreturn
declare { i32, i1 } @llvm.sadd.with.overflow.i32.i32(i32, i32) #2

; Function Attrs: mustprogress nofree nosync nounwind readnone willreturn
declare i1 @llvm.expect.i1.i1(i1, i1) #3

attributes #0 = { nofree nounwind }
attributes #1 = { noinline norecurse nounwind optnone willreturn }
attributes #2 = { mustprogress nofree nosync nounwind readnone speculatable willreturn }
attributes #3 = { mustprogress nofree nosync nounwind readnone willreturn }

!llvm.module.flags = !{!0}
!llvm.dbg.cu = !{!1}

!0 = !{i32 1, !"Debug Info Version", i32 3}
!1 = distinct !DICompileUnit(language: DW_LANG_C, file: !2, producer: "xpl", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, sdk: "xpl")
!2 = !DIFile(filename: "program.ke", directory: ".")
