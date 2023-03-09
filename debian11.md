## Recipe for building Cantrip in Debian Docker container

Tested on 2023-03-08:

```
mkdir -p /data/sparrow7
docker run --init -d --rm --name sparrow7 \
  -v /data/sparrow7:/work debian:11 sleep inf

# Root set-up:
docker exec -it sparrow7 /bin/bash
# Replace number and name here:
addgroup --gid 8352 egrimley
adduser --uid 8352 --gid 8352 egrimley # hit return lots
apt-get update
apt-get -y dist-upgrade
apt-get install -y build-essential cmake cpio curl device-tree-compiler gawk git \
  haskell-stack libarchive-dev libxml2-utils ninja-build pip \
  python3 python3-libarchive-c python3-yaml qemu-system-arm wget

# User set-up:
docker exec -it -u egrimley sparrow7 /bin/bash
# Replace name and address here:
git config --global user.email "edmund.grimley-evans@arm.com"
git config --global user.name "Edmund Grimley Evans"
mkdir ~/bin
PATH="$HOME/bin:$PATH"
ln -s /usr/bin/python3 ~/bin/python
curl https://storage.googleapis.com/git-repo-downloads/repo > ~/bin/repo
chmod a+rx ~/bin/repo
curl https://sh.rustup.rs -sSf | sh # hit return
source "$HOME/.cargo/env"
pip install aenum future jinja2 jsonschema ordered_set \
  plyplus pyelftools pyfdt six sortedcontainers tempita
cd
wget https://developer.arm.com/-/media/Files/downloads/gnu/11.2-2022.02/binrel/gcc-arm-11.2-2022.02-x86_64-aarch64-none-linux-gnu.tar.xz
tar xf gcc-arm-11.2-2022.02-x86_64-aarch64-none-linux-gnu.tar.xz
PATH="~/gcc-arm-11.2-2022.02-x86_64-aarch64-none-linux-gnu/bin:$PATH"
echo 'PATH="$HOME/bin:$PATH"' >> ~/.bashrc
echo 'PATH="$HOME/gcc-arm-11.2-2022.02-x86_64-aarch64-none-linux-gnu/bin:$PATH"' >> ~/.bashrc
rustup toolchain add nightly-2023-01-26-x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-none

# Get Sparrow source:
cd /work
mkdir sparrow
cd sparrow
repo init -u https://github.com/AmbiML/sparrow-manifest -m sparrow-manifest.xml
repo sync -j$(nproc)

# Undo some recent breaking changes:
( cd cantrip/projects/cantrip/apps/system/components/SecurityCoordinator/cantrip-security-coordinator && git checkout e7b4b1022645efcab64d55209315eb730d4a2bd0 )

# Build and test:
docker exec -it -u egrimley sparrow7 /bin/bash
cd /work/sparrow
export PLATFORM=rpi3
source build/setup.sh
m simulate

# After about 10-15 minutes QEMU is running and we see:
48 bytes in-use, 130510544 bytes free, 48 bytes requested, 1392640 overhead
2 objs in-use, 2 objs requested
CANTRIP> EOF
# Interrupt with C-a x

# To demonstrate Rust apps running:
perl -i -pe 's/_C_/_RUST_/ if /hello/;' build/platforms/rpi3/cantrip_builtins.mk
cat > build/platforms/rpi3/autostart.repl <<'EOF'
builtins
install hello.app
install logtest.app
start hello
start logtest
EOF
m simulate
```
