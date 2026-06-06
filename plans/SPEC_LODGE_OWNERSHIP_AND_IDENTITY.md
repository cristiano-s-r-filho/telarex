# Spec: Lodge Ownership, Deletion & Multi-Session Identity

## 1. Problem

In a P2P CRDT-based system, there is no central server to enforce ownership or deletion. Multiple devices belonging to the same user must be recognized as a single identity. This spec defines the cryptographic and protocol mechanisms for lodge sovereignty, deletion, and multi-device identity.

## 2. Identity Model: Hierarchical Key Hierarchy

```
User Identity (Dilithium root keypair — created once, kept secret)
  ├── Device Key A (ed25519) → Automerge Actor ID_A
  │     Signed by User Identity: "Actor ID_A belongs to User PubKey"
  ├── Device Key B (ed25519) → Automerge Actor ID_B
  │     Signed by User Identity: "Actor ID_B belongs to User PubKey"
  └── Device Key C (ed25519) → Automerge Actor ID_C
        Signed by User Identity: "Actor ID_C belongs to User PubKey"
```

### 2.1 Key Types

| Key | Algorithm | Purpose | Storage |
|-----|-----------|---------|---------|
| Identity Root Key | ML-DSA (Dilithium) mode2 | Lodge ownership, identity signing | Encrypted in OS keychain / config dir |
| Device/Session Key | ed25519 | Per-device P2P identity, Automerge actor | Device-local, regeneratable |

### 2.2 Identity Document (TOML)

```toml
[identity]
public_key = "ml-dsa-xxxx..."
# master_key is NEVER stored — only derived per-session

[devices]
# Each device is registered by signing its device key with the identity key
device_keys = [
  { id = "device-a", public_key = "ed25519:abc...", label = "Laptop" },
  { id = "device-b", public_key = "ed25519:def...", label = "Phone" },
]
```

### 2.3 Multi-Session Resolution

When two devices of the same user connect to a lodge:
- They present different Automerge actor IDs
- Each presents a signed statement: `ActorID_X signed_by(IdentityKey)`
- The lodge deduplicates: same identity key = same user
- UI shows: `Alice (Laptop)` and `Alice (Phone)` or groups as `Alice [2]`

### 2.4 Device Revocation

If a device is lost:
1. User signs a revocation message: `"Device Key X revoked at timestamp T"` with their identity key
2. Revocation broadcast via Gossipsub to known lodges
3. Future ops from that device key are rejected by all peers
4. The device key is added to a local revocation list persisted in SQLite

## 3. Lodge Ownership Model

### 3.1 Lodge Manifest

Every lodge has a signed manifest created by its founder:

```rust
struct LodgeManifest {
    lodge_id: Uuid,
    name: String,
    created_at: i64,
    founder_public_key: Vec<u8>,  // ML-DSA public key
    founder_signature: Vec<u8>,   // Signature proving ownership
    policy: AccessPolicy,          // "open" | "invite_only" | "key_required"
}
```

### 3.2 Capability-Based Delegation (UCAN-style)

Lodge capabilities are signed tokens:

```rust
struct LodgeCapability {
    lodge_id: Uuid,
    issuer: Vec<u8>,            // public key of issuer (root or delegate)
    subject: Vec<u8>,           // public key of recipient
    permissions: Vec<Permission>, // [Read, Write, Admin, Invite, Remove]
    expires_at: i64,            // expiry timestamp
    conditions: CapabilityConditions, // optional constraints
    issuer_signature: Vec<u8>,  // signature over all above
}
```

Permission types:
- `Read` — sync and read document contents
- `Write` — edit documents
- `Invite` — invite new members (delegate capabilities)
- `Remove` — remove members
- `Admin` — full control including deletion

### 3.3 Capability Attenuation

A delegate can issue sub-capabilities that are strictly equal or narrower:
- Cannot grant permissions they don't have
- Cannot extend expiry beyond their own
- Cannot widen conditions

### 3.4 Lodge Deletion Protocol

```
Creator                            Peers
   │                                  │
   ├── Sign Tombstone ──────────────► │
   │   {lodge_id, reason, timestamp,  │
   │    signature}                    │
   │                                  │
   │                                  ├── Validate signature
   │                                  ├── Stop syncing CRDT data
   │                                  ├── Mark lodge as tombstoned
   │                                  ├── Remove from UI
   │                                  └── Optionally delete local data
   │                                  │
   └── Creator's local ──────────────┘
       delete + tombstone broadcast
```

A tombstone is a signed message broadcast via Gossipsub on the lodge topic:

```rust
struct LodgeTombstone {
    lodge_id: Uuid,
    reason: String,
    timestamp: i64,
    new_lodge_id: Option<Uuid>,  // redirect to replacement lodge
    founder_signature: Vec<u8>,  // ML-DSA signature of founder
}
```

**Properties:**
- Only the founder (root key) can sign a tombstone
- No capability delegation can grant tombstone permission — it's inherent to the root
- A tombstoned lodge is dead: peers will refuse all operations on it
- The founder can optionally provide a redirect to a new lodge ID
- This is a "soft delete" — peers may still have the data locally. True deletion is impossible in P2P.

### 3.5 Recovery / Key Loss

If the founder loses their identity key:
- At lodge creation, the founder may designate M-of-N recovery keys
- Recovery keys are stored in the lodge manifest at creation time
- To recover: M-of-N recovery keys sign a `RecoveryClaim` → replaces root key
- Without recovery keys set: the lodge is permanently orphaned

## 4. SQLite Schema Changes

Add to `database/mod.rs`:

```sql
CREATE TABLE IF NOT EXISTS identity_keys (
    id INTEGER PRIMARY KEY,
    public_key BLOB NOT NULL,
    label TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    is_device INTEGER DEFAULT 1,
    revoked INTEGER DEFAULT 0,
    revoked_at TEXT
);

CREATE TABLE IF NOT EXISTS lodge_capabilities (
    id INTEGER PRIMARY KEY,
    lodge_id TEXT NOT NULL,
    issuer_key BLOB NOT NULL,
    subject_key BLOB NOT NULL,
    permissions TEXT NOT NULL,   -- JSON array
    issued_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    signature BLOB NOT NULL
);
```

## 5. Membership Flow

```
Create Lodge:
  1. Generate LodgeManifest with founder's ML-DSA key
  2. Sign manifest
  3. Create Gossipsub topic: lodge:{lodge_id}
  4. Broadcast manifest on topic
  5. Peers validate founder_signature against manifest content

Join Lodge:
  1. Receive LodgeManifest (via invite link or discovery)
  2. Validate founder_signature
  3. Generate CapabilityRequest → send to founder (or delegated inviter)
  4. Founder signs a LodgeCapability → send back
  5. Present capability when syncing — peers validate before accepting ops

Leave Lodge:
  1. Sign a LeaveMessage with device key
  2. Broadcast on lodge topic
  3. Remove lodge from local UI
  4. Peers mark device as departed

Kick Member:
  1. Admin signs RevokeCapability for the member's device key
  2. Broadcasts revocation
  3. All peers reject future ops from that device key
```

## 6. Implementation Plan

### Phase 1: Foundation
- [ ] Add identity key generation (ML-DSA root + ed25519 device keys)
- [ ] Implement identity document save/load with encrypted root key
- [ ] Extend SQLite with identity_keys and lodge_capabilities tables
- [ ] Write unit tests for key generation, signing, verification

### Phase 2: Capability System
- [ ] Implement LodgeCapability struct, serialization, and validation
- [ ] Implement capability attenuation (issue sub-capabilities)
- [ ] Implement capability revocation
- [ ] Integrate capability checks into NetworkManager sync
- [ ] Write property-based tests for capability attenuation invariants

### Phase 3: Lodge Lifecycle
- [ ] Implement LodgeTombstone signing and validation
- [ ] Implement recovery key scheme (M-of-N)
- [ ] Add multi-device identity resolution (UI groups cursors by identity)
- [ ] Write integration tests: 2-device same-user scenario

### Phase 4: UI Integration
- [ ] Add "Members" panel to editor view showing device badges
- [ ] Add "Transfer Ownership" and "Delete Lodge" commands
- [ ] Add device management in config UI
- [ ] Add invitation flow (share capability via QR/link)
