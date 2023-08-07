; ModuleID = 'program.ke'
source_filename = "program.ke"
target triple = "x86_64-unknown-linux-gnu"

@0 = private constant [50 x i8] c"Error: i32 addition overflow!\0A    program.ke:1:9\0A\00"

; Function Attrs: nofree nounwind
declare noundef i32 @printf(i8* nocapture noundef readonly) local_unnamed_addr #0

; Function Attrs: noinline norecurse nounwind optnone willreturn
define i32 @main(i32 %0, i32** %1) local_unnamed_addr #1 {
  %3 = call { i64, i1 } @llvm.sadd.with.overflow.i64.i64(i64 1, i64 2)
  %4 = extractvalue { i64, i1 } %3, 0
  %5 = extractvalue { i64, i1 } %3, 1
  %6 = call i1 @llvm.expect.i1.i1(i1 %5, i1 false)
  br i1 %6, label %7, label %9

7:                                                ; preds = %2
  %8 = call i32 @printf(i8* getelementptr inbounds ([50 x i8], [50 x i8]* @0, i32 0, i32 0))
  br label %10

9:                                                ; preds = %2
  br label %10

10:                                               ; preds = %9, %7
  %11 = phi i64 [ %4, %9 ], [ undef, %7 ]
  %12 = alloca i64, align 8
  store i64 %11, i64* %12, align 4
  %13 = alloca i64*, align 8
  store i64* %12, i64** %13, align 8
  %14 = alloca i32, align 4
  store i32 100, i32* %14, align 4
  %15 = alloca i64*, align 8
  store i64* %12, i64** %15, align 8
  ret i32 0
}

; Function Attrs: mustprogress nofree nosync nounwind readnone speculatable willreturn
declare { i64, i1 } @llvm.sadd.with.overflow.i64.i64(i64, i64) #2

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
