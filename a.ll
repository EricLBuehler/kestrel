; ModuleID = 'program.ke'
source_filename = "program.ke"
target triple = "x86_64-unknown-linux-gnu"

@0 = private constant [56 x i8] c"Error: std::u64 addition overflow!\0A    program.ke:2:17\0A\00"
@1 = private constant [57 x i8] c"Error: std::bool addition overflow!\0A    program.ke:6:17\0A\00"

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
  %8 = call i32 @printf(i8* getelementptr inbounds ([56 x i8], [56 x i8]* @0, i32 0, i32 0))
  br label %10

9:                                                ; preds = %2
  br label %10

10:                                               ; preds = %9, %7
  %11 = phi i64 [ %4, %9 ], [ undef, %7 ]
  %12 = alloca i64, align 8
  store i64 %11, i64* %12, align 4
  %13 = alloca i64*, align 8
  store i64* %12, i64** %13, align 8
  %14 = alloca i64, align 8
  store i64 100, i64* %14, align 4
  %15 = alloca i64*, align 8
  store i64* %12, i64** %15, align 8
  %16 = call { i1, i1 } @llvm.sadd.with.overflow.i1.i1(i1 true, i1 false)
  %17 = extractvalue { i1, i1 } %16, 0
  %18 = extractvalue { i1, i1 } %16, 1
  %19 = call i1 @llvm.expect.i1.i1(i1 %18, i1 false)
  br i1 %19, label %20, label %22

20:                                               ; preds = %10
  %21 = call i32 @printf(i8* getelementptr inbounds ([57 x i8], [57 x i8]* @1, i32 0, i32 0))
  br label %23

22:                                               ; preds = %10
  br label %23

23:                                               ; preds = %22, %20
  %24 = phi i1 [ %17, %22 ], [ undef, %20 ]
  %25 = alloca i1, align 1
  store i1 %24, i1* %25, align 1
  ret i32 0
}

; Function Attrs: mustprogress nofree nosync nounwind readnone speculatable willreturn
declare { i64, i1 } @llvm.sadd.with.overflow.i64.i64(i64, i64) #2

; Function Attrs: mustprogress nofree nosync nounwind readnone willreturn
declare i1 @llvm.expect.i1.i1(i1, i1) #3

; Function Attrs: mustprogress nofree nosync nounwind readnone speculatable willreturn
declare { i1, i1 } @llvm.sadd.with.overflow.i1.i1(i1, i1) #2

attributes #0 = { nofree nounwind }
attributes #1 = { noinline norecurse nounwind optnone willreturn }
attributes #2 = { mustprogress nofree nosync nounwind readnone speculatable willreturn }
attributes #3 = { mustprogress nofree nosync nounwind readnone willreturn }

!llvm.module.flags = !{!0}
!llvm.dbg.cu = !{!1}

!0 = !{i32 1, !"Debug Info Version", i32 3}
!1 = distinct !DICompileUnit(language: DW_LANG_C, file: !2, producer: "kestrel", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, sdk: "kestrel")
!2 = !DIFile(filename: "program.ke", directory: ".")
