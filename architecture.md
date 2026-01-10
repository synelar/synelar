# SynID - Complete Architecture & Codebase Documentation

## Table of Contents
1. [Project Overview](#project-overview)
2. [Tech Stack](#tech-stack)
3. [File Structure](#file-structure)
4. [Core Libraries & Utilities](#core-libraries--utilities)
5. [Data Flow](#data-flow)
6. [Component Architecture](#component-architecture)
7. [API Routes](#api-routes)
8. [NFT Minting Process](#nft-minting-process)
9. [Access Control System](#access-control-system)
10. [Storage & Data Persistence](#storage--data-persistence)

---

## Project Overview

**SynID** is a decentralized identity system built on Solana that allows users to:
- Create soulbound identity NFTs (SynID) on Solana devnet
- Encrypt personal data and store it on IPFS
- Monetize access to their data by receiving SOL payments
- Control which apps/users can access which fields
- Track earnings and access logs in real-time

### Key Features
- **Wallet-based Authentication**: Phantom wallet integration
- **Encrypted Storage**: AES-256-GCM encryption + IPFS
- **Real NFT Minting**: Solana SPL tokens with Metaplex metadata
- **Testnet Payouts**: Real SOL transfers on devnet
- **Interactive Demo**: Test access flow without minting
- **Dynamic Dashboard**: Real-time earnings and access tracking

---

## Tech Stack

### Frontend
- **Framework**: Next.js 16.0.7 (App Router)
- **UI**: React 19.2.1 with Tailwind CSS v4
- **Wallet Integration**: Phantom (Solana wallet)
- **State Management**: React hooks + localStorage
- **Animations**: Framer Motion, custom CSS animations
- **HTTP Client**: Native fetch API

### Backend
- **Runtime**: Next.js API Routes (serverless)
- **Blockchain**: Solana Web3.js SDK
- **NFT Standard**: SPL Token + Metaplex metadata
- **Storage**: IPFS (Pinata gateway)
- **Encryption**: Web Crypto API (AES-256-GCM)
- **Private Key**: bs58 for key parsing

### DevOps
- **Deployment**: Vercel
- **Database**: localStorage (client-side)
- **Environment**: Solana Devnet
