use riscv::register::sstatus::{self, Sstatus, SPP};

/*
    riscv register alias:
    x0            zero
    x1            ra
    x2            sp
    x3            gp
    x4            tp
    x5-x7         t0-t2
    x8            s0/fp
    x9            s1
    x10-x11       a0-a1(function args/return values)
    x12-x17       a2-a7(function args)
    x18-x27       s2-s11
    x28-x31       t3-t6
*/
#[repr(C)]
pub struct TrapContext {
    pub x: [usize; 32],
    pub sstatus: Sstatus,
    pub sepc: usize,
}

impl TrapContext {
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }

    /// generate a init trap_context, for first app back to user
    pub fn app_init_context(entry: usize, sp: usize) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry,
        };
        cx.set_sp(sp);
        cx
    }
}