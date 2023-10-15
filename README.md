# Nitro

Nitro is an experimental **OOP language** with the following goals:

- [x] Compiled to **native code** like C/C++/Go or Rust.
- [ ] **[Non-fragile](https://en.wikipedia.org/wiki/Fragile_binary_interface_problem)** and
  **stable** ABI.
- [x] A Nitro library will be distributed as a **compiled binary** similar to JAR or DLL file, not
  as a source code like most languages did.
- [x] Built-in **cross compilation** (e.g. one can produce Linux binary on Windows).
- [ ] **GC** using reference counting.
- [ ] **Runtime reflection**.
- [ ] Error handling using **exception**.
- [ ] **No null** value (except a pointer).
- [ ] **Option type**.

Nitro borrowed most of the syntax from Rust except:

- Nitro is an OOP language like Java or C#.
- Easy to learn, especially for people who already know Java, C# or Rust.
- No lifetime, no borrow checker, no const VS mut.
- No borrowed and owned type like `str` and `String`.
- Use exception like Java or C# for error handling (no checked exception).
- Nitro was designed for application programming rather than systems programming.

## Different from Java or C#

The main different is Nitro compiled to **native code** instead of Java bytecode or Common
Intermediate Language, which can be run **without a VM**. The benefit with this are:

- Low memory footprint.
- Fast startup.
- Can be run on a client machine directly without a VM.
- Easy to interop with other languages.

## Different from Swift and D

The reason Nitro was born is because of:

- D has fragile ABI.
- Swift does not support namespace.

## Current state

I'm currently writing the stage 0 compiler along side the `std` library. The goal of stage 0
compiler is to compile the `std` and `cli`. Once the first version of `cli` is fully working Nitro
will become a self-hosted language.

## Example

```
@pub
class Allocator;

impl Allocator {
    @pub
    fn Alloc(size: UInt, align: UInt): *UInt8 {
        @if(unix)
        let ptr = aligned_alloc(align, size);

        @if(win32)
        let ptr = _aligned_malloc(size, align);

        if ptr == null {
            @if(os != "windows")
            abort();

            @if(os == "windows")
            asm("int 0x29", in("ecx") 7, out(!) _);
        }

        ptr
    }

    @pub
    fn Free(ptr: *UInt8) {
        @if(unix)
        free(ptr);

        @if(win32)
        _aligned_free(ptr);
    }

    @if(unix)
    @ext(C)
    fn aligned_alloc(align: UInt, size: UInt): *UInt8;

    @if(unix)
    @ext(C)
    fn free(ptr: *UInt8);

    @if(win32)
    @ext(C)
    fn _aligned_malloc(size: UInt, align: UInt): *UInt8;

    @if(win32)
    @ext(C)
    fn _aligned_free(ptr: *UInt8);

    @if(unix)
    @ext(C)
    fn abort(): !;
}
```

## Build from source

### Prerequisites

- Git
- Rust
- C++ toolchain (e.g. MSVC, XCode, GCC)
- CMake

### Download the source

You need to clone this repository with submodules like this:

```sh
git clone --recurse-submodules https://github.com/ultimaweapon/nitro.git
```

### Build dependencies

Run the following command in the root of this repository:

#### Linux and macOS

```sh
CMAKE_BUILD_PARALLEL_LEVEL=2 ./build-deps.sh
```

#### Windows

```powershell
.\build-deps.ps1
```

### Build CLI

#### Linux and macOS

```sh
./build-cli.sh
```

Supply `debug` as a first argument if you want to hack on Nitro.

#### Windows

```powershell
.\build-cli.ps1
```

Set parameter `Type` to `debug` if you want to hack on Nitro (e.g. `.\build-cli.ps1 -Type debug`).

### Build distribution

#### Linux and macOS

```sh
./build-dist.sh
```

#### Windows

```powershell
.\build-dist.ps1
```

## License

BSD-2-Clause Plus Patent License
