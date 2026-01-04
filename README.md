# Incremental Reflexion Analysis Prototype (Rust)

This repository contains a **small, exploratory Rust prototype** for studying the core semantics of **incremental reflexion analysis**, inspired by the work of Prof. Rainer Koschke.

The project is intended purely as a **learning and research exercise** to better understand how mapping, propagation, lifting, and classification interact at a semantic level.

## Scope

- Experimental implementation of reflexion analysis concepts
- Focus on internal graph transformations and invariants
- Operates on simplified, abstract components

## How it works (briefly)

1. An abstract **architecture graph** and **implementation graph** are defined.
2. A **mapping** relates implementation elements to architectural components.
3. Implementation dependencies are **propagated and lifted** to the architectural level.
4. Resulting dependencies are **classified** into reflexion states (e.g., convergent, divergent, absent).

The implementation operates on simplified, synthetic examples to focus on semantics rather than real-world extraction.

## Status

- Exploratory / research quality
- APIs and structure may change
- Not a production-ready tool

