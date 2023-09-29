## CCA realmOS

Investigate possible low-TCB realmOS for running
[Veracruz](https://github.com/veracruz-project/veracruz) on top of
CCA.

We started by looking for an OS based on seL4:

- [IceCap](https://gitlab.com/icecap-project/icecap): We had Veracruz
  running on IceCap until 15 Aug 2023, when it was removed to simplify
  work on other platforms.
- [CantripOS](https://github.com/AmbiML/sparrow-cantrip-full):
  Unfortunately CantripOS does not yet support Rust std applications,
  and does not have support for vsock.

Summary:

| Name                                                        | Type                     | Rust std support | Vsock
|-------------------------------------------------------------|--------------------------|------------------|------------------------
| [CantripOS](https://github.com/AmbiML/sparrow-cantrip-full) | Low-TCB OS based on seL4 | No               | No
| [Redox](https://gitlab.redox-os.org/redox-os/redox)         | Microkernel              | Yes              | No
| [Unikraft](https://github.com/unikraft/unikraft)            | Unikernel dev kit        | No               | Recently added
| [Hermit](https://github.com/hermit-os)                      | Unikernel / library OS   | Yes              | Not yet, but virtio-net
| [Theseus OS](https://github.com/theseus-os/Theseus)         | New OS written in Rust   | No               | No

### [Redox](https://gitlab.redox-os.org/redox-os/redox)

a Microkernel written in Rust. it has support for Rust std on aarch64, no vsock support.
        - support for aarch64, builds with no errors but I encountered some runtime errors (it's been reported in their gitlab just few weeks ago)
https://gitlab.redox-os.org/redox-os/redox/-/issues/1376
        - Their repo seems healthy, fairly active with few contributors, compared to the rest, their code base is a bit larger and complex.

To get started:
> PS: we tried running Redox on June 2023

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

- we haven't tried to run it inside of CCA or get an application running on top of redox on its own since the previously mentioned error is still not fixed.

### [Unikraft](https://github.com/unikraft/unikraft)

- a unikernel dev kit
- rust support only for x86 (and only supports no_std), no rust for aarch64 as of June 2023.
- they just added support for vsock.

Since there is no rust support for aarch64, we moved on to find another libOS.

### [Hermit](https://github.com/hermit-os)

(Last updated in Sep 2023.)

- A lightweight unikernel (library OS) written entirely in Rust.
- Runs on x86_64 and AArch64, though AArch64 is less tested and
  documented, and there is no SMP:
  [#737](https://github.com/hermit-os/kernel/issues/737)
- Can run Rust applications (with std), as well as C/C++, Go and
  Fortran applications.
- It is claimed: "Rust applications that use the Rust runtime and do
  not directly use OS services are able to run on Hermit without
  modifications." Unfortunately, however, many Rust crates, including
  [Wasmtime](https://crates.io/crates/wasmtime), indirectly and
  perhaps unnecessarily depend on Linux/Unix features of
  [libc](https://crates.io/crates/libc).
- Virtio-net is supported, but vsock is not finished:
  [#826](https://github.com/hermit-os/kernel/pull/826)
- [Recent work on kernel](https://github.com/hermit-os/kernel/issues?q=sort%3Aupdated-desc)
  (there are also [other repos](https://github.com/hermit-os/))

To run a Rust hello_world program on AArch64 with Hermit from
crates.io (tested in a `debian:12` Docker container on 19 Sep 2023):

```
sudo apt-get install -y build-essential curl git qemu-system-arm
curl https://sh.rustup.rs -sSf | sh
source "$HOME/.cargo/env"
rustup default nightly-2023-09-19
rustup component add rust-src
```

Build Hermit loader, needed for running on QEMU:
```
cd
git clone https://github.com/hermit-os/loader
cd loader
cargo xtask build --target aarch64 --release
```

Create application:
```
cd
cargo new hello
cd hello
```

Edit `Cargo.toml` to contain:
```
[package]
name = "hello"
version = "0.1.0"
edition = "2021"

[target.'cfg(target_os = "hermit")'.dependencies.hermit]
version = "*"
```

Edit `src/main.rs` to contain:
```
#[cfg(target_os = "hermit")]
use hermit as _;

fn main() {
    println!("Hello, world!");
}
```

Build and run:
```
cargo build --target aarch64-unknown-hermit \
  -Z build-std=std,core,alloc,panic_abort \
  -Z build-std-features=compiler-builtins-mem

qemu-system-aarch64 \
    -machine virt,gic-version=3 \
    -cpu cortex-a72 -smp 1 -m 512M  \
    -display none -serial stdio -semihosting \
    -kernel ../loader/target/aarch64/release/hermit-loader \
    -device guest-loader,addr=0x48000000,initrd=target/aarch64-unknown-hermit/debug/hello
```

This line should appear in the output near the end:
```
Hello, world!
```

(Stop QEMU with C-c.)

To run the [httpd](httpd) example with Hermit from source repo:

```
cd
git clone --recurse-submodules https://github.com/hermit-os/hermit-rs
cd hermit-rs/kernel
git checkout main
cd .../httpd
```

Modify the Hermit dependency in `Cargo.toml` to be:
```
[target.'cfg(target_os = "hermit")'.dependencies.hermit]
path = "../../hermit-rs/hermit"
```

Build and run:
```
cargo build --target aarch64-unknown-hermit \
  -Z build-std=std,core,alloc,panic_abort \
  -Z build-std-features=compiler-builtins-mem

qemu-system-aarch64 \
    -machine virt,gic-version=3 \
    -cpu cortex-a72 -smp 1 -m 512M  \
    -display none -serial stdio -semihosting \
    -kernel ../../loader/target/aarch64/release/hermit-loader \
    -device guest-loader,addr=0x48000000,initrd=target/aarch64-unknown-hermit/debug/httpd \
    -netdev user,id=u1,hostfwd=tcp::8080-:8080 \
    -device virtio-net-pci,netdev=u1,disable-legacy=on
```

Output should finish with:
```
Starting server on port 8080
Now listening on port 8080
```

Test from another terminal (in the same Docker container):
```
$ curl http://127.0.0.1:8080
hello world
$
```

#### RustyHermit on CCA

- Tried following the same instructions to start a rustyHermit vm inside of CCA (FVP) using qemu.

- When we create the vm inside the normal world we get the following error. (we didn't try to and solve the issue)

```sh
LOADER][INFO] Loader: [0x40200000 - 0x4021f000]
[LOADER][INFO] Found ELF file with size 18396400
[LOADER][INFO] Parsing kernel from ELF at 0x48000000..0x4918b4f0 (18396400 B)
[LOADER][INFO] Loading kernel to 0x40400000
[LOADER][INFO] Detect 1 CPU(s)
[LOADER][INFO] Detect UART at 0x9000000
[LOADER][INFO] Jumping to HermitCore Application Entry Point at 0x4045a000
[0][INFO] Welcome to HermitCore-rs 0.6.1
[0][INFO] Kernel starts at 0x40400000
[0][INFO] BSS starts at 0x4047fd00
[0][INFO] TLS starts at 0x0 (size 0 Bytes)
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
[0][INFO] The current hermit-kernel is only implemented up to this point on aarch64.
[0][INFO] Attempting to exit via QEMU.
#
```

- If we try to create the vm in the realm world, it just crashes after it tries to ftech the device tree.

### [Theseus OS](https://github.com/theseus-os/Theseus)

Theseus is a new OS written in Rust to experiment with shifting
responsibilities like resource management into the compiler and other
ideas. Although there is no Rust std for Theseus, [Wasmtime has been
ported to
Theseus](https://www.theseus-os.com/2022/06/21/wasmtime-complete-no_std-port.html)
as WASM is seen as the way to run software written in an unsafe
language on Theseus.

Theseus was originally written for x86_64 but most of the core
subsystems are now also working on AArch64.

To build and run Theseus on AArch64 (tested in a `debian:12` Docker
container on 28 Sep 2023):

```
sudo apt-get install -y curl gcc gcc-aarch64-linux-gnu git grub-pc-bin make nasm \
  qemu-system-arm wget xorriso
curl https://sh.rustup.rs -sSf | sh
source "$HOME/.cargo/env"
git clone --recurse-submodules --depth 1 https://github.com/theseus-os/Theseus.git
cd Theseus
make ARCH=aarch64 iso
make ARCH=aarch64 orun host=no graphic=no
```

You can then hit return to see the prompt `>`. Currently only
statically linked applications work from this console but you can do,
for example:
```
> cd /extra_files
> cd foo
no such file or directory: foo
exit 1
> cd wasm
```

Exit QEMU with C-a x.

Outside Docker it is also possible to run Theseus with:
```
make ARCH=aarch64 orun host=no
```

You then get a graphical window, but [the graphics stack has not been
fully ported to
AArch64](https://github.com/theseus-os/Theseus/issues/1049) so this is
currently not useful on AArch64.

Test on Intel by substituting `x86_64` for `aarch64` and use
`host=yes` if the host is also Intel.
