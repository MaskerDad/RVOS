# RVOS
RVOS comes from rCore/xv6-riscv for learning Rust/RISC-V/OS.
Most of the code comes from rcore-tutorial, and based on this simple kernel, do some RISC-V experiments.

xv6-riscv writes the kernel code from machine mode, but this causes some trouble in debugging because the kernel must implement UART-related drivers first, which I'll create another branch to do. In the mainline code, we use 'sbi-RT' to provide runtime services to the kernel, and also use packages like 'riscv' to access registers, which saves us from having to redo the wheel and allows us to focus on the implementation of the operating system (but an understanding of SBI is essential).

## LibOS
![image-20240108163445142](https://cdn.jsdelivr.net/gh/MaskerDad/BlogImage@main/202401081634265.png)

`LibOS` can output some basic information in the SEE environment.

Let's see what needs to be done:
1. Print character sequences to the terminal with the help of 'sbi-rt'
2. Realize shutdown service with 'sbi-rt'
3. Write makefiles to customize the building and running rules of the kernel, and run the kernel through make run
4. Write link scripts to customize the memory layout and program entry of the kernel
5. Write the kernel entry function to achieve basic initialization and print debugging information
6. Implement LOG level control

## BatchOS

![image-20240108163533259](https://cdn.jsdelivr.net/gh/MaskerDad/BlogImage@main/202401081635289.png)

`BatchOS` implements privilege-level isolation, where the kernel program runs in S-Mode and a set of application programs are executed sequentially in U-Mode. These applications are queued for execution, and only after each program completes can the next one be executed. Once all the application programs have finished executing, the system exits.

Let's see what needs to be done:

1. U-Mode:

   * Implementing basic system calls in U-Mode, which is typically done by the standard library.

     * Implementing test programs in U-Mode.

2. S-Mode:

     * The kernel needs to obtain the binary images of user programs and link them to the kernel's data segment.

     * Implementing trap isolation between U-Mode and S-Mode. Traps cause a change in the execution flow, and the kernel needs to save and restore the trap context.

     * Different system calls in U-Mode should point to different branches of the kernel's trap handler.

## TimesharingOS

![image-20240108163653820](https://cdn.jsdelivr.net/gh/MaskerDad/BlogImage@main/202401081636849.png)

`TimesharingOS`:  This version of the kernel supports multiple applications residing in memory and implements simple task scheduling,  including two types:  application active abandonment and clock interrupt-based preemption. Multiple applications alternate execution until  complete.

We are going to face some problems:

1. multiple applications reside in memory at the same time, how to isolate the address?  (Virtual memory is not currently enabled)

   * user_app writes `build.py` to generate binary image files of applications with different starting addresses.

   * `load_apps` loads all {.data} segment application data into different locations in memory at once and assigns them separate user stacks and kernel stacks.


2. How do Task switching and Trap switching fit together?

   * For the kernel task switching scenario, the task execution flow is switched first,  and then the user Trap execution flow is switched. The two processes are actually independent of each other and do not  interfere with each other.

   * `task_switch`
     There is no privilege switching involved,  but the kernel stack is switched because the relevant fields of the kernel stack point to each application's independent  and distinct proprietary content.

   * `trap_switch`
     Involving the switching of privilege level, kernel stack and user stack,  the application Trap falls into the kernel context trap_context is saved by the kernel stack.

3. Why are the registers contained in task_context and trap_context different?

   * `task_context`:  We only need to save the callee saved register because the compiler will insert code to save the caller saved register  for us.

   * `trap_context`:  This is due to the user state program execution privileges or illegal instructions,  asynchronous interrupt caused by the execution stream switch, the compiler will not do any work for us,  so all the registers need to be saved and recovered by the kernel itself. 

---

Let's see what needs to be done:

- [x] Ensure application isolation on physical addresses, which applications need to do before supporting virtual memory
- [x] Refactor the RVOS batch.rs module into two parts: loader and task
  - [x] loader: Loads the app into memory
  - [x] task: For task management
    - [x] Global task manager
    - [x] Task scheduling mechanisms

- [x] RVOS supports active surrender, which enables collaboration between multiple applications, thus improving the overall execution efficiency
- [x] RVOS supports time slice scheduling based on clock interrupts, so that each application obtains CPU usage more fairly

## AddressSpaceOS

So far, we have completed a `TimesharingOS`, which seems to work well from the intuitive perspective of program execution, with these applications alternating until all are completed. However, there are the following drawbacks:

* `Opacity`

  All applications access memory directly through physical addresses, which requires developers to understand the layout of the physical address space. Writing applications is troublesome and requires developers to negotiate changes to the linking script.

* `Insecurity`

  Applications can freely access the address space of other applications, even the kernel's address space.

* `Inflexibility`

  Applications cannot dynamically use available physical memory at runtime. For example, when an application ends, the space it occupies is released, but this space cannot be dynamically used by other running applications.

Therefore, `AddressSpaceOS` will provide an abstract, more transparent, easy-to-use, and secure memory access interface for applications, which is a **virtual memory based on paging mechanism.**

* From the perspective of running application programs:  there is a very large readable/writable/executable address space (Address Space) starting from "0" address.
* From the perspective of the operating system:  each application is confined to run within the physical memory space allocated to it, and cannot read or write the memory space where other applications and the operating system are located.

---

***Key data structures:***

![image-20240109170430637](https://cdn.jsdelivr.net/gh/MaskerDad/BlogImage@main/202401091704792.png)

---

***SV39 address translation (from MIT6.828) :***

<img src="https://cdn.jsdelivr.net/gh/MaskerDad/BlogImage@main/202401091730641.png" alt="../_images/sv39-full.png" style="zoom: 33%;" />

***RVOS memory module initialization:***

![image-20240109170725879](https://cdn.jsdelivr.net/gh/MaskerDad/BlogImage@main/202401091707916.png)

---

***Application/kernel address space layout:***

![image-20240111174947532](https://cdn.jsdelivr.net/gh/MaskerDad/BlogImage@main/202401111749606.png)

---

***Kernel/application address space switching*** - `trap.S`

> `trampoline page`
>
> Next, we need to consider whether the instructions can still be executed continuously before and after switching the address space. It can be seen that we put the entire assembly code in `trap.S` into the `.text.trampoline` segment, and align it to a page of the code segment when adjusting the memory layout:
>
> In this way, this section of assembly code is placed in a physical page frame, and `__alltraps` happens to be located at the beginning of this physical page frame, and its physical address is marked by the external symbol `strampoline`. After paging is enabled, both the kernel and application code can only see their own virtual address spaces, and from their perspectives, this section of assembly code is placed on the highest virtual page of their respective address spaces. Since this section of assembly code involves address space switching when it is executed, it is called a trampoline page.
>
> In a short period of time before and after the trap is generated, there will be a relatively **extreme** situation, that is, when the trap is just generated, the CPU has entered the kernel state (ie Supervisor Mode), but at this time the execution code and data access are still in the user state virtual address space where the application is located, rather than the kernel virtual address space we usually understand. During this special period of time, why can CPU instructions be executed continuously?  Here it should be noted that regardless of the address space of the kernel or the application, the virtual page of the trampoline is located in the same position, and they will also be mapped to the same physical page frame that actually stores this section of assembly code. That is to say, when executing the `__alltraps` or `__restore` function to switch the address space, the mapping methods of the application's user state virtual address space and the kernel state virtual address space of the operating system kernel to the page where the address space switching instruction is located are the same, which means that the control flow of this instruction that switches the address space can still be executed continuously.

Why do we put the application's `TrapContext` in the second highest page of the application address space instead of the kernel stack in the kernel address space?

> The reason is that before saving the Trap context to the kernel stack, we have to do two things:
>
> - We must first switch to the kernel address space, which requires writing the kernel address space's token to the satp register;
> - Then we also need to save the top of the application's kernel stack, so that it can be used as the base address to save the Trap context.
>
> These two steps need to use registers as temporary buffers, but we cannot do this without destroying any of the general-purpose registers. Because in fact we need to use two pieces of information from the kernel: the kernel address space token, and the top of the application's kernel stack, but RISC-V only provides one `sscratch` register that can be used for buffering. Therefore, we have to save the Trap context in a virtual page of the application address space, instead of switching to the kernel address space to save it.

---

Let's see what needs to be done: 

- [x] Dynamic memory allocation is implemented to improve the dynamic use efficiency of memory by the application: the Rust heap data structure is used to make kernel programming more flexible.
- [x] Implement the physical page frame allocator.
- [x] The virtual and real memory mapping mechanism of page table is implemented. Enforce memory isolation between applications and between applications and the kernel:
  - [x] Simplify the compiler's address space Settings for the application: the uniform application starts at `0x10000`
  - [x] o achieve virtual and real address, virtual and real page number conversion auxiliary function
  - [x] Data structure representation of page tables, page table entries, and corresponding methods
  - [x] Application and kernel address space abstraction and corresponding methods
- [x] RVOS was reconstructed to make the original system `TimesharingOS` access the virtual memory mechanism framework:
  - [x] `loader`: Now we can load all apps to the same virtual address '0x10000'
  - [x] `tcb` Refactoring the process control block, PCB should manage the address space
  - [x] `task_manager` Refactoring TASK_MANAGER to support address-space-os
  - [x] `trap_return` Extend the 'trap_return' handling before returning to userspace.Previously we just jumped to '__restore' after handling an exception or before running the app. In fact, the kernel should do some other work like switching address space before actually returning to userspace
  - [x] `trap handling`: Implement application/kernel page table switching
  - [x] sys_write no longer has direct access to data in application space, and manual lookup of the page table is required to retrieve the physical page frame for the user-mode buffer.
  - [x] `translated_byte_buffer`: This function converts a buffer of the application address space into a form directly accessible to the kernel address space.
  - [x] `sys_write`

---

## ProcessOS

> 这个东西是什么？做什么的？为什么(存在的意义)？

* 介绍
  * `ProcessOS` 对  `AssressSpaceOS` 的升级
    * 将 `任务` 进一步扩展为真正意义上的 `进程`，进程相对于任务在运行过程中拥有以下能力：
      * `sys_fork` 创建子进程 
      * `sys_exec` 用新的应用内容覆盖当前进程，即达到执行新应用的目的
      * `sys_waitpid` 等待子进程结束并回收子进程资源
    * 拥有一个用户终端程序或称 **命令行** 应用（Command Line Application, 俗称 **shell** ），形成用户与操作系统进行交互的命令行界面

---

* 核心设计

  * 进程结构分析
    * 进程标识符 `PidHandle`：RAII => Drop => 回收pid
    * 内核栈 `KernelStack`：RAII => Drop => 回收物理页帧 (删除逻辑段)
    * 进程控制块 `TCB/TCBInner`：初始化后不变/变化
    * 进程管理器 `TaskManager`：仅负责管理所有任务
    * 处理器管理 `Processor`：维护CPU正在执行的任务
    
    ![](https://cdn.jsdelivr.net/gh/MaskerDad/BlogImage@main/202401261328817.png)
    
  * 进程管理框架
    * `fork/exec/waitpid` => 创建/覆盖/清理
    
    ![](https://cdn.jsdelivr.net/gh/MaskerDad/BlogImage@main/202401261329157.png)

---

* 你能回答这些问题吗？

  * **AddressSpaceOS中的任务和进程的异同？**
  * **关于 `KernelStack` 的设计，为什么就没有 `UserStack`？**
    * 内核栈位于内核地址空间，需要通过 `Drop` 专门完成回收工作；

    * 用户栈位于应用地址空间，资源在进程退出时回收；
  * **`fork/exec/waitpid` 存在的意义？**
    * 之前的设计都是由内核本身来管理进程，这并不合理。内核本质上是为用户提供服务的，因此应该暴露一些接口让用户拥有必要的进程控制权，这些权利通常有以下：
      * 用户可以选择某一个进程执行：
        * `sys_fork`：在用户态可以动态创建进程，而不是内核一开始就将 `TCB` 全部创建好；
        * `sys_exec`：通过 `sys_fork` 创建出来的只是进程的骨架/空壳，需要将具体且不同的应用程序内容填充进去，或者说覆盖掉；
      * 内核给提供了用户创建进程的能力，也必须同时提供回收进程资源的能力，让用户完全自行管理自身所创建的进程：
        * `sys_waitpid`：父进程彻底回收子进程资源
  * **对于进程资源清理工作，子进程会调用 `exit_current_and_run_next` 首先完成一部分，然后再由用户父进程主动调用 `waitpid` 进行彻底回收，为什么要这么解耦设计？**
    * `exit_current_and_run_next` 回收哪些内容？
      * 用户地址空间回收：存放进程数据和代码的物理页帧被回收
      * 存放页表的那些物理页帧此时还不会被回收，为什么？
    * `waitpid` 回收哪些内容？=> 子进程 `TaskControlBlock`
      * 内核栈 `KernelStack`
      * PID `PidHandle`
      * 存放用户页表的那些物理页帧 `MapArea::data_frames: BTreeMap<VirtPageNum, FrameTracker>`
    * **回答：**当一个进程通过 `exit` 系统调用退出之后，它所占用的资源并不能够立即全部回收。比如该进程的内核栈目前就正用来进行系统调用处理，如果将放置它的物理页帧回收的话，可能会导致系统调用不能正常处理。对于这种问题，一种典型的做法是当进程退出的时候内核立即回收一部分资源并将该进程标记为 **僵尸进程** (Zombie Process) 。之后，由该进程的父进程通过一个名为 `waitpid` 的系统调用来收集该进程的返回状态并回收掉它所占据的全部资源，这样这个进程才被彻底销毁。
  * 为什么从原执行流中分离出一个 `idle_task_cx` 执行流，这么设计有什么好处？
    * `Processor` 有一个不同的 idle 控制流，它运行在这个 CPU 核的启动栈上，功能是尝试从任务管理器中选出一个任务来在当前 CPU 核上执行。在内核初始化完毕之后，会通过调用 `run_tasks` 函数来进入 idle 控制流；
    * 这样做的主要目的是使得换入/换出进程和调度执行流在内核层各自执行在不同的内核栈上，分别是进程自身的内核栈和内核初始化时使用的启动栈。这样的话，调度相关的数据不会出现在进程内核栈上，也使得调度机制对于换出进程的Trap执行流是不可见的，它在决定换出的时候只需调用schedule而无需操心调度的事情。从而各执行流的分工更加明确了，虽然带来了更大的开销。

---

> (StpeByStep) Let's see what needs to be done: 

- [x] 用户层

  - [x] 增加系统调用
    - [x] RVOS进程模型的三个核心系统调用：`sys_fork/exec/waitpid`
    - [x] 查看进程PID的系统调用 `sys_getpid`
    - [x] 允许应用程序获取用户键盘输入: `sys_read` 系统调用

  - [x] 一组新的应用程序

    - [x] 运行在U-Mode下，但和内核深度绑定的特殊应用程序：
      - [x] 用户初始程序 `initproc.rs` ：会被内核 "唯一/自动/最早" 加载并执行
      - [x] shell 程序 `user_shell.rs` ：从键盘接收用户输入的应用名并执行对应的应用

    - [x] 一系列普通测试程序

- [x] 内核层

  - [x] 支持基于应用名查找应用的 ELF 可执行文件:
    - [x] 在 `os/build.rs` 更新了 `link_app.S` 的格式使得它包含每个应用的名字
    - [x] 提供 `get_app_data_by_name` 接口获取应用的 ELF 数据
  - [x] 进程管理核心数据结构
    - [x] 重构 `TaskControlBlock` 
      - [x] 新增 `PidHandle`: 全局PID分配器，提供RAII；
      - [x] 新增 `KernelStack`: 基于 `PID` 分配内核栈空间，提供RAII；
      - [x] 重构 `TaskControlBlock`: 拆分成 `immutable/mutable` 两部分；
    - [x] 任务管理器 `TaskManager` 功能解耦：
      - [x] 任务管理器 `TaskManager` 仅负责维护一个就绪任务队列;
      - [x] `Processor` 
        - [x] `current`: 维护正在执行的任务；
        - [x] `run_tasks/schedule`: 任务调度的idle控制流以及切换到该控制流的方法；

  - [x] 进程管理机制

    - [x] 初始进程的创建

      在内核初始化时调用 `add_initproc` 函数，其读取并解析初始应用 `initproc` 的 ELF 文件数据并创建初始进程 `INITPROC` ，随后会将它加入到全局任务管理器 `TASK_MANAGER` 中参与调度；

    - [x] 进程调度机制

      `suspend_current_and_run_next/exit_current_and_run_next`

    - [x] 进程生成机制

      增加内核对`fork/exec` 两个系统调用的支持，它们基于 `TaskControlBlock::fork/exec`；

    - [x] 进程资源回收机制
      - [x] 当一个进程主动退出或出错退出的时候，在 `exit_current_and_run_next` 中会立即回收一部分资源并在进程控制块中保存退出码；
      - [x] 父进程通过 `waitpid` 系统调用捕获到子进程退出码之后，它的进程控制块才会被回收，从而该进程的所有资源都被彻底回收；

  - [x] 进程的 I/O 输入机制

    支持用户终端程序 `user_shell` 读取用户键盘输入的功能，需要实现 `read` 系统调用。

---

* riscv-privilege-emu (isa/platform/hw) => replace/translate
  * riscv-user-emu

* ***os/hypervisor (AEE) => trap/simulate***
  * os == hypervisor `type-1`
  * os (hypervisor `type-2`)  

---

## FilesystemOS

* `rvfs`：搞一个新仓库来写这部分
* RVOS接入 `rvos`