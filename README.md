# lit-coin

We intended to build out a scoped down version of the blockchain network as described by the [Bitcoin whitepaper](https://bitcoin.org/bitcoin.pdf). We will make the following assumptions to make the project feasible during this course of the class:

- Each transaction will count as 1 block
- Clients in the network need to find each other in order to communicate. We anticipate building a truly decentralized node finding method to be quite troublesome so we will broker the conenction between clients using a central server but after that, all clients communicate peer-to-peer.
- All clients in the network will validate and broadcast blocks but also attempt the mining puzzle.

## Evaluation criteria

Our goal is to build something "system-y" with Rust that is non-trivial. We promise a working implementation, not a scalable or fast one. Most importantly, we want to have fun while building something in Rust.

Thus, we think the evaluation crieria should be that one can join our blockchain network and perform transactions on it and get rewarded for solving mining puzzles.
