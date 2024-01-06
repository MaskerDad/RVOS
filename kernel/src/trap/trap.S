.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm
    .section .text
    .global __alltraps
    .global __restore
    .align 2
__alltraps:
    csrrw sp, sscratch, sp
    # now sp->kerne_stack, sscratch->user_stack
    
    # allocate a TrapContext on kernel_stack
    addi sp, sp, -34*8
    
    # save g-regs
    # skip zero(x0)
    # skip sp(x2), we will save it later
    # skip tp(x4), application does not use it
    sd x1, 1*8(sp)
    sd x3, 3*8(sp)
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n + 1
    .endr

    # please feel free to use them
    # t0/t1/t2 were saved on kerne_stack
    # sstatus/sepc is important for trap_return
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    
    # read user_sp from sscratch and save it
    csrr t2, sscratch
    sd t2, 2*8(sp)
    
    # next, we're going to call trap_handler
    # set input argument of trap_handler(cx: &mut TrapContext)
    mv a0, sp
    call trap_handler
__restore:
    # case1: start running app by run_next_app->__restore
    # case2: back to U-Mode after handling trap
    
    # for case1, we need to set kernel_sp
    # now sscratch->user_stack
    mv sp, a0
    
    # restore sstatus/sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    ld t2, 2*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    csrw sscratch, t2

    # restore g-regs except sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr

    # release TrapContext on kernel_stack
    addi sp, sp, 34*8
    
    # now sp->kernel_stack, sscratch->user_stack
    csrrw sp, sscratch, sp
    sret