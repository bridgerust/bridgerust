---
title: Performance Benchmarks
description: Embex vs Native Clients speed comparison.
---

Embex leverages a high-performance **Rust core** with SIMD acceleration to deliver speedups over native Python/JS clients.

## Python Benchmarks

Benchmarks run on M1 Max with 10k vectors (384 dimensions).

| Provider     | Client    | Insert (ops/s) | Speedup  | Search Latency |
| :----------- | :-------- | :------------- | :------- | :------------- |
| **Qdrant**   | **Embex** | **24,825**     | **4.3x** | **1.95ms**     |
|              | Native    | 5,754          |          | 4.69ms         |
| **Weaviate** | **Embex** | **5,163**      | **4.1x** | **1.77ms**     |
|              | Native    | 1,256          |          | 4.03ms         |
| **Chroma**   | Embex     | 3,136          | 1.0x     | 3.97ms         |
|              | Native    | 3,077          |          | 3.46ms         |

![Benchmark Insert](https://raw.githubusercontent.com/bridgerust/bridgerust/main/assets/benchmark_insert.png)

![Benchmark Search](https://raw.githubusercontent.com/bridgerust/bridgerust/main/assets/benchmark_search.png)

(Graphs linked from repository assets)

## Micro-Benchmarks

| Operation     | Python | Embex (Rust) | Speedup  |
| :------------ | :----- | :----------- | :------- |
| Normalization | 45ms   | 11ms         | **4.1x** |
| Cosine Sim    | 230ms  | 58ms         | **4.0x** |
