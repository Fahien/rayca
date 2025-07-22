# Rayca Utilities

A fast, ergonomic, and type-safe utility library, providing robust handle-based containers, byte conversion utilities, logging macros with color support, and precise timing tools.

## Features

- **Pack<T>**: A handle-based, type-safe container for efficient storage and retrieval.
- **Handle<T>**: Strongly-typed indices for safe, ergonomic access to packed data.
- **AsBytes / IntoBytes**: Zero-copy conversion of slices and vectors to byte slices.
- **Logging Macros**: Colorful, cross-platform logging for status, warnings, errors, and more.
- **Timer**: Simple, high-precision timer for measuring elapsed and delta time.

## Getting Started

Add to your `Cargo.toml`:

```toml
[dependencies]
rayca = { git = "https://github.com/Fahien/rayca", directory = "rayca-util" }
```
