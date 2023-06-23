## CCA realmOS

Investigate possible low-TCB realmOS to run veracruz on, on top of CCA.
There are few options:
- we are started by looking for an OS based on seL4:
   - [IceCap](https://gitlab.com/icecap-project/icecap): we already have support for IceCap.
   - [CantripOS](https://github.com/AmbiML/sparrow-cantrip-full/tree/main): cantrip doesn't support Rust std applications atm, and doesn't have support for vsock.

A summary: 

| Name                                                                  |   Type                   |   Rust std support                               |   Vsock
|-----------------------------------------------------------------------|--------------------------|--------------------------------------------------|---------
| [CantripOS](https://github.com/AmbiML/sparrow-cantrip-full/tree/main) |Low_RCB OS (based on seL4)| only no_std                                      |  No
| [Redox](https://gitlab.redox-os.org/redox-os/redox)                   | Microkernel              |          yup                                     |  No
| [RustyHermit](https://github.com/hermitcore/rusty-hermit)             | Unikernel (libOS)        |          yup                                     |  No
| [Unikraft](https://github.com/unikraft/unikraft)                      | a unikernel dev kit      | no rust support for aarch64, for x86 only no_std |  Recently added  


### [Redox](https://gitlab.redox-os.org/redox-os/redox)
a Microkernel written in Rust. it has support for Rust std on aarch64, no vsock support.
        - support for aarch64, builds with no errors but I encountered some runtime errors (it's been reported in there gitlab just few weeks ago) 
https://gitlab.redox-os.org/redox-os/redox/-/issues/1376
        - Their repo seems healthy, fairly active with few contributors.

To get started: 

(The following instructions also worked in a `debian:11` Docker container
with `--device /dev/fuse --cap-add SYS_ADMIN --privileged`.)

```sh
curl -sf https://gitlab.redox-os.org/redox-os/redox/raw/master/bootstrap.sh -o bootstrap.sh
time bash -e bootstrap.sh
source ~/.cargo/env
sudo apt-get install qemu-system-aarch64
sudo apt-get install u-boot-tools
sudo apt-get install qemu-system-arm qemu-efi
sudo apt-get install fuse
cd redox
./build.sh -a aarch64 -c server qemu vga=no
```
> to change the configuration, go to `config/aarch64/server.toml`

at this point I got the following error: 
```sh
TRACE: fffffe8000258120
  FP fffffe8000258120: PC ffffff000007d51c
  FP fffffe8000258260: PC ffffff0000001b64
  FP fffffe80002582c0: PC 00000000f2000001
  00007fffffffe9f8: GUARD PAGE
kernel:INFO -- SIGNAL 11, CPU 0, PID ContextId(17)
kernel:INFO -- NAME /bin/pcid
redoxfs: found scheme disk/live
redoxfs: found path disk/live:/0
redoxfs: opening disk/live:/0
redoxfs: opened filesystem on disk/live:/0 with uuid dcbb6e58-3ebe-4ebe-83d4-378550da9d28
redoxfs: filesystem on disk/live:/0 matches uuid dcbb6e58-3ebe-4ebe-83d4-378550da9d28
redoxfs: mounted filesystem on disk/live:/0 to file:
init: failed to execute 'pcid /etc/pcid.d/': No such file or directory (os error 2)
init: failed to execute 'escalated': No such file or directory (os error 2)
smolnetd: smoltcpd: failed to open network:: syscall error: No such device
thread 'main' panicked at 'smoltcp: failed to daemonize: I/O error', src/smolnetd/main.rs:163:8
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
dnsd: dnsd: failed to open nameserver:: syscall error: No such device
thread 'main' panicked at 'dnsd: failed to daemonize: I/O error', src/dnsd/main.rs:94:8
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
Running with true and false
dhcpd: Can't open netcfg:ifaces/eth0/mac
error: getty: failed to open TTY display:2/activate: No such device
########## Redox OS ##########
# Login with the following:  #
# `user`                     #
# `root`:`password`          #
##############################
 
redox login: user
Welcome to Redox OS!
 
ion: creating history file at "file:/home/user/.local/share/ion/history"
ion: prompt expansion failed: pipeline execution error: command exec error: nul byte found in provided data
>>> 
ion: prompt expansion failed: pipeline execution error: command exec error: nul byte found in provided data
>>> 
```

Clean previous build:
```sh
rm -rf prefix/aarch64-unknown-redox/relibc-install/ cookbook/recipes/gcc/{build,sysroot,stage*} build/aarch64/*/{harddrive.img,livedisk.iso}
```


### [RustyHermit](https://github.com/hermitcore/rusty-hermit)

- A Rust based, lightwight unikernel, it can run Rust applications (with std), as well as C/C++/Go/Fortran applications. no vsock support atm. Rust applications that do not bypass the Rust runtime and directly use OS services are able to run on RustyHermit without modifications.
- In their repo they don't mention support for aarch64 for some reason adn they don't have getting started instructions for `aarch64`, but they do support different architectures. (maybe outdated documentation?)
The setup is simple, create you own rust hello_world application and add the following changes:

requirements:
```sh
cargo +nightly install uhyve --locked
cargo install cargo-download
rustup component add rust-src
rustup component add llvm-tools-preview
```
To build and run a hello-world rust (with std) application:
```sh
cargo new hello_world
cd hello_world
rustup default nightly
```

add the following changes to `src/main.rs`:
```sh
#[cfg(target_os = "hermit")]
use hermit_sys as _;

fn main() {
    println!("Hello, world!");
}
```
and these changes to `Cargo.toml`:
```sh
[target.'cfg(target_os = "hermit")'.dependencies]
hermit-sys = { version = "0.4.1" , default-features = false }

[features]
default = ["pci", "acpi"]
vga = ["hermit-sys/vga"]
pci = ["hermit-sys/pci"]
pci-ids = ["hermit-sys/pci-ids"]
acpi = ["hermit-sys/acpi"]
fsgsbase = ["hermit-sys/fsgsbase"]
smp = ["hermit-sys/smp"]
instrument = ["hermit-sys/instrument"]
```
To Build the application:
```sh
cargo build -Z build-std=std,core,alloc,panic_abort -Z build-std-features=compiler-builtins-mem --target aarch64-unknown-hermit
```

To run the application with QEMU, we need a bootloader, rustyHermit provides a rusty-loader, to build it:
```sh
git clone https://github.com/hermitcore/rusty-loader.git
cd rusty-loader
cargo xtask build --target aarch64
```

Now to run the application with QEMU:
```sh
qemu-system-aarch64 \
                  -machine virt,gic-version=3 \
                  -cpu cortex-a76 -smp 1 -m 512M  \
                  -semihosting -L /usr/share/qemu \
                  -display none -serial stdio \
                  -kernel target/aarch64/release/rusty-loader \
                  -device guest-loader,addr=0x48000000,initrd=../rusty-hermit/target/aarch64-unknown-hermit/debug/hello_world
```


RustyHermit can either use Qemu to run (in this case we need rusty-loader) or they have their own minimal hypervisor called uhyve, but there is only x86 support for uhyve ( and it doesn't work ..)

To Build and run a hello-world application for x86:
```sh
cargo new hello_world
cd hello_world
rustup default nightly
rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
PATH=${HOME}/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin:$PATH cargo build -Z build-std=std,core,alloc,panic_abort --target x86_64-unknown-hermit
cargo build -Z build-std=std,core,alloc,panic_abort -Z build-std-features=compiler-builtins-mem --target x86_64-unknown-hermit
```
RustyHermit can either use Qemu to run (in this case we need rusty-loader) or they have their own minimal hypervisor called uhyve, but there is only x86 support for uhyve ( and it doesn't work ..)
To Build for x86 and test in uhyve
```sh
uhyve target/x86_64-unknown-hermit/debug/hello_world
```

To run the application with QEMU, we need to a kernel/bootloader, rustyHermit provides a rusty-loader, to build it:
```sh
git clone https://github.com/hermitcore/rusty-loader.git
cd rusty-loader
cargo xtask build --target x86_64
```

Now to run the application with QEMU:
```sh
qemu-system-x86_64 -display none -smp 1 -m 64M -serial stdio -L /usr/share/qemu -kernel target/x86_64/debug/rusty-loader -initrd ../rusty-hermit/target/x86_64-unknown-hermit/debug/hello_world -cpu qemu64,apic,fsgsbase,rdtscp,xsave,fxsr -device isa-debug-exit,iobase=0xf4,iosize=0x04 -enable-kvm
```
> Output:
```sh
[LOADER][INFO] Loader: [0x100000 - 0x12c018]
[LOADER][INFO] Found Multiboot information at 0x9500
[LOADER][WARN] Mapping 1 4KiB pages from 0x9000..0xa000 to 0x9000..0xa000
[LOADER][WARN] Mapping 1 4KiB pages from 0x12e000..0x12f000 to 0x12e000..0x12f000
[LOADER][INFO] Found module: [0x12e000 - 0x19d3d80]
[LOADER][INFO] Module length: 0x18a5d80
[LOADER][INFO] Found an ELF module at 0x12e000
[LOADER][WARN] Mapping 209 4KiB pages from 0x12f000..0x200000 to 0x12f000..0x200000
[LOADER][WARN] Mapping 12 2MiB pages from 0x200000..0x1a00000 to 0x200000..0x1a00000
[LOADER][INFO] Parsing kernel from ELF at 0x12e000..0x19d3d80 (25845120 B)
[LOADER][WARN] Mapping 2 2MiB pages from 0x1a00000..0x1e00000 to 0x1a00000..0x1e00000
[LOADER][INFO] Loading kernel to 0x1a00000
[LOADER][INFO] TLS is at 0x1c0f3e0..0x1c0f45a (122 B)
[LOADER][INFO] Use stack address 0xa000
[LOADER][WARN] Mapping 8 4KiB pages from 0xa000..0x12000 to 0xa000..0x12000
[LOADER][INFO] BootInfo located at 0x12b010
[LOADER][INFO] Jumping to HermitCore Application Entry Point at 0x1b44b30
[0][INFO] Welcome to HermitCore-rs 0.6.1
[0][INFO] Kernel starts at 0x1a00000
[0][INFO] BSS starts at 0x1c33258
[0][INFO] TLS starts at 0x1c0f3e0 (size 122 Bytes)
[0][INFO] Total memory size: 63 MB
[0][INFO] Kernel region: [0x1a00000 - 0x1e00000]
[0][INFO] A pure Rust application is running on top of HermitCore!
[0][INFO] Heap: size 26 MB, start address 0x1e00000
[0][INFO] Heap is located at 0x1e00000..0x3800000 (0 Bytes unmapped)
[0][INFO] 
[0][INFO] ===================== PHYSICAL MEMORY FREE LIST ======================
[0][INFO] 0x00000003800000 - 0x00000003FE0000
[0][INFO] ======================================================================
[0][INFO] 
[0][INFO] 
[0][INFO] ================== KERNEL VIRTUAL MEMORY FREE LIST ===================
[0][INFO] 0x00000003800000 - 0x00800000000000
[0][INFO] ======================================================================
[0][INFO] 
[0][INFO] 
[0][INFO] ========================== CPU INFORMATION ===========================
[0][INFO] Model:                   QEMU Virtual CPU version 2.5+
[0][INFO] Frequency:               3191 MHz (from Measurement)
[0][INFO] SpeedStep Technology:    Not Available
[0][INFO] Features:                MMX SSE SSE2 SSE3 MCE FXSR XSAVE RDTSCP CLFLUSH X2APIC HYPERVISOR FSGSBASE 
[0][INFO] Physical Address Width:  40 bits
[0][INFO] Linear Address Width:    48 bits
[0][INFO] Supports 1GiB Pages:     No
[0][INFO] ======================================================================
[0][INFO] 
[0][INFO] HermitCore-rs booted on 2023-06-21 15:31:33.443215 +00:00:00
[0][WARN] PCI Device @8086:7000 has multiple functions! Currently only one is handled.
[0][INFO] 
[0][INFO] ======================== PCI BUS INFORMATION =========================
[0][INFO] 00:00 Unknown Class [0600]: Unknown Vendor Unknown Device [8086:1237]
[0][INFO] 00:01 Unknown Class [0601]: Unknown Vendor Unknown Device [8086:7000]
[0][INFO] 00:02 Unknown Class [0300]: Unknown Vendor Unknown Device [1234:1111], MemoryBar: 0xfd000000 (size 0x1000000), MemoryBar: 0xfebf0000 (size 0x1000)
[0][INFO] 00:03 Unknown Class [0200]: Unknown Vendor Unknown Device [8086:100E], IRQ 11, MemoryBar: 0xfebc0000 (size 0x20000), IOBar: 0xc000 (size 0x40)
[0][INFO] ======================================================================
[0][INFO] 
[0][INFO] Found an ACPI revision 0 table at 0xF5AE0 with OEM ID "BOCHS "
[0][INFO] IOAPIC v17 has 24 entries
[0][INFO] Disable IOAPIC timer
[0][INFO] 
[0][INFO] ===================== MULTIPROCESSOR INFORMATION =====================
[0][INFO] APIC in use:             x2APIC
[0][INFO] Initialized CPUs:        1
[0][INFO] ======================================================================
[0][INFO] 
[0][INFO] Compiled with PCI support
[0][INFO] Compiled with ACPI support
[0][INFO] HermitCore is running on common system!
[0][WARN] Unable to read entropy! Fallback to a naive implementation!
Hello World!
[0][INFO] Number of interrupts
[0][INFO] [0][7]: 1
[0][INFO] Shutting down system
```



### [Unikraft](https://github.com/unikraft/unikraft)
- a unikernel dev kit
- rust support only for x86 (and only supports no_std), no rust for aarch64.
- they just added support for vsock.

TO-DO: still trying to test unikraft
