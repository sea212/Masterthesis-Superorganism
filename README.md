# Masterthesis-Superorganism
Significant advancements in decentralized storage and computation
technologies, decentralized consensus-based distributed ledger
technologies and artificial intelligence have created new but largely
unexplored possibilities. This work evaluates how those technologies
can be combined to create a decentralized platform, that is completely
controlled by the community and whose purpose is solely defined by the
community. The system permits the bundling of power in form of
central authorities and equally spreads the power over the community,
the only requirement for participation being the possession of a
computer with a webcam. In addition, financial wealth must not lead
to an higher impact. When launched, the systems purpose is not
defined. The default features of the system evolve during time
depending on the ideas of the community and truly democratic
processes. The system is able to reward users for participation with an
intrinsic cryptographic currency. This should encourage the users to
share ideas and concerns and in addition enables the possibility to
spawn projects, which are realized and operated by the community,
whereupon the participants are rewarded by the system. Whatever is
decided by the community and created within the system, is paid by
the system and owned by everybody, it becomes a common good.
Hence, the community shapes the system and defines its purpose so
that it serves the community according to their needs.

To experience this system, you can either run a pre-built vm (1) or build everything from the sources and run it directly on your machine (2).

### 1 - Using the pre-built VM
1) Install VirtualBox
2) Extract Superorganism.vdi.xz, you can retrieve it from IPFS:
   - Direct: ipfs://QmPHTjEYS8J8gfQJZ9gzq1bGLmjkT2Gb5jD19znLvtjzSV?filename=Superorganism.vdi.xz
   - Gateway 1: https://ipfs.io/ipfs/QmPHTjEYS8J8gfQJZ9gzq1bGLmjkT2Gb5jD19znLvtjzSV?filename=Superorganism.vdi.xz
   - Gateway 2: https://dweb.link/ipfs/QmPHTjEYS8J8gfQJZ9gzq1bGLmjkT2Gb5jD19znLvtjzSV?filename=Superorganism.vdi.xz
3) Create a new virtual machine in VirtualBox and insert Superorganism.vdi as the virtual hard drive
4) Configure processor usage, RAM, VRAM, etc.
5) Run the virtual machine
6) Double click on the desktop item "run-blockchain.sh" to start the Superorganism ecosystem
7) Double click on the desktop item "run-frontend.sh" and wait for the console log that ends with "HtmlWebpackPlugin_0 [built]"
8) Launch firefox, it should display the webpage at localhost:3000
9) Interact with the system.

### 2 - Build everything from sources
1) Install rustup (https://rustup.rs/)
2) Install the nightly compiler toolchain: rustup toolchain install nightly-2020-10-05
3) List rust toolchains: rustup show
4) Copy string containing "nightly-2020-10-05", for example "nightly-2020-10-05-x86_64-unknown-linux-gnu"
5) Select the copied string as the default toolchain: rustup default <copied toolchain string>
6) Install WebAssembly target: rustup target add wasm32-unknown-unknown
7) Switch folder to implementation/superorganism
8) Run: cargo build --release (note: needs packages clang, llvm, build-essential, git)
9) While it builds, get yarn 1.22.5 (https://github.com/yarnpkg/yarn/releases/tag/v1.22.5) and node v12.19.0 (https://nodejs.org/en/blog/release/v12.19.0/)
10) Switch folder to implementation/superorganism-frontend
11) Get dependencies by running: yarn
12) When "cargo build --release" has finished, run "cargo run --release" in the same console
13) When "yarn" has finished and "cargo run --release" is running, execute "yarn run start" in the same console where "yarn" was executed
14) Launch firefox and browse the web page at localhost:3000
15) Interact with the system.
