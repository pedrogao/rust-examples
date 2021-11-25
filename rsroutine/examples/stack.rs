#![feature(llvm_asm)]

const STACK_SIZE: isize = 48;

#[derive(Debug, Default)]
#[repr(C)]
struct RoutineContext {
    rsp: u64, // rsp 寄存器
}

fn hello() -> ! {
    println!("hello from rsroutine!");
    loop {} // loop，永不退出
}

unsafe fn ctx_switch(new: *const RoutineContext) {
    // rsp register stores a pointer to the next value on the stack
    // 通过 rsp 来模拟栈，rsp-- 就是入栈，rsp++就是出栈
    // rsp 栈顶，在高位，存储 caller 的返回地址(return address)
    // 1. 覆盖 rsp 寄存器的值，将其置为 hello 函数的入口地址
    // 2. 调用 ret，从栈顶弹出返回地址，即刚刚覆盖的 hello 函数地址，并将该地址赋给 rip
    // 3. CPU 执行 rip寄存器指向的地址，即 hello 函数
    llvm_asm!("
        mov 0x00($0), %rsp
        ret
        " // 第一个 $0，表示一个寄存器
        :
        : "r"(new) // r => register，编译器会自动给 new 分配一个寄存器
        :
        : "alignstack" // 对齐
    );
}

fn main() {
    let mut ctx = RoutineContext::default();
    let mut stack = vec![0_u8; STACK_SIZE as usize];

    unsafe {
        let stack_bottom = stack.as_mut_ptr().offset(STACK_SIZE); // 高地址内存是栈顶
        let stack_aligned = (stack_bottom as usize & !15) as *mut u8; // 16 字节对齐
        std::ptr::write(stack_aligned.offset(-16) as *mut u64, hello as u64); // 从低位向高位写
        ctx.rsp = stack_aligned.offset(-16) as u64; // 从低位向高位读
        ctx_switch(&mut ctx);
    }
}
