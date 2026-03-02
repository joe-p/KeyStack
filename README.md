# KeyStack

KeyStack is a KMS (Key Management System) that is intended to be simple and secure. The primary feature of KeyStack is its modular architecture built on top of WebAssembly (WASM). WASM allows functionality to be added to KeyStack without needing to modify the source code and ensuring that each new functionality is fully sandboxed from the rest of the system. This means that even if a vulnerability is found in one module, it will not affect the security of the entire system.

KeyStack defines multiple providers that can be implemented via WASM modules or by adding structs that implement the corresponding traits in the Rust code.

| Provider          | Description                                                                                            | Rust Support | WASM Support |
| ----------------- | ------------------------------------------------------------------------------------------------------ | ------------ | ------------ |
| Secret Provider   | Store and retrieve secrets.                                                                            | Yes          | TODO         |
| Crypto Provider   | Perform actions with secrets (i.e. signing)                                                            | Yes          | TODO         |
| Identity Provider | User authentication and role verification                                                              | Yes          | TODO         |
| Context Provider  | Takes action based on the context of a request (i.e. deny access) and/or inject context into a request | Yes          | Yes          |
| Log Provider      | Logging events from `keystack-core` and other providers                                                | TODO         | TODO         |

An example stack might be:

| Provider          | Implementation  |
| ----------------- | --------------- |
| Secret Provider   | OS Keyring      |
| Crypto Provider   | Libcrux Ed25519 |
| Identity Provider | Pocket-ID       |
| Context Provider  | algokit-core    |
| Log Provider      | stdout          |

This stack would allow for the following flow

1. Users to authenticate themselves with a passkey via pocket-id
1. Signing context enriched by algokit-core to control access (i.e. amount limits)
1. The signing key is retrieved from the OS keyring
1. The transaction payload is signed with libcrux.

# Status

KeyStack is currently a work-in-progress and is not yet ready for production use. Major breaking changes are expected. The current target milestone is to have an HTTP server on top of `keystack-core` with a simple identity provider, such as [pocket-id](https://pocket-id.org/) and a keyring-based secret provider.

## Vibe-Coded Proof-of-Concept

This repo contains the actual implementation (artisanal hand-crafted code) of KeyStack.

A proof-of-concept implementation of KeyStack and an HTTP server with pocket-id can be found here: https://github.com/joe-p/keystack-poc. The poc repo was entirely vibe-coded and only exists as a proving grounds for the overall architecture of KeyStack.
