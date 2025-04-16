# Lab1 实验报告

## 功能实现

实现 `SYS_TRACE` 系统调用。读取内存时，使用 `unsafe` 片段将参数作为裸指针进行读操作；写入内存逻辑同上。实现系统调用计数时，在 `TaskControlBlock` 中添加了一二维数组，记录每个任务不同系统调用号的调用次数，并编写了对应的方法，在任务每次产生系统调用时更新相应计数器。

## 简答作业

1. 正确进入 U 态后，程序的特征还应有：使用 S 态特权指令，访问 S 态寄存器后会报错。自行测试这些内容（运行 三个 bad 测例 (ch2b_bad_*.rs) ）， 描述程序出错行为，同时注意注明你使用的 sbi 及其版本。

    我使用的 `rustsbi` 版本为 `RustSBI version 0.4.0-alpha.1, adapting to RISC-V SBI v2.0.0`.

    - 运行 `ch2b_bad_address.rs`，程序崩溃，内核打印错误信息 `[kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it.`

    - 运行 `ch2b_bad_instructions.rs`，程序崩溃，内核打印错误信息 `[kernel] IllegalInstruction in application, kernel killed it.`

    - 运行 `ch2b_bad_register.rs`，程序崩溃，内核打印错误信息 `[kernel] IllegalInstruction in application, kernel killed it.`

2. 深入理解 `trap.S` 中两个函数 `__alltraps` 和 `__restore` 的作用，并回答如下问题:
    1. L40：刚进入 `__restore` 时，`sp` 代表了什么值。请指出 `__restore` 的两种使用情景。

        刚进入 `__restore` 时，`sp` 代表内核栈地址。`__restore` 的使用场景有：从一次内核态的系统点用/中断/异常处理程序返回用户态。用户态应有程序首次启动时设置初始上下文信息。

    2. L43-L48：这几行汇编代码特殊处理了哪些寄存器？这些寄存器的的值对于进入用户态有何意义？请分别解释。

        这几行汇编代码特殊处理了 `sstatus`、`sepc` 和 `sscratch` 这三个寄存器。
        
        - `sstatus` 是 `Supervisor Status` 寄存器，存储了当前处理器 S 模式下的各种状态信息，包括 `SIE`、`SPIE`、`SPP` 等，与特权模式的切换相关。
        - `sepc` 是 `Supervisor exception program counter`，保存了发生异常或中断时应用程序的指令地址，用于返回用户态时恢复执行被中断的代码。
        - `sscratch` 是 `Scratch register for supervisor trap handlers`，用于在返回用户态前保存内核栈的地址。

    3. L50-L56：为何跳过了 `x2` 和 `x4`？

        - `x2` 寄存器即为 `sp` 寄存器，此处如果操作 `x2` 会导致后面 `LOAD_GP` 时栈的基地址错误。栈空间将会在后面的 `addi sp, sp, 34*8` 处被释放，并通过 `csrrw sp, sscratch, sp` 让 `sp` 寄存器重新指向用用户栈空间。

        - `x4` 寄存器为 `tp` 寄存器，因为目前用户态应用程序并不会用到该寄存器，故不需要保存。

    4. L60：该指令之后，`sp` 和 `sscratch` 中的值分别有什么意义？

        该指令后，`sp` 中的值代表用户栈地址，`sscratch` 中的值代表内核栈地址。

    5. `__restore`：中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？

        状态切换发生在 `sret` 指令。 `sret` 指令用于从 S 模式返回 U模式，在这条指令前，已经完成了通用寄存器的恢复、CSRs 寄存器的设置和用户栈地址（sp 寄存器）的恢复。`sret` 指令执行时，会重新设置 PC 寄存器为用户应用程序中断的位置，并且根据 `sstatus` 寄存器中的特权为将特权级别调整为用户态。

    6. L13：该指令之后，`sp` 和 `sscratch` 中的值分别有什么意义？
    
        该指令之后，`sp` 中的值代表内核栈地址，`sscratch` 中的值代表用户栈地址。

    7. 从 U 态进入 S 态是哪一条指令发生的？
        从 `ecall` 指令发生。在 `syscall.rs` 中，用户态应用程序调用 `syscall` 函数时，都会产生 `ecall` 指令，由硬件根据 `stvec` 寄存器中的跳转向量转到 `__alltraps` 进行处理。


https://five-embeddev.com/quickref/csrs.html