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
| [RustyHermit](https://github.com/hermitcore/rusty-hermit)             | Unikernel (libOS)        |          yup                                     | No (but support for virtio-net)
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

(The following instructions worked in a `debian:12` Docker container.)

requirements:
```sh
sudo apt-get install -y build-essential curl git libssl-dev ninja-build \
  pkg-config python3 qemu-system-arm
curl https://sh.rustup.rs -sSf | sh
source "$HOME/.cargo/env"
rustup default nightly-2023-05-01
rustup component add rust-src
rustup component add llvm-tools-preview
cargo install uhyve --locked
cargo install cargo-download
```
To build and run a hello-world rust (with std) application:
```sh
cargo new hello_world
cd hello_world
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
hermit-sys = { version = "0.5.1" , default-features = false }

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
cargo xtask build --target aarch64 --release
```

Now to run the application with QEMU:
```sh
qemu-system-aarch64 \
                  -machine virt,gic-version=3 \
                  -cpu cortex-a76 -smp 1 -m 512M  \
                  -semihosting -L /usr/share/qemu \
                  -display none -serial stdio \
                  -kernel target/aarch64/release/rusty-loader \
                  -device guest-loader,addr=0x48000000,initrd=../hello_world/target/aarch64-unknown-hermit/debug/hello_world
```

> output
```sh
[LOADER][INFO] Loader: [0x40200000 - 0x4021f000]
[LOADER][INFO] Found ELF file with size 20629232
[LOADER][INFO] Parsing kernel from ELF at 0x48000000..0x493ac6f0 (20629232 B)
[LOADER][INFO] Loading kernel to 0x40400000
[LOADER][INFO] TLS is at 0x4054a678..0x4054a6f2 (122 B)
[LOADER][INFO] Detect 1 CPU(s)
[LOADER][INFO] Detect UART at 0x9000000
[LOADER][INFO] Jumping to HermitCore Application Entry Point at 0x40539f10
[0][INFO] Welcome to HermitCore-rs 0.6.1
[0][INFO] Kernel starts at 0x40400000
[0][INFO] BSS starts at 0x4056eae8
[0][INFO] TLS starts at 0x4054a678 (size 122 Bytes)
[0][INFO] RAM starts at physical address 0x40000000
[0][INFO] Physical address range: 1024GB
[0][INFO] Support of 4KB pages: true
[0][INFO] Support of 16KB pages: true
[0][INFO] Support of 64KB pages: true
[0][INFO] Total memory size: 506 MB
[0][INFO] Kernel region: [0x40400000 - 0x40600000]
[0][INFO] A pure Rust application is running on top of HermitCore!
[0][INFO] Heap: size 446 MB, start address 0x200000
[0][INFO] Heap is located at 0x200000..0x1c000000 (0 Bytes unmapped)
[0][INFO] 
[0][INFO] ===================== PHYSICAL MEMORY FREE LIST ======================
[0][INFO] 0x0000005C4DF000 - 0x00000060000000
[0][INFO] ======================================================================
[0][INFO] 
[0][INFO] 
[0][INFO] ================== KERNEL VIRTUAL MEMORY FREE LIST ===================
[0][INFO] 0x00000000002000 - 0x00000000200000
[0][INFO] 0x0000001C000000 - 0x00000040000000
[0][INFO] 0x00000040600000 - 0x00000100000000
[0][INFO] ======================================================================
[0][INFO] 
[0][INFO] Intialize generic interrupt controller
[0][INFO] Found GIC Distributor interface at 0x8000000 (size 0x10000)
[0][INFO] Found generic interrupt controller at 0x80A0000 (size 0xF60000)
[0][INFO] Timer interrupt: 14
[0][INFO] 
[0][INFO] ========================== CPU INFORMATION ===========================
[0][INFO] Processor compatiblity:  arm,cortex-a76
[0][INFO] Counter frequency:       62500000 Hz (from CNTFRQ_EL0)
[0][INFO] ======================================================================
[0][INFO] 
[0][INFO] HermitCore-rs booted on 2023-06-29 14:15:35.0 +00:00:00
[0][INFO] Compiled with PCI support
[0][INFO] Compiled with ACPI support
[0][INFO] HermitCore is running on common system!
[0][WARN] Unable to read entropy! Fallback to a naive implementation!
Hello World!
[0][INFO] Shutting down system
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

RustyHermit supports [virtio-net](https://www.redhat.com/en/blog/introduction-virtio-networking-and-vhost-net) atm.

### [Unikraft](https://github.com/unikraft/unikraft)
- a unikernel dev kit
- rust support only for x86 (and only supports no_std), no rust for aarch64.
- they just added support for vsock.

TO-DO: still trying to test unikraft
