# Nitro

Nitro is an experimental OOP language inspired by Rust with the following goals:

- Compiled to native code.
- [Non-fragile](https://en.wikipedia.org/wiki/Fragile_binary_interface_problem) and stable ABI.
- A library is distributed as a compiled binary similar to JAR or DLL file.
- GC using reference counting.
- Runtime reflection.
- Error handling using exception.
- No null value (except a pointer).
- Option type.

Nitro borrowed most of the syntax from Rust except:

- Nitro is an OOP language like Java or C#.
- Easy to learn, especially for people who already know Java, C# or Rust.
- No lifetime, no borrow checker, no const VS mut.
- No borrowed and owned type like `str` and `String`.
- Use exception like Java or C# for error handling (no checked exception).
- Nitro was designed for application programming rather than systems programming.

The goal of Nitro is to be a modern OOP language with the productivity of Rust syntax.

## Different from Java or C#

The main different is Nitro compiled to native code instead of Java bytecode or Common Intermediate
Language, which can be run without a VM. The benefit with this are:

- Low memory footprint.
- Fast startup.
- Can be run on a client machine directly without a VM.
- Easy to interop with other languages.

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
    fn Alloc(size: usize, align: usize): *u8 {
        @cfg(unix)
        let ptr = aligned_alloc(align, size);

        @cfg(windows)
        let ptr = _aligned_malloc(size, align);

        if ptr == null {
            @cfg(os != "windows")
            abort();

            @cfg(os == "windows")
            asm("int 0x29", in("ecx") 7, out(!) _);
        }

        ptr
    }

    @pub
    fn Free(ptr: *u8) {
        @cfg(unix)
        free(ptr);

        @cfg(windows)
        _aligned_free(ptr);
    }

    @cfg(unix)
    @ext(C)
    fn aligned_alloc(align: usize, size: usize): *u8;

    @cfg(unix)
    @ext(C)
    fn free(ptr: *u8);

    @cfg(windows)
    @ext(C)
    fn _aligned_malloc(size: usize, align: usize): *u8;

    @cfg(windows)
    @ext(C)
    fn _aligned_free(ptr: *u8);

    @cfg(unix)
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

### Build Nitro

#### Linux and macOS

```sh
./build-nitro.sh
```

Supply `debug` as a first argument if you want to hack on Nitro.

#### Windows

```powershell
.\build-nitro.ps1
```

Set parameter `Type` to `debug` if you want to hack on Nitro (e.g. `.\build-nitro.ps1 -Type debug`).

## License

BSD-2-Clause Plus Patent License
