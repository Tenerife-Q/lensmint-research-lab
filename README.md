# LensMint Research Lab & Architecture Prototyping

This repository serves as the empirical research and hardware prototyping environment for the **LensMint Web3 Camera GSoC 2026 Proposal**. 

Instead of jumping straight into application-layer UI code, this lab focuses on solving the most critical architectural bottlenecks in decentralized camera hardware: **Authenticity Fragility (Storage Noise)** and **Edge Hardware Constraints (RPi OOM)**.

---

## 1. The Hash Fragility Benchmark (Python)
Strict `SHA-256` hashing completely fails when images are uploaded to decentralized gateways (IPFS/Filecoin) due to benign compression. To mathematically prove this, we benchmarked 100+ high-res images.

### ROC & Distribution Results
*(The threshold `Distance <= 5` yields an AUC of 1.0, ensuring 0% false positives for benign compression while strictly rejecting malicious tampering).*

![ROC Curve and Distribution](https://private-user-images.githubusercontent.com/202472710/573545194-b73ecd2c-b765-4550-8116-196a9ebbefb8.png?jwt=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJnaXRodWIuY29tIiwiYXVkIjoicmF3LmdpdGh1YnVzZXJjb250ZW50LmNvbSIsImtleSI6ImtleTUiLCJleHAiOjE3NzU0NzY0OTAsIm5iZiI6MTc3NTQ3NjE5MCwicGF0aCI6Ii8yMDI0NzI3MTAvNTczNTQ1MTk0LWI3M2VjZDJjLWI3NjUtNDU1MC04MTE2LTE5NmE5ZWJiZWZiOC5wbmc_WC1BbXotQWxnb3JpdGhtPUFXUzQtSE1BQy1TSEEyNTYmWC1BbXotQ3JlZGVudGlhbD1BS0lBVkNPRFlMU0E1M1BRSzRaQSUyRjIwMjYwNDA2JTJGdXMtZWFzdC0xJTJGczMlMkZhd3M0X3JlcXVlc3QmWC1BbXotRGF0ZT0yMDI2MDQwNlQxMTQ5NTBaJlgtQW16LUV4cGlyZXM9MzAwJlgtQW16LVNpZ25hdHVyZT04MWI0YTc0MDI5YjMwZTdlYzk4ZTY0ZmIxMDJmMzMzY2Y1YzU2OGQzOGFjN2ViYTIwYmJkYjYxZDg1MjRlZGY4JlgtQW16LVNpZ25lZEhlYWRlcnM9aG9zdCJ9.C-f3kRStuJTYZi3yo7r4Hyx9YhMejlPedPKZeXFTmqI)

> **Conclusion**: `pHash` + `Hamming Distance` is a mathematical necessity for production-ready verification.

---

## 2. Edge ZK Circuit Performance (Rust / RISC Zero)
To ensure the ZK prover can run flawlessly on a memory-constrained Raspberry Pi without OOM crashes, I implemented a bare-metal RISC Zero MVP for the Hamming Distance logic.

**Local Profiling Results:**
- **Proving Time**: ~2.5s - 5s
- **Total Cycle Count**: `4355` *(Over 99% more efficient than strict SHA-256 circuits)*
- **Memory Segments**: `1` *(Absolute minimum footprint, zero risk of RAM exhaustion)*

---

## 3. AArch64 Cross-Compilation CI
To guarantee that our Rust stack is fully compatible with the Raspberry Pi OS, this repository is equipped with a GitHub Actions CI pipeline. 
It strictly verifies that the ZK host and pHash extractor compile successfully to the `aarch64-unknown-linux-gnu` target on every push.

[![AArch64 Edge Build](https://github.com/Tenerife-Q/lensmint-research-lab/actions/workflows/cross-compile.yml/badge.svg)](https://github.com/Tenerife-Q/lensmint-research-lab/actions/workflows/cross-compile.yml)

---

## 4. Cross-Language Validation (Zero-Regression)
Added a `rust-phash-validator` module to ensure that the production Rust daemon extracts the exact same `pHash` binary signature as the Python research benchmark. This guarantees algorithmic consistency across the entire Web3 pipeline.

---

## 5. Future Roadmap: Edge Security & Anti-Cloning
While the PoC proves the functional viability, securing a physical IoT device requires strict IPC and hardware-bound encryption. Read the technical roadmap here:
 **[SECURITY_RFC.md: Productionizing the Daemon](./SECURITY_RFC.md)**