; ModuleID = 'program.ke'
source_filename = "program.ke"
target triple = "x86_64-unknown-linux-gnu"

@0 = private constant [51 x i8] c"Error: i32 addition overflow!\0A    program.ke:1:12\0A\00"
@1 = private constant [51 x i8] c"Error: i32 addition overflow!\0A    program.ke:4:14\0A\00"
@2 = private constant [50 x i8] c"Error: i32 addition overflow!\0A    program.ke:6:1\0A\00"

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
  %11 = phi i32 [ %4, %9 ], [ undef, %7 ]
  %12 = alloca i32, align 4
  store i32 %11, i32* %12, align 4
  %13 = load i32, i32* %12, align 4
  %14 = alloca i32, align 4
  store i32 %13, i32* %14, align 4
  %15 = load i32, i32* %12, align 4
  %16 = alloca i32, align 4
  store i32 %15, i32* %16, align 4
  %17 = load i32, i32* %16, align 4
  %18 = load i32, i32* %14, align 4
  %19 = call { i32, i1 } @llvm.sadd.with.overflow.i32.i32(i32 %17, i32 %18)
  %20 = extractvalue { i32, i1 } %19, 0
  %21 = extractvalue { i32, i1 } %19, 1
  %22 = call i1 @llvm.expect.i1.i1(i1 %21, i1 false)
  br i1 %22, label %23, label %25

23:                                               ; preds = %10
  %24 = call i32 @printf(i8* getelementptr inbounds ([51 x i8], [51 x i8]* @1, i32 0, i32 0))
  br label %26

25:                                               ; preds = %10
  br label %26

26:                                               ; preds = %25, %23
  %27 = phi i32 [ %20, %25 ], [ undef, %23 ]
  %28 = alloca i32, align 4
  store i32 %27, i32* %28, align 4
  store i32 0, i32* %28, align 4
  %29 = load i32, i32* %16, align 4
  %30 = load i32, i32* %12, align 4
  %31 = call { i32, i1 } @llvm.sadd.with.overflow.i32.i32(i32 %29, i32 %30)
  %32 = extractvalue { i32, i1 } %31, 0
  %33 = extractvalue { i32, i1 } %31, 1
  %34 = call i1 @llvm.expect.i1.i1(i1 %33, i1 false)
  br i1 %34, label %35, label %37

35:                                               ; preds = %26
  %36 = call i32 @printf(i8* getelementptr inbounds ([50 x i8], [50 x i8]* @2, i32 0, i32 0))
  br label %38

37:                                               ; preds = %26
  br label %38

38:                                               ; preds = %37, %35
  %39 = phi i32 [ %32, %37 ], [ undef, %35 ]
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
