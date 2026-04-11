# LensMint Edge Security RFC: Productionizing the Daemon

While the `lensmint-daemon-poc` successfully validates the cryptographic pipeline (OS Entropy -> Ed25519 -> pHash -> ZK Signature), migrating this into a production physical camera requires bridging two critical edge-security gaps. 

This RFC outlines the security roadmap for the GSoC coding period to harden the Raspberry Pi environment.

## 1. Defeating Local Privilege Escalation (IPC Security)
**Current PoC State**: The daemon listens on TCP `127.0.0.1:3030`.
**Vulnerability**: Any rogue background process or local SSRF exploit on the Pi could send a `POST /api/internal/capture` request, triggering unauthorized minting payloads.

**Production Solution: Unix Domain Sockets (UDS) with `SO_PEERCRED`**
- During the GSoC period, the Axum TCP listener will be replaced with a **Unix Domain Socket** (e.g., `/var/run/lensmint.sock`).
- **Identity-Based Authorization**: We will enforce `SO_PEERCRED` at the kernel level. The Rust daemon will check the UID/GID of every incoming request. Only the specific Linux user running the Kivy Camera UI will be authorized to trigger the capture pipeline. This physically isolates the minting privilege.

## 2. Hardware-Bound Key Persistence (Anti-Cloning)
**Current PoC State**: The Ed25519 keypair is generated entirely in-memory via `/dev/urandom` upon every daemon startup.
**Vulnerability**: Physical cameras need persistent identity. However, storing the private key in plain text on the SD card allows anyone to extract the card and clone the camera. (Standard Raspberry Pi 4 lacks a physical TPM/Secure Enclave).

**Production Solution: CPU-Bound Encryption-at-Rest**
- **Hardware Salt**: Upon the very first boot, the daemon will read the immutable CPU serial number from the Pi's silicon (`/sys/firmware/devicetree/base/serial-number`).
- **Key Derivation (KDF)**: The CPU serial will be used as the salt in an Argon2id / AES-GCM encryption pipeline to encrypt the generated 32-byte Ed25519 seed before persisting it to the SD card.
- **Anti-Extraction**: If a thief steals the SD card and boots it on another Raspberry Pi or a PC, the CPU serial mismatch will mathematically fail the decryption process, rendering the stolen identity useless.

## 3. Target Architecture & Security Boundary

```mermaid
graph TD
    %% Custom Styling
    classDef python fill:#e1f5fe,stroke:#0288d1,stroke-width:2px,color:#000
    classDef security fill:#ffebee,stroke:#d32f2f,stroke-width:2px,color:#000
    classDef rust fill:#fff3e0,stroke:#f57c00,stroke-width:2px,color:#000
    classDef external fill:#f3e5f5,stroke:#7b1fa2,stroke-width:2px,color:#000

    subgraph "Layer 1: Frontend"
        User((👤 Camera User)) --> UI[📷 Kivy Camera UI <br/> Python Process]
    end

    subgraph "Layer 2: IPC Security Boundary"
        UI == POST /capture ==> UDS{🔌 Unix Domain Socket <br/> /var/run/lensmint.sock}
        UDS -.-> Auth[🛡️ Kernel SO_PEERCRED <br/> UID/GID Validation]
        Auth -- Unauthorized --> Drop[❌ Drop Request]
    end

    subgraph "Layer 3: Rust Hardware Daemon"
        Auth == Authorized ==> Capture[⚡ Capture Event]
        
        %% Hardware-bound Key Pipeline
        CPU[🖥️ Hardware CPU Serial] --> KDF[⚙️ AES-GCM Decryption]
        SD[💾 SD Card: Encrypted Seed] --> KDF
        KDF --> Key[(🔑 In-Memory Ed25519 Key)]
        
        %% Cryptographic Pipeline
        Capture --> Ext[🔎 pHash Extractor]
        Ext --> Sign[✒️ Hardware Signer]
        Key -.->|Provide Key| Sign
    end

    subgraph "Layer 4: Outer Web3 World"
        Sign == Image + Metadata ==> Filecoin[(🗄️ Filecoin Storage)]
        Sign == Payload: pHash + Sig ==> ZK[🔗 RISC Zero zkVM Prover]
    end

    %% Apply Classes
    class UI python;
    class UDS,Auth,Drop security;
    class CPU,SD,KDF,Key,Capture,Ext,Sign rust;
    class Filecoin,ZK external;
