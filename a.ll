; ModuleID = 'program.ke'
source_filename = "program.ke"
target triple = "x86_64-unknown-linux-gnu"

; Function Attrs: nofree nounwind
declare noundef i32 @printf(i8* nocapture noundef readonly) local_unnamed_addr #0

; Function Attrs: noinline nounwind optnone
define i32 @main(i32 %0, i32** %1) local_unnamed_addr #1 {
  %3 = call { i32, i1 } @llvm.sadd.with.overflow.i32.i32(i32 1, i32 2)
  %4 = extractvalue { i32, i1 } %3, 0
  %5 = extractvalue { i32, i1 } %3, 1
  %6 = call i1 @llvm.expect.i1.i1(i1 %5, i1 false)
  br i1 %6, label %7, label %11

7:                                                ; preds = %2
  %string = alloca { [51 x i8] }, align 8
  %8 = getelementptr inbounds { [51 x i8] }, { [51 x i8] }* %string, i32 0, i32 0
  store [51 x i8] c"Error: i32 addition overflow!\0A    program.ke:1:10\0A\00", [51 x i8]* %8, align 1
  %9 = getelementptr [51 x i8], [51 x i8]* %8, i32 0, i32 0
  %10 = call i32 @printf(i8* %9)
  br label %12

11:                                               ; preds = %2
  br label %12

12:                                               ; preds = %11, %7
  %13 = phi i32 [ %4, %11 ], [ -1, %7 ]
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
attributes #1 = { noinline nounwind optnone }
attributes #2 = { mustprogress nofree nosync nounwind readnone speculatable willreturn }
attributes #3 = { mustprogress nofree nosync nounwind readnone willreturn }

!llvm.module.flags = !{!0}
!llvm.dbg.cu = !{!1}

!0 = !{i32 1, !"Debug Info Version", i32 3}
!1 = distinct !DICompileUnit(language: DW_LANG_C, file: !2, producer: "xpl", isOptimized: true, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, sdk: "xpl")
!2 = !DIFile(filename: "program.ke", directory: ".")
