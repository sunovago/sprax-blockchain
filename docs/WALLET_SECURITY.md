# Browser wallet persistence and recovery

The browser stores an AES-256-GCM encrypted recovery phrase and public account metadata
under `sprax.wallet.encrypted-vault.v1`. Passwords, plaintext recovery phrases and private
keys are not written to storage. New vaults use WebCrypto PBKDF2-HMAC-SHA256 with 600,000
rounds, a random 16-byte salt and 12-byte IV. The older 100,000-round version 1 format
remains readable. Invalid encoding, unsupported versions and unbounded KDF counts fail
before decryption. HTTPS or localhost with secure WebCrypto is required.

Reload opens the saved wallet locked. Wrong passwords leave it locked. Locking clears
the retained private-key byte array, account display and send/receive dialogs. Tab hiding,
page navigation and five minutes without interaction lock the wallet. The UI discards
async unlock work when the session changes. JavaScript strings and browser-engine copies
cannot be reliably zeroized; this is not a hardware wallet or independent audit assurance.

Use **Download encrypted wallet backup** to save the vault. A browser with no saved vault
can restore the file and unlock it with the existing password. Keep an offline recovery
phrase backup separately. Corrupted browser data is preserved and blocks new onboarding
instead of being silently overwritten. Importing a backup into this recovery screen
replaces that unreadable entry. Import does not accept plaintext keys or seed files.

## Chain interoperability change

The earlier SDK used SHA-256 for addresses, while Rust authenticates BLAKE3-derived
addresses. It also produced incompatible sign bytes. The SDK now uses the node's address
hash and Rust's exact transaction field order, Bech32 addresses and `priority_fee` field.
Ed25519 and Secp256k1 transfer fixtures are regenerated from TypeScript and executed by
Rust tests. REST balance responses contain integer atto-SPRX, never a formatted SPRX string.

The same recovery phrase/private key may therefore display a different address from an
earlier SDK build. Do not treat cached old addresses as signing identities or send funds
to them. Any funds deliberately assigned to a legacy SHA-256 address need an explicit,
reviewed chain migration; this code does not move or silently recover them.

The current derivation algorithm is SPRX-specific. Its displayed paths do not establish
BIP-44/SLIP-0010/BIP-32 interoperability with other wallets. Keep the SDK revision with
recovery documentation. Standard derivation with a versioned migration remains work.

RPC failures are shown as failures, rather than guessed zero balances/nonces or successful
receipts. Fiat quotes remain unavailable without a real price source. Named public
network endpoints in the SDK are configuration values, not evidence of a live mainnet.
Deployment, CSP/RPC hardening, browser end-to-end tests and independent security review
remain launch gates.
