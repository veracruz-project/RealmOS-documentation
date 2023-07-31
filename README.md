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
```
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

RustyHermit supports [virtio-net](https://www.redhat.com/en/blog/introduction-virtio-networking-and-vhost-net) atm:

To test `virtio-net`:

To Build the application:
```sh
cd httpd
HERMIT_IP="10.0.2.15" HERMIT_GATEWAY="10.0.2.2" cargo build -Z build-std=std,core,alloc,panic_abort -Z build-std-features=compiler-builtins-mem --target aarch64-unknown-hermit
```

To run the application with QEMU, we need a bootloader, rustyHermit provides a rusty-loader, to build it:
```sh
git clone https://github.com/hermitcore/rusty-loader.git
cd rusty-loader
cargo xtask build --target aarch64 --release
```

Now to run the application with QEMU: (make sure the paths to rusty-loader and the application are correct)

```sh
qemu-system-aarch64 \
                  -machine virt,gic-version=3 \
                  -cpu cortex-a72 -smp 1 -m 512M  \
                  -semihosting -L /usr/share/qemu \
                  -display none -serial stdio \
                  -kernel target/aarch64/release/rusty-loader \
                  -device guest-loader,addr=0x48000000,initrd=../RealmOS-documentation/httpd/target/aarch64-unknown-hermit/debug/httpd \
                  -netdev user,id=u1,hostfwd=tcp::8080-:8080 \
                  -device virtio-net-pci,netdev=u1,disable-legacy=on 
```

> output

```
[LOADER][INFO] Loader: [0x40200000 - 0x4021f000]
[LOADER][INFO] Found ELF file with size 35982672
[LOADER][INFO] Parsing kernel from ELF at 0x48000000..0x4a250d50 (35982672 B)
[LOADER][INFO] Loading kernel to 0x40400000
[LOADER][INFO] TLS is at 0x406df840..0x406df8d2 (146 B)
[LOADER][INFO] Detect 1 CPU(s)
[LOADER][INFO] Detect UART at 0x9000000
[LOADER][INFO] Jumping to HermitCore Application Entry Point at 0x406cf6c8
[0][INFO] Welcome to HermitCore-rs 0.6.2
[0][INFO] Kernel starts at 0x40400000
[0][INFO] BSS starts at 0x40717b90
[0][INFO] TLS starts at 0x406df840 (size 146 Bytes)
[0][INFO] RAM starts at physical address 0x40000000
[0][INFO] Physical address range: 16384GB
[0][INFO] Support of 4KB pages: true
[0][INFO] Support of 16KB pages: false
[0][INFO] Support of 64KB pages: true
[0][INFO] Total memory size: 504 MB
[0][INFO] Kernel region: [0x40400000 - 0x40800000]
[0][INFO] A pure Rust application is running on top of HermitCore!
[0][INFO] Heap: size 442 MB, start address 0x200000
[0][INFO] Heap is located at 0x200000..0x1bc00000 (0 Bytes unmapped)
[0][INFO] 
[0][INFO] ===================== PHYSICAL MEMORY FREE LIST ======================
[0][INFO] 0x0000005C2DD000 - 0x00000060000000
[0][INFO] ======================================================================
[0][INFO] 
[0][INFO] 
[0][INFO] ================== KERNEL VIRTUAL MEMORY FREE LIST ===================
[0][INFO] 0x00000000002000 - 0x00000000200000
[0][INFO] 0x0000001BC00000 - 0x00000040000000
[0][INFO] 0x00000040800000 - 0x00000100000000
[0][INFO] ======================================================================
[0][INFO] 
[0][INFO] Intialize generic interrupt controller
[0][INFO] Found GIC Distributor interface at 0x8000000 (size 0x10000)
[0][INFO] Found generic interrupt controller at 0x80A0000 (size 0xF60000)
[0][INFO] 
[0][INFO] ========================== CPU INFORMATION ===========================
[0][INFO] Processor compatiblity:  arm,cortex-a72
[0][INFO] Counter frequency:       62500000 Hz (from CNTFRQ_EL0)
[0][INFO] Run on hypervisor
[0][INFO] ======================================================================
[0][INFO] 
[0][INFO] HermitCore-rs booted on 2023-07-31 14:54:33.0 +00:00:00
[0][INFO] Mapping PCI Enhanced Configuration Space interface to virtual address 0x20000000 (size 0x10000000)
[0][INFO] Scanning PCI Busses 0 to 255
[0][INFO] Compiled with PCI support
[0][INFO] 
[0][INFO] ======================== PCI BUS INFORMATION =========================
[0][INFO] 00:00 Unknown Class [0600]: Unknown Vendor Unknown Device [1B36:0008]
[0][INFO] 00:01 Unknown Class [0200]: Unknown Vendor Unknown Device [1AF4:1041], IRQ 4, BAR1 Memory32 { address: 0x0, size: 0x1000, prefetchable: false }, BAR4 Memory64 { address: 0x8000000000, size: 0x4000, prefetchable: true }
[0][INFO] ======================================================================
[0][INFO] 
[0][INFO] HermitCore is running on common system!
[0][INFO] Found virtio network device with device id 0x1041
[0][WARN] Currently only mapping of 64 bit BAR's is supported!
[0][WARN] Currently only mapping of 64 bit BAR's is supported!
[0][INFO] Non Virtio PCI capability with id 11 found. And NOT used.
[0][ERROR] Found virtio capability whose BAR is not mapped or non existing. Capability of type 5 and id 0 for device 1041, can not be used!
[0][INFO] Feature set wanted by network driver are in conformance with specification.
[0][INFO] Feature set wanted by network driver are in conformance with specification.
[0][INFO] Driver found a subset of features for virtio device 1041. Features are: [VIRTIO_NET_F_MAC, VIRTIO_NET_F_STATUS, VIRTIO_F_RING_INDIRECT_DESC, VIRTIO_F_VERSION_1]
[0][INFO] Features have been negotiated between virtio network device 1041 and driver.
[0][INFO] Created SplitVq: idx=0, size=256
[0][INFO] Created SplitVq: idx=1, size=256
[0][INFO] Network driver successfully initialized virtqueues.
[0][INFO] Device specific initialization for Virtio network device 1041 finished
[0][INFO] Network device with id 1041, has been initialized by driver!
[0][INFO] Virtio-net link is up after initialization.
[0][INFO] Virtio network driver initialized.
[0][INFO] Install virtio interrupt handler at line 4
[0][INFO] Try to nitialize network!
[0][INFO] MAC address 52-54-00-12-34-56
[0][INFO] Configure network interface with address 10.0.2.15/24
[0][INFO] Configure gateway with address 10.0.2.2
[0][INFO] MTU: 1500 bytes
[0][WARN] Unable to read entropy! Fallback to a naive implementation!
Starting server on port 8080
Now listening on port 8080
```

on another terminal window, run:
```sh
curl http://127.0.0.1:8080
```

> output 
> hello world

on the server (qemu) side, you should see an output:
```
received request! method: Get, url: "/", headers: [Header { field: HeaderField("Host"), value: "127.0.0.1:8080" }, Header { field: HeaderField("User-Agent"), value: "curl/7.88.1" }, Header { field: HeaderField("Accept"), value: "*/*" }]
```

Terminate the server with C-c:
```
qemu-system-aarch64: terminating on signal 2
```

### [Unikraft](https://github.com/unikraft/unikraft)
- a unikernel dev kit
- rust support only for x86 (and only supports no_std), no rust for aarch64.
- they just added support for vsock.

TO-DO: still trying to test unikraft
