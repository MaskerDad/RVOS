# RVOS
RVOS comes from rCore/xv6-riscv for learning Rust/RISC-V/OS.
Most of the code comes from rcore-tutorial, and based on this simple kernel, do some RISC-V experiments.

xv6-riscv writes the kernel code from machine mode, but this causes some trouble in debugging because the kernel must implement UART-related drivers first, which I'll create another branch to do. In the mainline code, we use 'sbi-RT' to provide runtime services to the kernel, and also use packages like 'riscv' to access registers, which saves us from having to redo the wheel and allows us to focus on the implementation of the operating system (but an understanding of SBI is essential).

## LibOS
LibOS can output some basic information in the SEE environment.

Let's see what needs to be done:
1. Print character sequences to the terminal with the help of 'sbi-rt'
2. Realize shutdown service with 'sbi-rt'
3. Write makefiles to customize the building and running rules of the kernel, and run the kernel through make run
4. Write link scripts to customize the memory layout and program entry of the kernel
5. Write the kernel entry function to achieve basic initialization and print debugging information
6. Implement LOG level control

## BatchOS
BatchOS implements privilege-level isolation, where the kernel program runs in S-Mode and a set of application programs are executed sequentially in U-Mode. These applications are queued for execution, and only after each program completes can the next one be executed. Once all the application programs have finished executing, the system exits.

Let's see what needs to be done:
1. U-Mode:
  * Implementing basic system calls in U-Mode, which is typically done by the standard library.
  * Implementing test programs in U-Mode.
2. S-Mode:
  * The kernel needs to obtain the binary images of user programs and link them to the kernel's data segment.
  * Implementing trap isolation between U-Mode and S-Mode. Traps cause a change in the execution flow, and the kernel needs to save and restore the trap context.
  * Different system calls in U-Mode should point to different branches of the kernel's trap handler.

## TimesharingOS