.option norvc
.attribute arch, "rv64gc"

.section .text.init
.global _start
_start:
    # 0 hart stack pointer & interrupt setup
    # store hartid in tp resgister
    mv      a2, a1
    la      a1, main
    j       _awaken

.section .text
.global _park
_park:
    # set sstatus.sie = 1
    csrr    t0, sstatus
    li      t1, 0b10
    or      t0, t0, t1
    csrw    sstatus, t0
    # set sie.ssie = 1
    csrr    t0, sie
    li      t1, 0b10
    or      t0, t0, t1
    csrw    sie, t0
0:
    wfi
    j       0b

.section .text
.global _awaken
_awaken:
    mv      tp, a0 
    # locate stack pointer
    mv      t0, tp
    la      sp, _stack_size
    mul     t0, t0, sp
    la      sp, _kernel_end
    sub     sp, sp, t0
    # setup hart, sstatus.fs = 1(Initial), stvec = _kernel_trap(Direct)
    la      t0, _kernel_trap
    csrw    stvec, t0
    li      t0, 0b01 << 13
    csrw    sstatus, t0
    mv      ra, a1
    mv      a1, a2
    ret

.section .text
.align 4
.global _kernel_trap
.global _from_kernel
_kernel_trap:
    # make room for gp and sp
    addi    sp, sp, -512
    # save generic registers.
    sd      x0, 0(sp)
    sd      x1, 8(sp)
    sd      x2, 16(sp)
    sd      x3, 24(sp)
    sd      x4, 32(sp)
    sd      x5, 40(sp)
    sd      x6, 48(sp)
    sd      x7, 56(sp)
    sd      x8, 64(sp)
    sd      x9, 72(sp)
    sd      x10, 80(sp)
    sd      x11, 88(sp)
    sd      x12, 96(sp)
    sd      x13, 104(sp)
    sd      x14, 112(sp)
    sd      x15, 120(sp)
    sd      x16, 128(sp)
    sd      x17, 136(sp)
    sd      x18, 144(sp)
    sd      x19, 152(sp)
    sd      x20, 160(sp)
    sd      x21, 168(sp)
    sd      x22, 176(sp)
    sd      x23, 184(sp)
    sd      x24, 192(sp)
    sd      x25, 200(sp)
    sd      x26, 208(sp)
    sd      x27, 216(sp)
    sd      x28, 224(sp)
    sd      x29, 232(sp)
    sd      x30, 240(sp)
    sd      x31, 248(sp)
    # check if need to save floating registers
    csrr    t0, sstatus
    srli   t0, t0, 13
    andi    t0, t0, 0b11
    li      t1, 3
    bne     t0, t1, 1f
0:
    fsd      f0, 256(sp)
    fsd      f1, 264(sp)
    fsd      f2, 272(sp)
    fsd      f3, 280(sp)
    fsd      f4, 288(sp)
    fsd      f5, 296(sp)
    fsd      f6, 304(sp)
    fsd      f7, 312(sp)
    fsd      f8, 320(sp)
    fsd      f9, 328(sp)
    fsd      f10, 336(sp)
    fsd      f11, 344(sp)
    fsd      f12, 352(sp)
    fsd      f13, 360(sp)
    fsd      f14, 368(sp)
    fsd      f15, 376(sp)
    fsd      f16, 384(sp)
    fsd      f17, 392(sp)
    fsd      f18, 400(sp)
    fsd      f19, 408(sp)
    fsd      f20, 416(sp)
    fsd      f21, 424(sp)
    fsd      f22, 432(sp)
    fsd      f23, 440(sp)
    fsd      f24, 448(sp)
    fsd      f25, 456(sp)
    fsd      f26, 464(sp)
    fsd      f27, 472(sp)
    fsd      f28, 480(sp)
    fsd      f29, 488(sp)
    fsd      f30, 496(sp)
    fsd      f31, 504(sp)
    # make floating dirty bit clean
    csrr    t0, sstatus
    li      t1, 2
    slli    t1, t1, 13
    not     t1, t1
    and     t0, t0, t1
    csrw    sstatus, t0
1:
    # go prepare for rust trap handler
    csrr	a0, scause
    csrr    a1, stval
    call    handle_kernel_trap
    # check if need to restore floating registers
    csrr    t0, sstatus
    srli    t0, t0, 13
    andi    t0, t0, 0b11
    li      t1, 3
    bne     t0, t1, 3f
2:
    fld     f0, 256(sp)
    fld     f1, 264(sp)
    fld     f2, 272(sp)
    fld     f3, 280(sp)
    fld     f4, 288(sp)
    fld     f5, 296(sp)
    fld     f6, 304(sp)
    fld     f7, 312(sp)
    fld     f8, 320(sp)
    fld     f9, 328(sp)
    fld     f10, 336(sp)
    fld     f11, 344(sp)
    fld     f12, 352(sp)
    fld     f13, 360(sp)
    fld     f14, 368(sp)
    fld     f15, 376(sp)
    fld     f16, 384(sp)
    fld     f17, 392(sp)
    fld     f18, 400(sp)
    fld     f19, 408(sp)
    fld     f20, 416(sp)
    fld     f21, 424(sp)
    fld     f22, 432(sp)
    fld     f23, 440(sp)
    fld     f24, 448(sp)
    fld     f25, 456(sp)
    fld     f26, 464(sp)
    fld     f27, 472(sp)
    fld     f28, 480(sp)
    fld     f29, 488(sp)
    fld     f30, 496(sp)
    fld     f31, 504(sp)
    # make floating dirty bit clean
    csrr    t0, sstatus
    li      t1, 2
    slli    t1, t1, 13
    not     t1, t1
    and     t0, t0, t1
    csrw    sstatus, t0
3:
    # restore generic registers
    # no need to restroe sp, tp, they are always the same
    ld      x0, 0(sp)
    ld      x1, 8(sp)
    ld      x3, 24(sp)
    ld      x5, 40(sp)
    ld      x6, 48(sp)
    ld      x7, 56(sp)
    ld      x8, 64(sp)
    ld      x9, 72(sp)
    ld      x10, 80(sp)
    ld      x11, 88(sp)
    ld      x12, 96(sp)
    ld      x13, 104(sp)
    ld      x14, 112(sp)
    ld      x15, 120(sp)
    ld      x16, 128(sp)
    ld      x17, 136(sp)
    ld      x18, 144(sp)
    ld      x19, 152(sp)
    ld      x20, 160(sp)
    ld      x21, 168(sp)
    ld      x22, 176(sp)
    ld      x23, 184(sp)
    ld      x24, 192(sp)
    ld      x25, 200(sp)
    ld      x26, 208(sp)
    ld      x27, 216(sp)
    ld      x28, 224(sp)
    ld      x29, 232(sp)
    ld      x30, 240(sp)
    ld      x31, 248(sp)

    addi sp, sp, 512
    sret
    
.section .trampoline.user_trap
.align 4
.global _user_trap
.global _restore
_user_trap:
    # sscratch holds the trapframe address
    csrrw	t6, sscratch, t6
    # save generic registers to t6
    sd      x0, 0(t6)
    sd      x1, 8(t6)
    sd      x2, 16(t6)
    sd      x3, 24(t6)
    sd      x4, 32(t6)
    sd      x5, 40(t6)
    sd      x6, 48(t6)
    sd      x7, 56(t6)
    sd      x8, 64(t6)
    sd      x9, 72(t6)
    sd      x10, 80(t6)
    sd      x11, 88(t6)
    sd      x12, 96(t6)
    sd      x13, 104(t6)
    sd      x14, 112(t6)
    sd      x15, 120(t6)
    sd      x16, 128(t6)
    sd      x17, 136(t6)
    sd      x18, 144(t6)
    sd      x19, 152(t6)
    sd      x20, 160(t6)
    sd      x21, 168(t6)
    sd      x22, 176(t6)
    sd      x23, 184(t6)
    sd      x24, 192(t6)
    sd      x25, 200(t6)
    sd      x26, 208(t6)
    sd      x27, 216(t6)
    sd      x28, 224(t6)
    sd      x29, 232(t6)
    sd      x30, 240(t6)
    mv		a0, t6
    csrrw	t6, sscratch, a0
    sd      t6, 248(a0)
    # save floating registers
    csrr    t0, sstatus
    srli    t0, t0, 13
    andi    t0, t0, 0b11
    li      t1, 3
    bne     t0, t1, 1f
0:
    fsd     f0, 256(a0)
    fsd     f1, 264(a0)
    fsd     f2, 272(a0)
    fsd     f3, 280(a0)
    fsd     f4, 288(a0)
    fsd     f5, 296(a0)
    fsd     f6, 304(a0)
    fsd     f7, 312(a0)
    fsd     f8, 320(a0)
    fsd     f9, 328(a0)
    fsd     f10, 336(a0)
    fsd     f11, 344(a0)
    fsd     f12, 352(a0)
    fsd     f13, 360(a0)
    fsd     f14, 368(a0)
    fsd     f15, 376(a0)
    fsd     f16, 384(a0)
    fsd     f17, 392(a0)
    fsd     f18, 400(a0)
    fsd     f19, 408(a0)
    fsd     f20, 416(a0)
    fsd     f21, 424(a0)
    fsd     f22, 432(a0)
    fsd     f23, 440(a0)
    fsd     f24, 448(a0)
    fsd     f25, 456(a0)
    fsd     f26, 464(a0)
    fsd     f27, 472(a0)
    fsd     f28, 480(a0)
    fsd     f29, 488(a0)
    fsd     f30, 496(a0)
    fsd     f31, 504(a0)
    # make floating dirty bit clean
    csrr    t0, sstatus
    li      t1, 2
    slli    t1, t1, 13
    not     t1, t1
    and     t0, t0, t1
    csrw    sstatus, t0
1:
    # save pc
    csrr    t6, sepc
    sd      t6, 512(a0)
    # load tp and sp
    ld      tp, 520(a0)
    ld      sp, 528(a0)
    # traps here redirect to kernel trap
    ld      t6, 544(a0)
    csrw    stvec, t6
    # install kernel page table
    ld      t6, 536(a0)
    csrw    satp, t6
    sfence.vma
    # prepare arguments
    csrr    a0, scause
    csrr    a1, stval
    ld      t0, _handle_user_trap
    jalr    t0
    # a0 -> satp
    # a1 -> &trapframe(inaccessible before satp install)
    # install context
    csrw    satp, a0
    sfence.vma
# _restore(satp: usize, trampframe: &TrampFrame)
_restore:
    # traps here redirect to user trap
    # restore special registers
    csrw    sscratch, a1
    ld      t6, 552(a1)
    csrw    stvec, t6
    ld      t6, 512(a1)
    csrw    sepc, t6
    # save tp and sp
    sd      tp, 520(a1)
    sd      sp, 528(a1)
    # restore floating registers
    csrr    t0, sstatus
    srli    t0, t0, 13
    andi    t0, t0, 0b11
    li      t1, 3
    bne     t0, t1, 3f
2:
    fld      f0, 256(a1)
    fld      f1, 264(a1)
    fld      f2, 272(a1)
    fld      f3, 280(a1)
    fld      f4, 288(a1)
    fld      f5, 296(a1)
    fld      f6, 304(a1)
    fld      f7, 312(a1)
    fld      f8, 320(a1)
    fld      f9, 328(a1)
    fld      f10, 336(a1)
    fld      f11, 344(a1)
    fld      f12, 352(a1)
    fld      f13, 360(a1)
    fld      f14, 368(a1)
    fld      f15, 376(a1)
    fld      f16, 384(a1)
    fld      f17, 392(a1)
    fld      f18, 400(a1)
    fld      f19, 408(a1)
    fld      f20, 416(a1)
    fld      f21, 424(a1)
    fld      f22, 432(a1)
    fld      f23, 440(a1)
    fld      f24, 448(a1)
    fld      f25, 456(a1)
    fld      f26, 464(a1)
    fld      f27, 472(a1)
    fld      f28, 480(a1)
    fld      f29, 488(a1)
    fld      f30, 496(a1)
    fld      f31, 504(a1)
    # make floating dirty bit clean
    csrr    t0, sstatus
    li      t1, 2
    slli    t1, t1, 13
    not     t1, t1
    and     t0, t0, t1
    csrw    sstatus, t0
3:
    # restore generic registers
    mv      t6, a1
    ld      x0, 0(t6)
    ld      x1, 8(t6)
    ld      x2, 16(t6)
    ld      x3, 24(t6)
    ld      x4, 32(t6)
    ld      x5, 40(t6)
    ld      x6, 48(t6)
    ld      x7, 56(t6)
    ld      x8, 64(t6)
    ld      x9, 72(t6)
    ld      x10, 80(t6)
    ld      x11, 88(t6)
    ld      x12, 96(t6)
    ld      x13, 104(t6)
    ld      x14, 112(t6)
    ld      x15, 120(t6)
    ld      x16, 128(t6)
    ld      x17, 136(t6)
    ld      x18, 144(t6)
    ld      x19, 152(t6)
    ld      x20, 160(t6)
    ld      x21, 168(t6)
    ld      x22, 176(t6)
    ld      x23, 184(t6)
    ld      x24, 192(t6)
    ld      x25, 200(t6)
    ld      x26, 208(t6)
    ld      x27, 216(t6)
    ld      x28, 224(t6)
    ld      x29, 232(t6)
    ld      x30, 240(t6)
_enter_user_breakpoint:
    sret
_handle_user_trap: .dword handle_user_trap

.section .text
.global _switch
# _switch(kernel_satp, trampoline: Address, satp: usize, trapframe: &TrapFrame)
_switch:
    csrw    satp, a0
    sfence.vma
    la      t0, _switch_internal
    li      t1, 0xfff
    and     t0, t0, t1
    add     ra, a1, t0
    mv      a0, a1
    mv      a1, a2
    mv      a2, a3
    ret

.section .trampoline.switch
.global _switch_internal
# _jump(trampoline: Address, satp: usize, trapframe: &TrapFrame)
_switch_internal:
    # modify sstatus.spie to 1 .spp to 0
    csrr    t0, sstatus
    li      t1, 0b10000
    or      t0, t0, t1
    li      t1, (1 << 8)
    not     t1, t1
    and     t0, t0, t1
    csrw    sstatus, t0
    # install page table
    csrw    satp, a1
    la      t0, _restore
    li      t1, 0xfff
    and     t0, t0, t1
    add     ra, a0, t0
    mv      a0, a1
    mv      a1, a2
    sfence.vma
    ret