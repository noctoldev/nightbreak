# nightbreak

## Overview
This project is a minimal EFI (Extensible Firmware Interface) bootkit stub written in Rust. It operates in a UEFI environment, loading and decrypting a payload from a specific disk location using the ChaCha20 encryption algorithm, then transferring execution to the decrypted payload. The stub is designed to run without the Rust standard library (`#![no_std]`) and includes a custom bump allocator for memory management.

## Features
- **Payload Loading**: Reads a payload from a fixed Logical Block Address (LBA) on a block device.
- **ChaCha20 Decryption**: Decrypts the payload in-place using a hardcoded key and nonce.
- **Memory Management**: Allocates memory for the payload and stub using UEFI Boot Services.
- **Self-Relocation**: Copies the stub to a new memory location before executing the payload.
- **No Standard Library**: Operates in a bare-metal UEFI environment with minimal dependencies.

## Technical Details
- **Language**: Rust
- **Target**: UEFI firmware (x86_64 architecture assumed)
- **Dependencies**: 
  - `uefi` crate for UEFI protocol and service access
  - `core` library for no-std compatibility
- **Payload Location**: LBA 34, size of 16 * 512 bytes (8 KiB)
- **Encryption**: ChaCha20 with a 256-bit key and 96-bit nonce
- **Memory Ros: Custom bump allocator with a 8 KiB static pool

### Key Components
1. **Entry Point (`efi_main`)**: Initializes the UEFI environment, disables the watchdog timer, and calls the main loop.
2. **Main Loop (`main_loop`)**: 
   - Locates a block device via `BlockIO` protocol.
   - Allocates memory for the payload and reads it from disk.
   - Decrypts the payload using ChaCha20.
   - Copies the stub to a new memory location and jumps to the payload.
3. **ChaCha20 Implementation**: A custom, minimal implementation for payload decryption.
4. **Bump Allocator**: Manages a static memory pool for dynamic allocations.

## Prerequisites
- Rust toolchain with `no_std` support (e.g., `nightly` for UEFI development).
- `uefi` crate (`cargo add uefi`).
- A UEFI-compatible build environment (e.g., `x86_64-uefi` target).
- A disk image or device with the encrypted payload at LBA 34.

## Building
1. Install the Rust toolchain:
   ```rustup target add x86_64-unknown-uefi```

# limitations
- Hardcoded payload location (LBA 34) and size (8 KiB).
- No error handling beyond UEFI status checks.
- Assumes a single block device is available.
- Key and nonce are hardcoded, (do not do, easy to change go for it).

# TODO 
- Dynamic Payload Location: Add a configuration header or scan the disk for a signature to locate the payload dynamically instead of hardcoding LBA 34.
- Secure Key Storage: Replace hardcoded key and nonce with a derivation mechanism (e.g., using TPM, UEFI variables, or a hardware-specific seed).
- Error Handling: Implement robust error checking for UEFI calls, disk I/O, and memory allocation, with fallback behaviors or debug output.
- Payload Verification: Add a digital signature (e.g., SHA256 + RSA) to verify the payload’s integrity and authenticity before execution.
- Stealth Features: Hide the stub from memory introspection by relocating it to a non-enumerable region or hooking UEFI services.
- Multi-Stage Loading: Support a multi-stage payload (e.g., a small loader that fetches a larger payload from network or encrypted partition).
- Anti-Forensics: Zero out sensitive memory (e.g., key, nonce, decrypted buffers) after use to prevent recovery.
- Compression: Compress the payload before encryption and decompress it at runtime to reduce disk footprint.
- UEFI Variable Persistence: Store configuration or state in UEFI variables for persistence across reboots.
- Driver Injection: Inject a payload into the UEFI runtime or early OS boot process (e.g., as a DXE driver or bootkit).
- Platform Detection: Add logic to detect hardware/firmware specifics (e.g., CPUID, SMBIOS) and adapt behavior accordingly.
- Network Support: Integrate a simple network stack (e.g., via SimpleNetwork protocol) to fetch payloads remotely.
- Obfuscation: Obfuscate the stub’s code and strings to evade static analysis or signature-based detection.
- Logging: Implement a minimal logging system (e.g., to video memory or serial port) for debugging in a UEFI environment.

# extra
- Flamingo (myself) developed the bootkit, maeven gave me the idea
- shout out stealth 
