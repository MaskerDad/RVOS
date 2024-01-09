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

介绍：

---

关键数据结构：

![image-20240109170430637](https://cdn.jsdelivr.net/gh/MaskerDad/BlogImage@main/202401091704792.png)

---

RVOS内存模块初始化：

![image-20240109170725879](https://cdn.jsdelivr.net/gh/MaskerDad/BlogImage@main/202401091707916.png)

---

内核/应用地址空间切换 - `trap.S`

//TODO



---

Let's see what needs to be done: 

- [ ] 实现动态内存分配，提高了应用程序对内存的动态使用效率：使用Rust堆数据结构，使内核编程更灵活

- [ ] 实现物理页帧分配器
- [ ] 实现虚实地址、虚实页号的转换辅助函数
- [ ] 实现页表的虚实内存映射机制。加强应用之间，应用与内核之间的内存隔离：
  - [ ] 简化编译器对应用的地址空间设置：统一应用程序的起始地址为 `0x10000`
  - [ ] 页表、页表项的数据结构表示以及相应方法
  - [ ] 应用、内核地址空间的抽象以及相应方法

- [ ] 重构RVOS，使原系统 `timesharing-os` 接入虚拟内存机制框架
  - [ ] `trap handling`：实现页表切换 - trampoline
  - [ ] `access different address-space`：实现跨越地址空间的内存访问