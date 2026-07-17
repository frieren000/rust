# 用 Rust 为 QEMU 编写操作系统 — 详细步骤指南

> 目标：从零搭建一个可在 QEMU 中启动的最小 Rust 内核，并逐步扩展到打印、中断、内存管理等基础功能。  
> 适用环境：Linux（本指南以 x86_64 为例）  
> 参考路径：Philipp Oppermann 的 [Writing an OS in Rust](https://os.phil-opp.com/) 思路，并结合现代 `bootloader` / `cargo` 工作流。

---

## 目录

1. [前置知识与目标](#1-前置知识与目标)
2. [环境准备](#2-环境准备)
3. [创建 freestanding Rust 项目](#3-创建-freestanding-rust-项目)
4. [禁用标准库与 panic 处理](#4-禁用标准库与-panic-处理)
5. [自定义目标与链接脚本](#5-自定义目标与链接脚本)
6. [接入引导程序（Bootloader）](#6-接入引导程序bootloader)
7. [在 QEMU 中启动](#7-在-qemu-中启动)
8. [VGA 文本模式打印](#8-vga-文本模式打印)
9. [串口输出（调试利器）](#9-串口输出调试利器)
10. [CPU 异常与中断](#10-cpu-异常与中断)
11. [页表与堆分配器](#11-页表与堆分配器)
12. [建议的后续扩展](#12-建议的后续扩展)
13. [常见问题排查](#13-常见问题排查)
14. [推荐学习资源](#14-推荐学习资源)

---

## 1. 前置知识与目标

### 1.1 你需要大致了解

- Rust 基础语法、`cargo`、`unsafe`、所有权
- 操作系统概念：内核 / 用户态、中断、页表、虚拟内存
- 一点 x86_64 汇编与调用约定会更轻松（非必须）

### 1.2 最终最小目标（MVP）

完成以下能力即算“能跑起来的 OS 内核”：

1. 用 Rust 写出无标准库（`#![no_std]`）内核
2. 通过 bootloader 生成可启动镜像
3. 用 QEMU 启动并看到屏幕/串口输出
4. 正确处理 `panic`
5. （进阶）处理断点异常、建立堆分配

### 1.3 架构选择说明

本指南默认：

| 项目 | 选择 | 原因 |
|------|------|------|
| CPU | x86_64 | 资料最多，QEMU 支持好 |
| 固件 | BIOS 或 UEFI（由 bootloader 处理） | 不必手写 16 位实模式 |
| 启动方式 | `bootloader` crate | 专注内核逻辑，少踩坑 |
| 模拟器 | QEMU | 调试方便、可重定向串口 |

> 若你想从更底层开始（手写 multiboot / 自己写 boot.asm），可在完成本指南后再回头做。

---

## 2. 环境准备

### 2.1 安装 Rust

若尚未安装：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustc --version
cargo --version
```

安装 nightly（很多 OS 开发特性仍依赖 nightly）：

```bash
rustup toolchain install nightly
rustup default nightly   # 也可只在项目里用 +nightly
rustup component add rust-src --toolchain nightly
rustup component add llvm-tools --toolchain nightly
```

安装常用 cargo 工具：

```bash
cargo install bootimage
cargo install cargo-binutils
# 可选：便于查看目标信息
rustup target add x86_64-unknown-none
```

### 2.2 安装 QEMU

Debian / Ubuntu：

```bash
sudo apt update
sudo apt install -y qemu-system-x86 qemu-system-gui
qemu-system-x86_64 --version
```

Fedora：

```bash
sudo dnf install qemu-system-x86
```

Arch：

```bash
sudo pacman -S qemu-system-x86
```

### 2.3 建议的工作目录

```bash
mkdir -p ~/study/rust-os
cd ~/study/rust-os
```

下文假设项目名为 `blog_os`（可自行改名）。

---

## 3. 创建 freestanding Rust 项目

### 3.1 新建 crate

```bash
cargo new --bin blog_os
cd blog_os
```

此时默认是普通用户态程序，依赖 `std`，不能直接当内核用。接下来要改成 **freestanding（裸机）** 程序。

### 3.2 项目结构预览（完成后大致如下）

```text
blog_os/
├── Cargo.toml
├── .cargo/
│   └── config.toml
├── x86_64-blog_os.json          # 自定义目标（可选方案）
├── src/
│   ├── main.rs
│   ├── lib.rs                   # 可选：把内核逻辑放到 lib
│   ├── vga_buffer.rs
│   ├── serial.rs
│   └── interrupts.rs
└── README.md
```

---

## 4. 禁用标准库与 panic 处理

因为内核没有操作系统提供的运行时，必须关闭 `std`。

### 4.1 修改 `src/main.rs`

```rust
#![no_std]  // 不链接标准库
#![no_main] // 不使用常规入口 main（由引导程序跳到我们指定的入口）

use core::panic::PanicInfo;

/// 内核入口。函数名可自定义，但需与 bootloader 约定一致。
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // 暂时死循环，证明能“站住”
    loop {}
}

/// 本环境没有 std 的 panic 处理，必须自己提供
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
```

要点：

- `#![no_std]`：只用 `core`
- `#![no_main]`：没有 Rust 运行时的 `main`
- `_start`：常见裸机入口符号
- `-> !`：永不返回（内核没有“正常退出”）

### 4.2 尝试直接编译会失败（正常）

```bash
cargo build
```

常见错误原因：

1. 目标三元组仍是宿主系统（有 libc）
2. 缺少 panic 策略 / 链接参数
3. 未配置 bootloader

下一步用自定义目标或 `x86_64-unknown-none` 解决。

---

## 5. 自定义目标与链接脚本

有两种主流做法。

### 方案 A：使用 `x86_64-unknown-none`（更简单，推荐起步）

```bash
rustup target add x86_64-unknown-none
```

创建 `.cargo/config.toml`：

```toml
[build]
target = "x86_64-unknown-none"

[target.x86_64-unknown-none]
runner = "bootimage runner"
```

`Cargo.toml` 增加：

```toml
[package]
name = "blog_os"
version = "0.1.0"
edition = "2021"

[dependencies]
bootloader = "0.9"

[package.metadata.bootimage]
# 启动后把 QEMU 串口输出打到终端
run-args = [
    "-serial", "stdio",
    "-display", "gtk"   # 无图形环境可改成 -display none
]
```

> `bootloader 0.9` 与经典教程兼容最好。`bootloader 0.11+` API 有变化（见第 6 节备注）。

### 方案 B：自定义 JSON 目标（更可控）

创建 `x86_64-blog_os.json`：

```json
{
  "llvm-target": "x86_64-unknown-none",
  "data-layout": "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128",
  "arch": "x86_64",
  "target-endian": "little",
  "target-pointer-width": "64",
  "target-c-int-width": "32",
  "os": "none",
  "executables": true,
  "linker-flavor": "ld.lld",
  "linker": "rust-lld",
  "panic-strategy": "abort",
  "disable-redzone": true,
  "features": "-mmx,-sse,+soft-float"
}
```

`.cargo/config.toml`：

```toml
[unstable]
build-std = ["core", "compiler_builtins"]
build-std-features = ["compiler-builtins-mem"]

[build]
target = "x86_64-blog_os.json"

[target.'cfg(target_os = "none")']
runner = "bootimage runner"
```

并确保：

```bash
rustup component add rust-src --toolchain nightly
```

然后用 nightly 构建：

```bash
cargo +nightly build
```

**说明：**

- `disable-redzone`：中断时保护栈上 red zone
- `soft-float` / 关闭 SSE：早期内核中断路径更安全
- `build-std`：为自定义目标编译 `core`

---

## 6. 接入引导程序（Bootloader）

裸机 ELF 不能直接被 BIOS/UEFI 启动，需要引导程序把内核加载到内存、切到长模式，再跳到 `_start`。

### 6.1 使用 `bootloader` 0.9 + `bootimage`（经典路径）

`Cargo.toml`：

```toml
[dependencies]
bootloader = "0.9.23"
```

安装 runner：

```bash
cargo install bootimage
```

构建可启动镜像：

```bash
cargo bootimage
```

成功后会生成类似：

```text
target/x86_64-unknown-none/debug/bootimage-blog_os.bin
```

### 6.2 入口约定

`bootloader 0.9` 默认寻找 `_start`。确保：

```rust
#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {}
}
```

### 6.3 关于 `bootloader` 0.11（可选了解）

新版本改用 workspace / 独立 bootloader 包，入口类似：

```rust
#![no_std]
#![no_main]

use bootloader_api::{entry_point, BootInfo};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    loop {}
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
```

初学者建议先用 **0.9 + bootimage**，跑通后再升级。

---

## 7. 在 QEMU 中启动

### 7.1 一键运行

配置好 `runner` 后：

```bash
cargo run
```

等价于用 QEMU 加载 `bootimage-*.bin`。

### 7.2 手动启动

```bash
qemu-system-x86_64 \
  -drive format=raw,file=target/x86_64-unknown-none/debug/bootimage-blog_os.bin \
  -serial stdio \
  -display gtk
```

无桌面环境（纯 SSH）可用：

```bash
qemu-system-x86_64 \
  -drive format=raw,file=target/x86_64-unknown-none/debug/bootimage-blog_os.bin \
  -serial stdio \
  -display none
```

### 7.3 常用调试参数

```bash
# 启动时停住，等待 GDB
qemu-system-x86_64 \
  -drive format=raw,file=target/x86_64-unknown-none/debug/bootimage-blog_os.bin \
  -serial stdio \
  -s -S
```

另开终端：

```bash
gdb -ex "target remote :1234" \
    -ex "symbol-file target/x86_64-unknown-none/debug/blog_os"
```

### 7.4 成功标志

- QEMU 窗口打开（或 `-display none` 不报错）
- 进程不立即退出
- 之后加打印时能看到字符

此时你已经完成 **“Rust 内核 + Bootloader + QEMU”** 最小闭环。

---

## 8. VGA 文本模式打印

x86 PC 的 VGA 文本缓冲位于物理地址 `0xb8000`。

### 8.1 新建 `src/vga_buffer.rs`

```rust
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[volatile::Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe), // 不可打印字符
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let c = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(c);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

use core::fmt;

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

pub fn print_something() {
    use core::fmt::Write;
    let mut writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };
    writer.write_string("Hello from Rust OS!\n");
    let _ = write!(writer, "The answer is {}", 42);
}
```

依赖 `volatile`（防止编译器优化掉对显存的写）：

```toml
[dependencies]
volatile = "0.2.6"
```

> 注意：`volatile` 0.3+ / 0.4+ API 变化较大，与上面示例匹配的是 **0.2.x**。

### 8.2 在入口调用

```rust
mod vga_buffer;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    vga_buffer::print_something();
    loop {}
}
```

`cargo run` 后应在 QEMU 窗口看到黄色文字。

### 8.3 进一步：实现 `println!` 宏

可参考 phil-opp 教程，用 `lazy_static` + `spin::Mutex` 做成全局 Writer，然后：

```rust
println!("Hello {}", "World");
```

相关依赖示例：

```toml
lazy_static = { version = "1.4", features = ["spin_no_std"] }
spin = "0.5"
```

---

## 9. 串口输出（调试利器）

无图形或自动化测试时，UART COM1（`0x3F8`）非常有用。

### 9.1 依赖

```toml
[dependencies]
uart_16550 = "0.2"
lazy_static = { version = "1.4", features = ["spin_no_std"] }
spin = "0.5"
```

### 9.2 `src/serial.rs`

```rust
use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    SERIAL1.lock().write_fmt(args).expect("Printing to serial failed");
}
```

### 9.3 QEMU 参数

确保：

```text
-serial stdio
```

这样 `serial_println!("boot ok")` 会直接出现在宿主机终端。

---

## 10. CPU 异常与中断

### 10.1 目标

- 加载 IDT（中断描述符表）
- 处理断点异常（`int3`）
- 之后扩展到时钟中断、键盘中断

### 10.2 依赖

```toml
[dependencies]
x86_64 = "0.14"
```

### 10.3 最小 IDT 示例思路

```rust
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    serial_println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}
```

需要开启：

```rust
#![feature(abi_x86_interrupt)]
```

并在 `_start` 中：

```rust
init_idt();
x86_64::instructions::interrupts::int3(); // 触发断点，验证 handler
```

若串口打印出异常帧，说明异常路径工作正常。

### 10.4 双故障与 IST

处理 page fault / double fault 时，需要单独的中断栈（TSS + IST），否则内核栈溢出可能无法恢复。这是下一阶段必做项。

---

## 11. 页表与堆分配器

### 11.1 BootInfo

`bootloader` 会传入内存映射等信息（0.9 通过约定 / 某些版本用入口参数）。拿到物理内存区域后：

1. 建立 offset page table
2. 实现 frame allocator
3. 映射堆区域
4. 接入 `linked_list_allocator` 或 `buddy_system_allocator`

### 11.2 堆初始化示意

```toml
[dependencies]
linked_list_allocator = "0.9"
```

```rust
use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    // 1. 映射 HEAP_START .. HEAP_START+HEAP_SIZE
    // 2. unsafe { ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE); }
    Ok(())
}
```

之后即可在内核中使用 `alloc::vec::Vec` 等（仍需 `#![no_std]` + `extern crate alloc`）。

---

## 12. 建议的后续扩展

按推荐顺序推进：

| 阶段 | 内容 | 产出 |
|------|------|------|
| 1 | 本指南 MVP | QEMU 启动 + VGA/串口 |
| 2 | IDT / 异常 | 断点、双重故障可处理 |
| 3 | PIC/APIC + 键盘 | 能读键入 |
| 4 | 分页与堆 | `Box`/`Vec` 可用 |
| 5 | 多任务 | 协作/抢占式调度 |
| 6 | 文件系统 | 读 ramdisk / virtio-blk |
| 7 | 用户态 | ring3、系统调用 |
| 8 | 进程与 ELF 加载 | 跑起第一个用户程序 |

每完成一阶段，建议：

1. 写集成测试（QEMU + 串口退出码）
2. 在 README 记录启动命令与已知限制
3. 提交 git（便于回滚）

### 集成测试思路

在测试失败/成功时向 QEMU `isa-debug-exit` 设备写端口退出：

```text
-device isa-debug-exit,iobase=0xf4,iosize=0x04
```

成功退出码约定为特定值，方便 CI。

---

## 13. 常见问题排查

### 13.1 `cargo build` 报找不到 `std`

- 确认 `#![no_std]`
- 确认 target 为 `x86_64-unknown-none` 或自定义 none 目标
- 不要在内核里 `use std::...`

### 13.2 `bootimage` 找不到 / runner 失败

```bash
cargo install bootimage
which bootimage
# 确保 ~/.cargo/bin 在 PATH
```

### 13.3 QEMU 黑屏无输出

1. 先用 `serial_println!` + `-serial stdio` 确认内核是否执行
2. 检查是否写到 `0xb8000`
3. 确认没有立刻 triple fault 重启（QEMU 会表现为反复重启）

### 13.4 一启动就重启（triple fault）

常见原因：

- IDT 未正确加载就开中断
- 栈溢出
- 错误的页表映射
- 未禁用 red zone 却在中断中使用

可用：

```bash
qemu-system-x86_64 ... -d int,cpu_reset -D qemu.log
```

查看复位原因。

### 13.5 `volatile` / `x86_64` 版本不兼容

锁版本到教程匹配组合，例如：

```toml
bootloader = "0.9.23"
volatile = "0.2.6"
x86_64 = "0.14.10"
uart_16550 = "0.2.0"
spin = "0.5.2"
lazy_static = { version = "1.4.0", features = ["spin_no_std"] }
```

### 13.6 无图形环境

```bash
qemu-system-x86_64 ... -display none -serial stdio
```

或使用 VNC：

```bash
-vnc :1
```

---

## 14. 推荐学习资源

1. **Writing an OS in Rust**（强烈推荐按章节跟做）  
   https://os.phil-opp.com/

2. **OSDev Wiki**（硬件细节百科）  
   https://wiki.osdev.org/

3. **Intel SDM / AMD64 手册**（查异常、页表、MSR）  
   https://www.intel.com/sdm

4. **QEMU 文档**  
   https://www.qemu.org/docs/master/

5. **crate 文档**  
   - https://docs.rs/bootloader  
   - https://docs.rs/x86_64  
   - https://docs.rs/uart_16550  

---

## 附录 A：从零到第一次启动的命令清单

```bash
# 0. 工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup toolchain install nightly
rustup component add rust-src llvm-tools --toolchain nightly
rustup target add x86_64-unknown-none
cargo install bootimage cargo-binutils
sudo apt install -y qemu-system-x86   # 按发行版调整

# 1. 项目
cargo new --bin blog_os
cd blog_os

# 2. 按本指南修改：
#    - src/main.rs（no_std / no_main / _start / panic_handler）
#    - Cargo.toml（bootloader 等依赖）
#    - .cargo/config.toml（target + runner）

# 3. 构建并运行
cargo bootimage
cargo run
```

---

## 附录 B：最小 `Cargo.toml` 模板

```toml
[package]
name = "blog_os"
version = "0.1.0"
edition = "2021"

[dependencies]
bootloader = "0.9.23"
volatile = "0.2.6"
spin = "0.5.2"
x86_64 = "0.14.10"
uart_16550 = "0.2.0"
lazy_static = { version = "1.4.0", features = ["spin_no_std"] }

[package.metadata.bootimage]
run-args = ["-serial", "stdio"]
test-args = [
    "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
    "-serial", "stdio",
    "-display", "none"
]
test-success-exit-code = 33
```

---

## 附录 C：学习节奏建议

| 天数 | 任务 |
|------|------|
| Day 1 | 环境 + freestanding + QEMU 黑屏死循环 |
| Day 2 | VGA `Hello` + `println!` |
| Day 3 | 串口 + panic 打印 |
| Day 4 | IDT + breakpoint |
| Day 5 | Double fault + TSS |
| Day 6–7 | 页表遍历与映射 |
| Day 8+ | 堆、键盘、调度…… |

---

## 结语

用 Rust 写 QEMU 操作系统，关键路径是：

**`#![no_std]` 内核 → bootloader 生成镜像 → QEMU 启动 → VGA/串口确认执行 → 异常与内存管理逐步扩展。**

先追求“能启动、能打印、能稳定不重启”，再追求功能完备。每一步都用串口日志和 QEMU 调试参数验证，会少走很多弯路。

若你希望下一步直接在 `~/study/rust` 下生成可运行的骨架工程，可以继续说明偏好（bootloader 0.9 经典版 / 0.11 新版，是否需要中文注释代码）。
