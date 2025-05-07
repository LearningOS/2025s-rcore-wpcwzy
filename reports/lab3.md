## Lab3 实验报告

## 功能实现

上一章实现的 `sys_get_time` `sys_mmap` 和 `sys_munmap` 不需要大改，但是因为本章更改了任务管理器 `TaskManager` 的结构，所以需要对 `mmap` 和 `munmap` 获取当前任务 `MemorySet` 的代码进行少量修改。

`spawn` 系统调用的实现主要参考了 `fork` 系统调用的实现，区别在于不需要复制父进程的地址空间。根据程序名加载 `elf_data`，创建新的进程控制块，设置好进程的父子关系后，将新的进程控制块添加到准备队列中。

stride 调度算法的实现为，在进程控制块中增加 `stride` 和 `priority` 属性，实现 `set_priority` 系统调用来修改进程的优先级。修改进程管理器从准备队列取进程的逻辑，之前的逻辑为取队头，现在改为使用遍历算法找到 stride 最小的进程进行调度，并为该进程的 stride 根据公式 $P.pass = \frac{BIG\_STRIDE}{P.priority}$ 加上对应的步长 pass.

在我的实现中，`BIG_STRIDE = 0x1000`.

## 简答作业

**stride 算法深入**

stride 算法原理非常简单，但是有一个比较大的问题。例如两个 pass = 10 的进程，使用 8bit 无符号整形储存 stride， p1.stride = 255, p2.stride = 250，在 p2 执行一个时间片后，理论上下一次应该 p1 执行。

- 实际情况时轮到 p1 执行吗？为什么？

    不是，理由如下：

    p2 执行完一个时间片后，`p2.stride += pass = 260`，但是使用 8bit 无符号整型存储 stride，此时发生溢出，`p2.stride` 值实际为4。下一次调度时，选取 stride 最小的进程进行执行，那么下一次还是 p2 被选择。

我们之前要求进程优先级 >= 2 其实就是为了解决这个问题。可以证明， 在不考虑溢出的情况下 , 在进程优先级全部 >= 2 的情况下，如果严格按照算法执行，那么 STRIDE_MAX – STRIDE_MIN <= BigStride / 2。

- 为什么？尝试简单说明（不要求严格证明）。

    当进程优先级都不低于2时，将会满足 $pass \leq \frac{BigStride}{2}$，而每次调度时都会选择 stride 最小的进程进行调度，并将该进程的 stride 累加 pass.

    这也就是说，不管选择哪个进程运行，其 stride 值最多只会增加 $\frac{BigStride}{2}$，又因为调度算法每次选择 stride 值最小的进程进行调度，不会使现在已经是 stride 最大的进程的 stride 值继续增大，故在任何时刻都不会出现 STRIDE_MAX – STRIDE_MIN > BigStride / 2 的情况。

- 已知以上结论，考虑溢出的情况下，可以为 Stride 设计特别的比较器，让 BinaryHeap<Stride> 的 pop 方法能返回真正最小的 Stride。补全下列代码中的 partial_cmp 函数，假设两个 Stride 永远不会相等。

    ```rust
    use core::cmp::Ordering;

    struct Stride(u64);

    impl PartialOrd for Stride {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            let a = self.0;
            let b = other.0;

            if a < b {
                // 按照前面证明不溢出时差值不会大于一半
                if (b-a) > 127 {
                    Some(Ordering::Greater)
                } else {
                    Some(Ordering::Less)
                }
            } else {
                // 同上
                if (a-b) > 127 {
                    Some(Ordering::Less)
                } else {
                    Some(Ordering::Greater)
                }
            }
        }
    }

    impl PartialEq for Stride {
        fn eq(&self, other: &Self) -> bool {
            // 题目假设两 stride 永远不会相等
            false
        }
    }
    ```
    
TIPS: 使用 8 bits 存储 stride, BigStride = 255, 则: `(125 < 255) == false`, `(129 < 255) == true`.

# 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与以下各位就以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：
   
   无

2. 此外，我还参考了一下资料，还在代码中对应的位置以注释形式记录了具体的参考来源和内容。

    无

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。