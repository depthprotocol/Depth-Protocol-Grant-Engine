# Depth Grant Engine (DGE) Solana Program Architecture

This directory contains the foundational Rust source code for the DGE program, designed to be deployed as a Solana smart contract.

# Program Status: Architectural Proof of Concept (Non-Deployed)

# Strategic Rationale (Hackathon):
To maximize the limited development window, the team prioritized mechanism fidelity and demonstration. The core logic from these Rust files was modeled and validated in the live-running JavaScript simulator (index.html).

# Key Files:

lib.rs: Defines the program entry points, instruction parsing, and contains the core D-Metric update logic.

core.rs (or relevant logic file): Contains the core data structures (DMetricAccount) and the implementation of the Adaptive Quorum calculation.

This architecture is fully prepared for final compilation and deployment during the Colosseum Accelerator.
