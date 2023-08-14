; ModuleID = 'program.ke'
source_filename = "program.ke"
target triple = "x86_64-unknown-linux-gnu"

@0 = private constant [56 x i8] c"Error: std::u64 addition overflow!\0A    program.ke:2:17\0A\00"
@1 = private constant [57 x i8] c"Error: std::bool addition overflow!\0A    program.ke:6:17\0A\00"

declare i32 @printf(ptr) local_unnamed_addr

; Function Attrs: noinline norecurse optnone willreturn
define i32 @main(i32 %0, ptr %1) local_unnamed_addr #0 {
  %3 = call { i64, i1 } @llvm.sadd.with.overflow.i64.i64(i64 1, i64 2)
  %4 = call i1 @llvm.expect.i1.i1(i1 false, i1 false)
  br i1 %4, label %5, label %7

5:                                                ; preds = %2
  %6 = call i32 @printf(ptr @0)
  br label %8

7:                                                ; preds = %2
  br label %8

8:                                                ; preds = %7, %5
  %9 = alloca i64, align 8
  store i64 3, ptr %9, align 4
  %10 = alloca ptr, align 8
  store ptr %9, ptr %10, align 8
  %11 = alloca i64, align 8
  store ptr %11, ptr %10, align 8
  %12 = alloca ptr, align 8
  store ptr %9, ptr %12, align 8
  %13 = call { i1, i1 } @llvm.sadd.with.overflow.i1.i1(i1 true, i1 false)
  %14 = call i1 @llvm.expect.i1.i1(i1 false, i1 false)
  br i1 %14, label %15, label %17

15:                                               ; preds = %8
  %16 = call i32 @printf(ptr @1)
  br label %18

17:                                               ; preds = %8
  br label %18

18:                                               ; preds = %17, %15
  %19 = alloca i1, align 1
  store i1 true, ptr %19, align 1
  %20 = call i1 @x()
  %21 = alloca i1, align 1
  store i1 %20, ptr %21, align 1
  %22 = call i1 @x()
  %23 = alloca i1, align 1
  store i1 %22, ptr %23, align 1
  %24 = load i1, ptr %21, align 1
  %25 = load i1, ptr %23, align 1
  %26 = icmp eq i1 %24, %25
  %27 = alloca i1, align 1
  store i1 %26, ptr %27, align 1
  ret i32 0
}

; Function Attrs: mustprogress nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare { i64, i1 } @llvm.sadd.with.overflow.i64.i64(i64, i64) #1

; Function Attrs: mustprogress nocallback nofree nosync nounwind willreturn memory(none)
declare i1 @llvm.expect.i1.i1(i1, i1) #2

; Function Attrs: mustprogress nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare { i1, i1 } @llvm.sadd.with.overflow.i1.i1(i1, i1) #1

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none)
define i1 @x() local_unnamed_addr #3 {
  ret i1 true
}

attributes #0 = { noinline norecurse optnone willreturn }
attributes #1 = { mustprogress nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { mustprogress nocallback nofree nosync nounwind willreturn memory(none) }
attributes #3 = { mustprogress nofree norecurse nosync nounwind willreturn memory(none) }

!llvm.module.flags = !{!0}
!llvm.dbg.cu = !{!1}

!0 = !{i32 1, !"Debug Info Version", i32 3}
!1 = distinct !DICompileUnit(language: DW_LANG_C, file: !2, producer: "kestrel", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false, sdk: "kestrel")
!2 = !DIFile(filename: "program.ke", directory: ".")
