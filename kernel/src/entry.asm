    .section .text.entry
    .globl _start
_start:
    # with a 4096-byte stack per cpu
    # sp = boot_stack_top + (hartid + 1) * 4096
    # a0 is hartid
    la sp, boot_stack_top
    li t0, 1024*4
    addi a0, a0, 1
     
    mv t1, zero
Loop:
    bge t0, a0, End
    sub sp, sp, t0
    addi t1, t1, 1
    beq x0, x0, Loop
End:
    call rust_main

    .section .bss.stack
    .globl boot_stack_lower_bound
boot_stack_lower_bound:
    .space 4096 * 16
    .globl boot_stack_top
boot_stack_top:
