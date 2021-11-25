#![feature(llvm_asm, naked_functions)]

const DEFAULT_STACK_SIZE: usize = 1024 * 1024 * 2;
const MAX_ROUTINES: usize = 10;

// 全局运行时实例
static mut RUNTIME: usize = 0;

#[derive(Debug, Default)]
#[repr(C)]
struct Context {
    rsp: u64, // rsp 寄存器
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    rbx: u64,
    rbp: u64,
}

#[naked]
unsafe fn ctx_switch() {
    // 注意：16进制
    llvm_asm!(
        "
        mov     %rsp, 0x00(%rdi)
        mov     %r15, 0x08(%rdi)
        mov     %r14, 0x10(%rdi)
        mov     %r13, 0x18(%rdi)
        mov     %r12, 0x20(%rdi)
        mov     %rbx, 0x28(%rdi)
        mov     %rbp, 0x30(%rdi)

        mov     0x00(%rsi), %rsp
        mov     0x08(%rsi), %r15
        mov     0x10(%rsi), %r14
        mov     0x18(%rsi), %r13
        mov     0x20(%rsi), %r12
        mov     0x28(%rsi), %rbx
        mov     0x30(%rsi), %rbp
        "
    );
}

#[derive(Debug, PartialEq, Eq)]
enum State {
    Available, // 可用
    Running,   // 正在运行
    Ready,     // 可运行
}

#[derive(Debug)]
struct Routine {
    id: usize,
    stack: Vec<u8>,
    state: State,
    ctx: Context,
}

impl Routine {
    fn new(id: usize) -> Self {
        Self {
            id,
            stack: vec![0_u8; DEFAULT_STACK_SIZE],
            state: State::Available,
            ctx: Context::default(),
        }
    }
}

#[derive(Debug)]
pub struct Runtime {
    current: usize,
    routines: Vec<Routine>,
}

impl Runtime {
    pub fn new() -> Self {
        let current_id = 0;
        let mut initial_routine = Routine::new(current_id);
        initial_routine.state = State::Running;
        let mut routines = vec![initial_routine];

        let mut available_routines: Vec<Routine> =
            (1..MAX_ROUTINES).map(|id| Routine::new(id)).collect();
        routines.append(&mut available_routines);
        Self {
            current: current_id,
            routines: routines,
        }
    }

    pub fn init(&self) {
        unsafe {
            let ptr: *const Runtime = self;
            RUNTIME = ptr as usize;
        }
    }

    pub fn r#yield(&mut self) -> bool {
        // 找到一个 ready 的，然后让其运行
        let mut pos = self.current;
        while self.routines[pos].state != State::Ready {
            pos += 1;
            if pos == self.routines.len() {
                pos = 0;
            }
            if pos == self.current {
                // 找到了自己，证明没有其它人运行了，所以退出
                return false;
            }
        }

        if self.routines[self.current].state != State::Available {
            self.routines[self.current].state = State::Ready;
        }

        self.routines[pos].state = State::Running;
        let old_pos = self.current;
        self.current = pos;

        unsafe {
            let old: *mut Context = &mut self.routines[old_pos].ctx;
            let new: *const Context = &self.routines[pos].ctx;
            llvm_asm!(
                "mov $0, %rdi
                 mov $1, %rsi"::"r"(old), "r"(new)
            );
            ctx_switch();
        }
        self.routines.len() > 0
    }

    pub fn r#return(&mut self) {
        if self.current != 0 {
            self.routines[self.current].state = State::Available;
            self.r#yield();
        }
    }

    // +----------+ High
    // |----------| 16 aligned
    // |   guard  |
    // |   hello  |     
    // |     f    | <- rsp
    // |          |
    // |          |
    // +----------+ Low
    pub fn spawn(&mut self, f: fn()) {
        // 找到一个可用的
        let avaliable = self
            .routines
            .iter_mut()
            .find(|r| r.state == State::Available)
            .expect("no match routine");
        let sz = avaliable.stack.len();
        unsafe {
            let stack_bottom = avaliable.stack.as_mut_ptr().offset(sz as isize); // 高地址内存是栈顶
            // https://stackoverflow.com/questions/10224564/what-does-alignment-to-16-byte-boundary-mean-in-x86
            // 0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_1111
            // 1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_0000
            //&xxxx_xxxx_xxxx_xxxx_xxxx_xxxx_xxxx_xxxx_xxxx_xxxx_xxxx_xxxx_xxxx_xxxx_xxxx_xxxx 
            //&xxxx_xxxx_xxxx_xxxx_xxxx_xxxx_xxxx_xxxx_xxxx_xxxx_xxxx_xxxx_xxxx_xxxx_xxxx_0000 => 恰好是16的倍数 
            let stack_aligned = (stack_bottom as usize & !15) as *mut u8;
            std::ptr::write(stack_aligned.offset(-16) as *mut u64, guard as u64);
            std::ptr::write(stack_aligned.offset(-24) as *mut u64, hello as u64);
            std::ptr::write(stack_aligned.offset(-32) as *mut u64, f as u64);
            avaliable.ctx.rsp = stack_aligned.offset(-32) as u64; // 16 字节对齐
        }
        avaliable.state = State::Ready;
    }

    pub fn run(&mut self) {
        while self.r#yield() {}
        std::process::exit(0);
    }
}

fn hello() {
    println!("hello routine");
}

fn guard() {
    unsafe {
        let ptr = RUNTIME as *mut Runtime;
        (*ptr).r#return();
    }
}

pub fn yield_routine() {
    unsafe {
        let ptr = RUNTIME as *mut Runtime;
        (*ptr).r#yield();
    }
}

fn main() {
    let mut rt = Runtime::new();
    rt.init();
    rt.spawn(|| {
        println!("1 STARTING");
        let id = 1;
        for i in 0..10 {
            println!("routine: {} counter: {}", id, i);
            yield_routine();
        }
        println!("1 FINISHED");
    });
    rt.spawn(|| {
        println!("2 STARTING");
        let id = 2;
        for i in 0..15 {
            println!("routine: {} counter: {}", id, i);
            yield_routine();
        }
        println!("2 FINISHED");
    });
    rt.run();
}
