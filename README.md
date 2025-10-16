# Depth-Protocol-Grant-Engine
The Depth Protocol Grant Engine (DGE) is a two-sided governance firewall designed to protect DAO treasuries from mercenary capital allocation.

# Building Trust with Non-Monetary Collateral

We solve the crisis of governance failure in DeFi by replacing mere token holdings with verifiable Founder Commitment (Depth-Points) as the primary credential for accessing and managing funds.

Submission Type: DAO Governance / Protocol Infrastructure

# The Dual Security Layer Rationale

The DGE operates on two non-negotiable security layers: the Commitment Layer (Depth-Points) and the Risk Management Layer (Adaptive Quorum & Builder Bond).

Layer 1: Commitment & Allocation Rights

We enforce the principle that investment in self must precede access to community capital.

Mechanism

Description

Security Rationale

D-Metric Floor

A minimum 10 D-Point score is required to submit any grant proposal.

Filters out zero-commitment and spam accounts, ensuring a basic level of contribution and competency.

Builder Bond

A $300 USD collateral stake (paid in $DPT$ or stablecoin) is required for submission.

Ensures the founder has skin in the game. This bond covers initial risk and is liquidated upon project failure (D-Metric < 50).

Commitment Scaling

Max grant capacity scales from $1,000 (at 10 D-Pnt) up to $10,000 (at 250 D-Pnt).

Forces founders to demonstrate long-term commitment (250 D-Points) to earn the right to request high-value funds, protecting the treasury from large, early-stage risks.

Layer 2: Adaptive Risk Management (Adaptive Quorum)

All funds are disbursed in phased tranches, and the vote to release each tranche is governed by the project's current performance status.

Adaptive Quorum (AQ) Logic

The required approval percentage for any milestone release vote dynamically increases as the project's performance (D-Metric) declines.

$$\text{Required Approval} = 51\% + \text{Risk Penalty}$$

$$\text{Risk Penalty} = \frac{(100 - \text{Current D-Metric})}{2}$$

D-Metric

Risk Penalty

Required Quorum

Analysis

100 (Baseline)

0%

51%

Standard vote for a project on track.

60 (Struggling)

20%

71%

A significant increase in consensus is needed to continue funding a high-risk project.

50 (Liquidation Floor)

25%

76%

If a vote is even attempted, the project needs overwhelming community support to avoid liquidation.

# Project Architecture

The DGE is built as a modular program utilizing the Depth Protocol OS (DPOS) library.

File

Content

Purpose

src/lib.rs

Core Rust logic (Adaptive Quorum, Builder Bond calculation).

On-chain verification and security engine.

DepthGrantSimulator.jsx

Single React component demo.

Front-end visualization for judges to test D-Metric and AQ mechanics.

README.md

This document.

Technical specification and sales narrative.

# How to Test the Simulator

Test Max Capacity: Set Founder D-Metric to 250. Observe the Max Allocation Right show $10,000.

Test Commitment Floor: Set Founder D-Metric to 5. Observe the Max Allocation Right drop to $0, and the Submission Blocked status.

Test Risk Adjustment: Start with Project D-Metric at 100. The Adaptive Quorum is 51%. Reduce the Project D-Metric to 60. The Adaptive Quorum immediately increases to 71%, demonstrating real-time risk mitigation.

Test Liquidation: Reduce the Project D-Metric to 50. Observe the LIQUIDATION status trigger, signaling the immediate forfeiture of the Builder Bond.

The Depth Protocol Grant Engine ensures that only those who invest in themselves are entrusted with community capital.
