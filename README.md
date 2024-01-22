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
- [ ] RVOS was reconstructed to make the original system `TimesharingOS` access the virtual memory mechanism framework:
  - [x] `loader`: Now we can load all apps to the same virtual address '0x10000'
  - [ ] `tcb` Refactoring the process control block, PCB should manage the address space
  - [ ] `task_manager` Refactoring TASK_MANAGER to support address-space-os
  - [ ] `trap_return` Extend the 'trap_return' handling before returning to userspace.Previously we just jumped to '__restore' after handling an exception or before running the app. In fact, the kernel should do some other work like switching address space before actually returning to userspace
  - [ ] `trap handling`：Implement application/kernel page table switching
  - [ ] `access different address-space`：Implement memory access across address Spaces