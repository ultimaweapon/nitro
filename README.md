# Pluto

Pluto is an experimental OOP language inspired by Rust with the following goals:

- Compiled to native code.
- [Non-fragile](https://en.wikipedia.org/wiki/Fragile_binary_interface_problem) and stable ABI.
- GC using reference counting.
- Runtime reflection.
- Error handling using exception.
- No null value (except a pointer).
- Option type.

Pluto borrowed most of the syntax from Rust except:

- Pluto is an OOP language like Java or C#.
- Easy to learn, especially for people who already know Java, C# or Rust.
- No lifetime, no borrow checker, no const VS mut.
- No borrowed and owned type like `str` and `String`.
- Use exception like Java or C# for error handling (no checked exception).
- Pluto was designed for application programming rather than systems programming.

The goal of Pluto is to be a modern OOP language with the productivity of Rust syntax.

## Different from Java or C#

The main different is Pluto compiled to native code instead of Java bytecode or Common Intermediate
Language, which can be run without a VM. The benefit with this are:

- Low memory footprint.
- Fast startup.
- Can be run on a client machine directly without a VM.
- Easy to interop with other languages.

## Current state

I'm currently writing the stage 0 compiler along side the `std` library. The goal of stage 0
compiler is to compile the `std` and `cli`. Once the first version of `cli` is fully working Pluto
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
git clone --recurse-submodules https://github.com/ultimaweapon/pluto.git
```

### Build dependencies

#### Linux and macOS

Run the following command in the root of this repository:

```sh
CMAKE_BUILD_PARALLEL_LEVEL=2 ./build-deps.sh
```

## License

BSD-2-Clause Plus Patent License
