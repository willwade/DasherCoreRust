# DasherCoreRust

A Rust implementation of the Dasher text entry system core. This library provides the core functionality of Dasher, a zooming predictive text entry system designed for situations where keyboard input is impractical (for instance, accessibility or mobile devices).

## Building

### Prerequisites

- Rust toolchain (1.70.0 or later) - Install via [rustup](https://rustup.rs/)
- Cargo (comes with Rust)
- For WebAssembly builds: wasm-pack
- For C++ integration: A C++ compiler (gcc, clang, or MSVC)

### Build Options

#### As a Rust Library

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Build with SIMD optimizations
cargo build --release --features simd

# Build documentation
cargo doc --open
```

#### As WebAssembly

```bash
# Install wasm-pack if you haven't already
cargo install wasm-pack

# Build for web
wasm-pack build --target web --features wasm

# Build for Node.js
wasm-pack build --target nodejs --features wasm
```

#### As a C++ Library

The library can be built as a static or dynamic library for C++ integration using the FFI interface.

```bash
# Build as a static library
cargo build --release

# The library will be available at:
# - Windows: target/release/dasher_core.lib
# - Linux/macOS: target/release/libdasher_core.a
```

## Integration with C++

### Using with CMake

```cmake
# CMakeLists.txt example
cmake_minimum_required(VERSION 3.15)
project(MyDasherApp)

# Add Rust library
add_library(dasher_core STATIC IMPORTED)
set_target_properties(dasher_core PROPERTIES
    IMPORTED_LOCATION "${CMAKE_SOURCE_DIR}/rust/target/release/libdasher_core.a"
    INTERFACE_INCLUDE_DIRECTORIES "${CMAKE_SOURCE_DIR}/rust/include"
)

# Add your C++ executable
add_executable(my_app main.cpp)
target_link_libraries(my_app PRIVATE dasher_core)
```

### C++ Header Example

```cpp
// dasher.hpp
#pragma once
#include <cstdint>

extern "C" {
    // Opaque types
    struct DasherInterfaceFFI;
    struct DasherScreenFFI;
    struct DasherInputFFI;

    // Core functions
    DasherInterfaceFFI* dasher_interface_create(void* settings);
    void dasher_interface_destroy(DasherInterfaceFFI* interface);
    bool dasher_interface_new_frame(DasherInterfaceFFI* interface, uint64_t time_ms);
    
    // Screen handling
    DasherScreenFFI* dasher_create_screen(int32_t width, int32_t height);
    void dasher_destroy_screen(DasherScreenFFI* screen);
    
    // Input handling
    DasherInputFFI* dasher_create_mouse_input();
    void dasher_destroy_input(DasherInputFFI* input);
    bool dasher_set_mouse_coordinates(DasherInputFFI* input, int32_t x, int32_t y);
}
```

### C++ Usage Example

```cpp
#include "dasher.hpp"
#include <memory>

class DasherApp {
public:
    DasherApp() {
        m_interface = dasher_interface_create(nullptr);
        m_screen = dasher_create_screen(800, 600);
        m_input = dasher_create_mouse_input();
    }
    
    ~DasherApp() {
        dasher_destroy_input(m_input);
        dasher_destroy_screen(m_screen);
        dasher_interface_destroy(m_interface);
    }
    
    void update(uint64_t time_ms, int32_t mouse_x, int32_t mouse_y) {
        dasher_set_mouse_coordinates(m_input, mouse_x, mouse_y);
        dasher_interface_new_frame(m_interface, time_ms);
    }

private:
    DasherInterfaceFFI* m_interface;
    DasherScreenFFI* m_screen;
    DasherInputFFI* m_input;
};
```

## Features

- Core Dasher text entry system
- WebAssembly support for web applications
- FFI interface for native integration
- SIMD optimizations (optional)
- Custom rendering backend support

## License

GPL-2.0-or-later - See LICENSE file for details
