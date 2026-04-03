# LensMint Authenticity Hash Resilience Benchmark
**Date**: 2026-04-03
**Dataset**: 100 random real-world photography images (800x600)

## Context
This benchmark simulates how decentralized storage gateways (e.g., IPFS/Filecoin) process images. It proves that strict SHA-256 verification is completely unviable for Web3 hardware cameras, while **pHash maintains mathematical stability suitable for ZK verification**.

## Benchmark Results

| Network Noise Simulation | SHA-256 Failure Rate | pHash Avg Hamming Distance | pHash Max Distance | pHash Match (Threshold <= 5) |
|--------------------------|----------------------|----------------------------|--------------------|-----------------------------|
| Gateway 95% JPEG | **100.0%** (Avalanche) | 0.14 | 2 | **100.0%** |
| Gateway 85% JPEG | **100.0%** (Avalanche) | 0.18 | 2 | **100.0%** |
| Aggressive 70% JPEG | **100.0%** (Avalanche) | 0.18 | 2 | **100.0%** |
| Resized (80% scale) | **100.0%** (Avalanche) | 0.26 | 2 | **100.0%** |

## Conclusion
- **SHA-256 is entirely shattered** by even a visually lossless 95% JPEG re-encode (100% failure rate).
- **pHash provides robust provenance**. Setting the ZK configurable threshold to `5` safely absorbs standard decentralized storage noise.