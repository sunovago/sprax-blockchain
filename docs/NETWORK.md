# SPRX Protocol — P2P Networking Specification

## Transport & Framing
SPRX P2P networking operates over TCP on port `26656` (`NetworkConfig.p2p_port`), using length-prefixed binary/JSON framed messages (`sprax_network::NetworkMessage`).

---

## Message Protocol

| Message Type | Purpose |
|:---|:---|
| `Handshake` | Initial connection exchange containing chain ID, height, and peer identity |
| `HandshakeAck` | Acknowledgment of compatible protocol version and genesis hash |
| `TxGossip` | Broadcasts new unconfirmed transactions into peer mempools |
| `BlockGossip` | Propagates newly finalized blocks across full nodes |
| `Proposal` | Transmits round block proposals from the designated proposer |
| `Vote` | Gossips signed Prevote and Precommit attestations |
| `EquivocationEvidence` | Broadcasts double-signing cryptographic proofs across validators |
| `PeerDiscoveryRequest` | Requests list of known active peer addresses |
| `PeerDiscoveryResponse` | Returns active peer address list for dynamic discovery |
| `Ping` / `Pong` | Heartbeat keep-alive and connection health monitoring |

---

## Peer Discovery & Topology
Nodes connect to static bootstrap peers on startup and periodically exchange `PeerDiscoveryRequest` / `PeerDiscoveryResponse` messages to maintain an active routing table of connected peers.
