; ModuleID = 'program.ke'
source_filename = "program.ke"
target triple = "x86_64-unknown-linux-gnu"

@0 = private constant [56 x i8] c"Error: std::i32 addition overflow!\0A    program.ke:7:18\0A\00"

; Function Attrs: nofree nounwind
declare noundef i32 @printf(i8* nocapture noundef readonly) local_unnamed_addr #0

; Function Attrs: noinline norecurse nounwind optnone willreturn
define i32 @main(i32 %0, i32** %1) local_unnamed_addr #1 {
  %3 = alloca i32, align 4
  %4 = alloca i32*, align 8
  store i32* %3, i32** %4, align 8
  br label %else

5:                                                ; preds = %else
  %6 = call i32 @printf(i8* getelementptr inbounds ([56 x i8], [56 x i8]* @0, i32 0, i32 0))
  br label %13

7:                                                ; preds = %else
  br label %13

done:                                             ; preds = %13
  %8 = alloca i32**, align 8
  store i32** %4, i32*** %8, align 8
  ret i32 0

else:                                             ; preds = %2
  %9 = call { i32, i1 } @llvm.sadd.with.overflow.i32.i32(i32 1, i32 1)
  %10 = extractvalue { i32, i1 } %9, 0
  %11 = extractvalue { i32, i1 } %9, 1
  %12 = call i1 @llvm.expect.i1.i1(i1 %11, i1 false)
  br i1 %12, label %5, label %7

13:                                               ; preds = %7, %5
  %14 = phi i32 [ %10, %7 ], [ undef, %5 ]
  %15 = alloca i32, align 4
  store i32 %14, i32* %15, align 4
  br label %done
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
!1 = distinct !DICompileUnit(language: DW_LANG_C, file: !2, producer: "kestrel", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, sdk: "kestrel")
!2 = !DIFile(filename: "program.ke", directory: ".")
