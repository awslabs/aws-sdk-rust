# HTTP Connection Pool

## Requirements

### Existing HTTP client behavior is preserved

For equivalent configuration, the client preserves the existing client's observable behavior for HTTP
and HTTPS, HTTP/1.1 and HTTP/2, direct and proxied connections, DNS overrides, connect and read
timeouts, connection poisoning, and connection metadata capture.

This includes request-target form, proxy authentication, TLS negotiation, timeout scope, response-body
ownership, and error classification. Any difference requires an explicit compatibility
decision rather than an implicit change in the pool.
[Establishment failure classification](#establishment-failure-classification) records one such
decision: how failures before a connection's transport is established are classified.

### Connections to one origin are bounded

`max_connections_per_host = N` bounds admitted connections to one origin across every partition.
An origin is a scheme, host, and port, so HTTP and HTTPS are bounded separately, as is each
non-default port. Connecting, handshaking, open active, and open idle connections count against the bound.
Each origin has an independent bound. The default is unbounded, and a configured value of zero is rejected.

The bound applies to connections admitted by the pool, not to sockets still held by the operating system
while replaced connections finish tearing down. It is therefore not a file-descriptor ceiling;
[connection retirement](#connection-retirement-and-maintenance) defines that distinction.

### Connection placement follows declared topology

The caller declares the fixed partition set and the maximum scope of connection reuse. Each partition is
a placement scope for establishment and protocol drivers, with an optional network interface. A
connection's transport and protocol tasks remain on the partition that created it for their lifetime;
cross-partition reuse moves dispatch authority, not I/O placement, interface binding, or accounting.

Without explicit partitions, the pool has exactly one anonymous, unbound partition. It binds to one Tokio
runtime on first establishment and may be used across that runtime's worker threads.

### Eligible requests make progress

A request that can neither reuse nor create a connection parks without polling. Under the stated executor
and connection-progress assumptions, scheduling is work-conserving among eligible waiters, later arrivals
do not bypass a committed waiter indefinitely, and each resource grant performs work bounded independently
of waiter and partition count.

The guarantee applies while an eligible reusable connection, a reclaimable HTTP/1 transition, or a
released permit can become reachable. [Liveness](#liveness) defines the progress assumptions and the
cross-scope HTTP/2 condition in which no such resource becomes reachable.

### Local reuse is partition-local and bounded

A local reuse hit performs bounded work independent of partition count. It consults no origin-wide
coordination and reads no other partition's state. An unbounded origin constructs no admission, peer-index,
or cross-partition ordering machinery; cross-partition coordination begins only after local reuse misses on
a bounded origin with no free capacity.

### Coordination cost is bounded

Pool locks are never nested. Admission selects from maintained origin or
eligibility-group indexes rather than scanning partitions or connections, and
each detached action performs bounded work before it either settles or produces
one successor. Cross-partition H2 reuse moves generation identity, not transport
ownership. Changes that claim to preserve local-path cost must show that they add
no origin-wide coordination, partition-wide scan, or shared allocation to a
steady local reuse hit.

### Pool behavior and state are observable

The pool reports connection lifecycle events and per-partition statistics sufficient to diagnose
establishment, reuse, waiting, logical close, and physical teardown. Events identify the origin and owning
partition, and installed connections also identify the stable connection and negotiated protocol.
Statistics distinguish establishment, admitted H1 and H2 state, draining state, active H2 streams, waiters,
and physically live transports.

## Architecture

The architecture proceeds from topology and ownership through local selection,
establishment, bounded coordination, dispatch, retirement, and telemetry. The
model below summarizes the state and request path; the later sections define
each contract.

### The model

The pool sits between Hyper and a connector. Hyper provides the HTTP/1.1 and HTTP/2 implementations:
[`handshake`](https://github.com/hyperium/hyper/blob/v1.11.0/src/client/conn/http1.rs#L140) yields a
[`SendRequest`](https://github.com/hyperium/hyper/blob/v1.11.0/src/client/conn/http1.rs#L23) dispatch handle
and a connection driver future, and behind those live the request/response state machines, HPACK, and flow
control. Transport arrives as a connector, a `Service<Uri>` yielding `(IO, Connected)`; the pool builds on
that interface without altering it, so the connectors this client already assembles for TLS, proxies, and
tests compose unchanged.

What the pool owns is the layer between them: which connection a request dispatches on, when establishment
starts, how much capacity exists and who holds it, whether a connection may be reused from another runtime,
and when an idle connection closes.

#### Topology

Connections are indexed by two things that vary independently.

A **partition** is a placement scope for connection establishment and drivers, plus an optional network
interface its sockets bind to. An explicit partition names its runtime; the anonymous partition binds to the
current Tokio runtime on first use. The set is fixed at construction.

An **origin** is a scheme, host, and port — the web's origin as
[RFC 6454](https://www.rfc-editor.org/rfc/rfc6454) defines it, canonicalized so two spellings of one server
are one origin. The pool discovers origins lazily from requests rather than declaring them at construction.

A connection belongs to exactly one of each: one partition established it, and it can serve one origin. Their
intersection is an **`OriginCell`**, which holds the connections a partition has for an origin and is created
on first use of that pair.

```text
ConnectionPool
|-- Partition P0 (runtime 0, eth0)
|   |-- OriginCell(P0, s3)
|   `-- OriginCell(P0, dynamodb)
|-- Partition P1 (runtime 1, eth1)
|   `-- OriginCell(P1, s3)
`-- bounded-origin coordination
    |-- OriginAdmission(s3)
    |     `-- P0 and P1 cells
    `-- OriginAdmission(dynamodb)
          `-- P0 cell
```

Because the partition set is fixed and the origin set is not, partitions are the outer level: each partition
owns its own map from origin to cell, so the structure that grows is always inside one partition. A cell that
no request has asked for does not exist.

An **`OriginAdmission`** holds what all partitions sharing a bounded origin must agree on: its capacity budget,
demand schedule, and H1/H2 supply indexes. It stores the shared `OriginKey` once and keys its internal cell,
demand, and supply records by `PartitionId`; the origin component is invariant inside this authority. Nothing
else spans partitions.

#### Ownership and lifetime

```text
ConnectionPool
`-- PartitionRegistry
    |-- PartitionState by PartitionId
    |   `-- OriginCell by OriginKey       local waiters and protocol records
    `-- OriginAdmission by OriginKey      bounded-origin permits and ordering

Client
|-- ConnectionPool
`-- resolved PartitionState
```

| Type              | Created                                | Destroyed                                     | Shared across partitions |
| ----------------- | -------------------------------------- | --------------------------------------------- | ------------------------ |
| `ConnectionPool`  | by the builder                         | when the last pool, client, and request release it | —                        |
| `Partition`       | at construction, from the declared set | at pool drop                                  | no                       |
| `OriginCell`      | first request for (partition, origin)  | not while the origin is live                  | no                       |
| `OriginAdmission` | first request for a *bounded* origin   | not while the pool lives                      | yes                      |
| `Client`          | by the caller, freely                  | by the caller                                 | —                        |

`Client` is what a caller holds and what implements the smithy runtime's `HttpClient`. It pairs the pool with
one resolved partition, so a request never searches for its partition — the handle already names it.

Each mutable authority has one lock domain:

| Authority | Owns |
| --- | --- |
| Admission registry | One admission authority for each bounded canonical origin. |
| Partition origin map | Stable cell identity for one partition and canonical origin. |
| Origin admission | Available capacity, canonical demand, indexed H1/H2 supply, and retained H1 matches. |
| Origin cell | Local acquisition order, H1 sender state, H2 flights and generations, and supply revisions. |
| Connection lifecycle | Dispatch eligibility, accepted-dispatch count, bounded capacity, and physical connection completion. |
| H2 request claim | Upload and response completion for one prospective or accepted request. |
| Partition maintenance | Idle deadlines, wake publication, and shutdown. |

No transition holds two pool locks at once. Values that cross lock domains own
their rejection or drop fallback until the receiving authority commits them.

An `OriginAdmission` exists only for a bounded origin. A local miss normally
establishes on the requesting partition. When no permit is free, admission may
use compatible peer protocol state or reclaim peer capacity for that demand.
An unbounded origin never needs cross-partition admission or peer indexes; its
cells are independent. Reuse scope controls which peer protocol state is
compatible, while reclaim may recover capacity across eligibility groups.

Pool retention, request accounting, physical connection ownership, and bounded capacity
have different lifetimes.

```text
pool lifetime

caller-held Client ------------------------------> ConnectionPool
request future, until response head/error -------> ConnectionPool
```

A `Client` and an in-flight request through response headers retain the pool.
Producing a response head or terminal error ends the request future's pool
hold. Protocol-specific guards own the remaining cleanup:

```text
post-header protocol lifetime

H1Exchange <------------------ response body or readiness task
PhysicalConnectionGuard <----- driver or upgraded root I/O

H2 request claim
  |-- response guard <--------- response body or upgrade bridge
  `-- upload guard <------------ accepted H2 request-body adapter
```

An H1 exchange owns Hyper's exclusive HTTP/1 request handle (`SendRequest`) and
returns it only after a reusable message boundary. An
H2 request claim releases only after both request sides finish. Root I/O
may move from the driver into an upgrade while the same
`PhysicalConnectionGuard` tracks client ownership. Bounded capacity has a
separate owner path:

```text
bounded connection capacity

OriginAdmission
  `-- issue --> EstablishmentPermit
                  +-- failure/drop ------------------------> OriginAdmission
                  `-- install --> ConnectionState owns CapacityLease
                                      `-- logical close ---> OriginAdmission
```

A bounded permit moves from admission to establishment and then to the
installed `ConnectionState`. Logical close returns it. Dispatch handles,
`DispatchGuard`, and H2 request claims never own a connection permit.

HTTP/1 and HTTP/2 share admission but not connection semantics:

| HTTP/1 | HTTP/2 |
| --- | --- |
| One request exclusively owns a sender. | Many requests may share one generation. |
| Admission may retain an H1 match while a supplier-cell reservation crosses locks. | A route is detached identity-only work; admission retains no matching route lifecycle. |
| Peer reuse transfers sender ownership to the requesting cell. | Peer reuse follows a route to the connection-owning generation. |
| Borrow and reclaim serve the origin demand head; an incompatible head may deliberately block younger groups. | Route-ready eligibility groups rotate without consuming origin capacity. |
| Reclaim closes a provisional H1 candidate or an idle H1 record. | Reclaim closes one exact idle generation. |
| Reuse ends at one complete HTTP/1 exchange boundary. | An accepted request claim ends after both upload and response sides finish. |

These differences are protocol properties, not parallel abstractions waiting to
be commonized. Shared code owns only the capacity, demand, and detached-action
mechanics that have the same authority in both protocols.

#### Request path

A request resolves its partition-local cell and first attempts compatible
local selection. A miss registers one acquisition. Capacity and available
protocol state determine how that acquisition completes.

```text
Client(partition P, request URI)
`-- resolve OriginCell(P, origin)
    |
    |-- compatible local connection ------------------> dispatch
    |
    `-- local miss -> register one acquisition
        |
        |-- unbounded origin or free permit
        |   `-- establish on P owner runtime ----------> dispatch
        |
        `-- bounded origin at capacity
            |-- compatible peer connection ------------> dispatch
            |-- reclaimable peer capacity
            |   `-- close peer; transfer permit
            |       `-- establish on P owner runtime --> dispatch
            `-- otherwise park until state changes

selected protocol handle
`-- commit against logical close -> Hyper
    |-- request completes ------------> return or release protocol state
    |-- protocol upgrade ------------> caller owns upgraded lifecycle
    `-- connection-terminal failure -> logical close, then physical connection ownership ends
```

[Local connection selection](#local-connection-selection) defines local selection.
[Connection establishment](#connection-establishment) defines establishment, placement, and protocol
convergence. [Bounded-capacity coordination](#bounded-capacity-coordination) defines parking, borrow, reclaim,
and resource delivery, with [Liveness](#liveness) stating when those paths guarantee progress.
[Dispatching and completing a request](#dispatching-and-completing-a-request) defines request preparation,
stale-reuse retry, and the transfer to response or upgrade ownership. The path ends in
[return or retirement](#connection-retirement-and-maintenance), where each terminal outcome either makes the
connection reusable or closes it.

### Topology and identity

The model introduces the partition-origin shape. This section defines the public identities, placement
contract, stable cells, and canonical origin key that implement it.

#### Partitions

`PartitionId` is a stable caller-chosen identity for an explicit partition. `Partition` packages that identity,
a driver spawner, and an optional network interface as immutable construction-time state. The fields remain
private because callers configure placement through constructors rather than inspecting its representation.

```rust
/// Stable identity used to construct clients and correlate telemetry.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PartitionId(/* private */);

impl PartitionId {
    /// Reserved identity for the implicit default partition.
    pub const ANONYMOUS: Self;
    pub const fn from_index(index: usize) -> Self;
    pub const fn is_anonymous(self) -> bool;
}

/// Construction-time placement for connections and their protocol drivers.
pub struct Partition {
    /* private: identity, driver spawner, and optional network interface */
}

impl Partition {
    pub fn new(id: PartitionId, spawner: DriverSpawner) -> Self;
    pub fn interface(self, nic: impl Into<String>) -> Self;
}

/// Runtime placement for one partition's connection-owned work.
pub struct DriverSpawner {
    /* private: type-erased placement implementation */
}

impl DriverSpawner {
    pub fn tokio(handle: tokio::runtime::Handle) -> Self;
}
```

For example, a thread-per-core caller can declare one partition from the identity and runtime it already
maintains:

```rust
Partition::new(PartitionId::from_index(core), DriverSpawner::tokio(Handle::current()))
    .interface("eth0")
```

The caller declares partitions and the pool infers none. Whether two runtimes should share connections
depends on why they are separate, and that reason exists only in the caller's design: a thread-per-core
service separates runtimes for cache locality, a multi-tenant host for isolation, a multi-interface host to
drive independent links. The pool can observe that several runtimes exist but not which of these is true, and
the three want different behavior.

Drivers are spawned only through their partition's `DriverSpawner` and never move once spawned. That is what
keeps a connection's I/O on the runtime that established it, whichever partition later dispatches on it: a
reused connection carries only its dispatch handle across the boundary, never its driver.

`Partition::interface` configures placement through the default HTTP connector.
The binding is applied before connect, so a connected socket retains its egress
placement when another partition uses its dispatch authority. Interface
existence, permissions, and other host-specific failures are connector errors
reported during establishment. Custom connector construction has no pool-level
interface-placement contract.

A pool with no declared partitions has exactly one unbound owner partition. Its first use binds that anonymous
partition to one Tokio runtime for connection-owned work; requests may originate on that runtime or on other
runtimes, and establishment, drivers, and pending return work are submitted back to the captured owner.
Partitions without an interface compare as one group, so the common case performs no per-request interface
work. The anonymous partition has the reserved identity `PartitionId::ANONYMOUS`, used by events and
statistics; callers cannot declare it explicitly. Every explicit identifier is caller-owned. A thread-per-core
caller can therefore reconstruct `PartitionId::from_index(thread_id)` when it declares the
topology, creates each thread's client, and reads per-partition statistics, without plumbing pool-issued
handles between those sites.

`DriverSpawner::tokio` takes the handle of the runtime that owns the partition's connections and spawns
on it regardless of which thread invokes `spawn`. A caller that wants the constructing runtime passes
`Handle::current()`, whose panic outside a runtime is Tokio's own. No spawner is supplied by the caller
for the anonymous partition, which captures its runtime on first use as
[Connection establishment](#connection-establishment) describes. Under the `test-util` feature, a hidden
`DriverSpawner::from_fn` adapts a spawn function so placement tests can observe task submission; it is
not a supported extension point.

A spawner is placement, not runtime ownership. It does not keep its runtime alive and takes no part in
shutdown. A runtime that drops spawned work is observed by the pool through the connection's own close
path. The placement contract is type-erased behind `DriverSpawner` so the pool can extend it without
changing the public type.

##### Alternatives

**Pool-issued opaque partition handles.** Partition identity almost always derives from a numbering the caller
already maintains — a thread index, a core index, a worker number — and the caller needs that identity at
several independent sites: declaring partitions, constructing per-thread clients, and correlating statistics
back to threads or interfaces. An opaque handle must be plumbed to every one of those sites, and any caller
keying a map by partition needs it hashable, which reintroduces an identifier with extra steps.

**Deriving partitions from runtime detection.** Requires the pool to answer why runtimes are separate, which
it cannot observe.

#### Origins and cells

`OriginKey` is the owned public identity used by statistics, events, and callers that need to name an origin.
It semantically contains an HTTP or HTTPS scheme, a canonical host, and an optional non-default port. Its
storage representation is private so lookup and retained-key storage can evolve independently.

```rust
/// An owned, canonical HTTP or HTTPS origin.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct OriginKey {
    /* private: scheme, canonical host, and optional non-default port */
}

impl OriginKey {
    pub fn from_uri(uri: &Uri) -> Result<Self, InvalidOrigin>;
    pub fn from_parts(
        scheme: Scheme,
        host: impl AsRef<str>,
        port: Option<u16>,
    ) -> Result<Self, InvalidOrigin>;

    pub fn scheme(&self) -> &Scheme;
    pub fn host(&self) -> &str;
    pub fn port(&self) -> Option<u16>;
}

#[derive(Debug)]
pub struct InvalidOrigin { /* private */ }
```

Two connections are interchangeable exactly when they share an origin, which is why this and not something
narrower or wider is the key: a server enforces its per-client connection limit at this granularity, and TLS
parameters are negotiated for it. `https://example.com` and `https://example.com:8443` are distinct origins,
as are the `http` and `https` forms of one authority.

The key is a canonicalized origin, not the raw `(scheme, authority)` a URI carries, because two spellings of
one server must not become two origins — each with its own `OriginAdmission`, together admitting twice the
bound against a host that sees one. Canonicalization elides the scheme's default port, so `https://x` and
`https://x:443` are one key, and drops userinfo, which the origin does not include and TLS does not vary on.
Host comparison is already ASCII-case-insensitive. Two spellings are not unified: an
internationalized host and its punycode form (the connector resolves what it is given; equating them would
pull in Unicode normalization), and a fully-qualified name with a trailing dot, which is a distinct DNS name.
IPv6 literals are parsed and normalized to the standard compressed spelling so equivalent address text has
one identity. A URI zone identifier remains case-sensitive and is retained exactly; it is an opaque
interface-scoped value rather than a DNS name. Unknown IP-literal forms are not case-folded.

The request path does not construct an owned `OriginKey` merely to probe a partition's origin map. It first
builds a private canonical lookup key that borrows the URI host when its bytes already have canonical form and
owns temporary host storage only when normalization requires it. One possible private representation uses
`Cow<'a, str>` for the host; the exact equivalent-key and map machinery is not part of the public contract.

```text
URI -> private canonical lookup key
         +-- host already canonical ----> borrow URI host
         `-- host needs normalization --> temporary owned host
                    |
                    +-- map hit  -> use existing cell; discard lookup key
                    `-- map miss -> convert once to owned OriginKey and insert
```

An already-canonical hit therefore allocates no host storage. A miss converts the lookup key into the map's
owned key once; a non-canonical hit may allocate temporary normalized bytes but does not retain another key.
`OriginKey::from_uri` and `OriginKey::from_parts` instead construct owned public values and perform the same
canonicalization. `from_parts` lets an observer construct a key once without reparsing a URI for every
statistics sample. It validates the host with the structured HTTP authority parser. Both constructors reject a
scheme other than HTTP or HTTPS, an invalid host or port, and any input that does not name an origin.
`InvalidOrigin` carries the offending component and source error for diagnostics, implements `Error`, and
exposes no second, less strict public key representation.

Canonicalization distinguishes an absent port from malformed, zero, or
out-of-range port text. Invalid explicit ports cannot alias the scheme default.

The request's HTTP version is absent. A request marked HTTP/1.1 may dispatch on an HTTP/2
connection, so version is a dispatch-eligibility question decided per connection, not an identity question
decided per origin. Including it would split one origin's connections into two populations that cannot share
capacity, and would make the pool's shape depend on which requests happened to arrive first.

A URI without both a scheme and an authority names no origin and is rejected before any lookup, rather than
being mapped to a placeholder.

#### Stable identity

A cell is not destroyed while its origin is live, and an origin's state is not destroyed while the pool
lives.

Stability is what lets one partition act on another's cell without checking whether it is still there. When a
request cannot find a connection locally it looks to peer cells for the same origin, and every such
cross-partition operation names a cell it did not create. If a cell could disappear between selection and
use, each reference would need a liveness check and a generation to detect a reused slot — and those checks
would sit on the path taken when a request has *already* failed to find a connection, where latency is least
affordable.

Stability is needed only within an origin. A permit belongs to one origin's admission and cannot be spent on
another; a connection to one origin cannot serve a request for a different one. No reference crosses origins,
so an origin and all of its cells form a unit that could in principle be removed together, while removing one
cell from a live origin could not.

The cost is retained memory. Cells accumulate as the origin set grows and are never reclaimed, bounded by
partitions × origins ever touched, so a many-core client reaching many origins retains cells long after their
connections are gone. Stable identity retains those cells; see
[Reclaiming quiescent origins](#reclaiming-quiescent-origins) for reclamation constraints.

##### Alternatives

**Evicting individual cells that hold no connections.** Reclaims memory at the granularity that breaks the
property above: a peer cell selected for a cross-partition operation could be freed before it is used, so
every such reference would carry a generation and a validity check on the slow path. Whole-origin removal
needs neither, because nothing outside an origin references its cells.

#### Obligations

Cells and their identity:

* **Cell stability** [safety] — a cell is not destroyed while its origin is reachable from the pool.
* **Reference validity** [safety] — a reference to a peer cell for a live origin requires no liveness check
  before use.
* **Cell uniqueness** [safety] — at most one cell exists per (partition, origin) pair; concurrent first use
  produces exactly one.
* **Lazy cells** [optimization] — a (partition, origin) pair no request has named has no cell.

Partitions:

* **Driver placement** [safety] — a connection's driver is spawned only through its partition's
  `DriverSpawner`, and never migrates.
* **Binding immutability** [safety] — a partition's interface binding is fixed at construction and applied
  before connect.
* **Default partition** [safety] — a pool with no declared partitions has exactly one anonymous partition;
  its first use binds one owner runtime, every later establishment and driver uses that runtime, and request
  tasks may execute on another runtime.
* **Interface comparison cost** [optimization] — comparing two unbound partitions performs no string work.

Origins:

* **Key totality** [safety] — every dispatched request maps to exactly one origin; a URI lacking scheme or
  authority is rejected before lookup.
* **Canonical key** [safety] — origins equivalent under default-port elision and userinfo removal map to one
  key, so one server is one `OriginAdmission`.
* **Version independence** [safety] — the request's HTTP version does not participate in the origin key.
* **Allocation-free canonical hit** [optimization] — looking up an already-canonical request origin allocates
  no host storage; only normalization or insertion may own host bytes.

### Smithy client boundary

The smithy runtime selects an HTTP connector for an operation by calling
`HttpClient::http_connector(settings, components)`. `Client` returns a `SharedHttpConnector` around a private
`PoolConnector`. This is a cheap request-policy facade over the client's resolved partition, not another pool
or transport stack: constructing one performs no DNS, TLS, Hyper, or admission setup.

```text
Client(pool, partition P): HttpClient
  |
  +-- http_connector(settings A, components A) -> PoolConnector(pool, P, policy A)
  `-- http_connector(settings B, components B) -> PoolConnector(pool, P, policy B)
                                                        |
                                                        `-- one ConnectionPool
                                                            `-- one OriginAdmission per bounded origin
```

The facade clones the complete non-exhaustive `HttpConnectorSettings` and extracts the operation components
its request policy uses. Its `call` moves a `Client` clone and that policy into the request future, which is the
strong pool reference already shown above. A facade may be cached as an implementation optimization, but its
identity is never a partition, origin, pool, or admission key. Differing connect or read timeouts therefore do
not multiply `max_connections_per_host`: every facade for one `Client` reaches the same origin-wide admission
authority.

Connect and read timeouts remain operation policy rather than connection identity. The facade uses the
operation's `AsyncSleep` from `RuntimeComponents` when present, then the same default-sleep fallback as the
existing client. A configured timeout requires the resulting sleep implementation; its absence never disables
the timeout. The read timeout bounds `PoolConnector::call` through response headers, including time spent
waiting for acquisition or a shared HTTP/2 flight. A connect timeout wraps only the transport connector through
TLS and ALPN. Once the connected transport moves to the connection-partition owner task, neither request
timeout supervises or cancels that task's Hyper handshake; the pool has no separate handshake timeout.
A request joining an existing HTTP/2 flight owns no connector operation, so its connect timeout does not apply
to that flight. Its read timeout or caller cancellation removes that participant while the owner task
continues. A fully cancelled successful flight may therefore install an idle generation that remains until
ordinary close or idle expiration.

Idle maintenance is pool policy, not operation policy. Its `TimeSource` and `AsyncSleep` are fixed by the pool
builder and shared by the partition maintenance tasks; they do not depend on which operation first asks for a
facade. Operation `RuntimeComponents` may vary between smithy clients sharing a pool without changing idle
age or creating another pool.

`Client::validate_base_client_config` runs an idempotent transport preflight so native trust roots, when
applicable, load when this HTTP client is selected rather than on its first request. The preflight opens no
socket and creates no settings-keyed pool or connector cache. `validate_final_config` remains a no-op.
`connector_metadata` reports `hyper/1.x`, preserving the HTTP client identifier used in user-agent metadata.
`Client` and `PoolConnector` use bounded custom `Debug` implementations that report immutable configuration
and the resolved partition, not live origin or cell state.

### Local connection selection

A request arrives on a `Client`, which already names its partition. Reuse is therefore two steps: the
partition's origin map, then the cell.

```text
request on partition P for origin O
  |-- P already resolved by Client
  `-- P.origins[O]                     partition-local origin lookup
      `-- select compatible state      one OriginCell lock
```

That is the entire path for a reuse hit. It performs no origin-wide coordination, reads no other partition's
state, and touches no `OriginAdmission` or peer index. Its synchronization is
the requesting partition's own cell lock. A peer may acquire that lock to
reserve or settle a bounded H1 match, so the lock can be contended, but the
local hit does not consult origin-wide state. Its work is
independent of partition count, and traffic for another origin does not share
the cell.

A reused connection may be dead: the server can close an idle connection while it sits in the cell, and the
pool learns this only on dispatch. So "take a live idle connection" is provisional until the request is
accepted. Hyper's `try_send_request` returns the unsent request when the connection failed before accepting
it, and that returned request is the retry boundary — a reused connection that fails before acceptance is
transparently retried on a fresh one, invisibly to the caller. A request the connection had already accepted
is not retried here; whether to retry it is the caller's policy, because the pool cannot know the request was
not acted on. This distinguishes a *reused* connection, where a pre-acceptance failure is the expected
stale-idle race and is absorbed, from a *fresh* one, whose failure is a real error the caller sees.

On a local miss, one acquisition attempt may wait for a compatible H1 return
while preparing establishment. The returned H1 and establishment result compete
to complete the launching waiter, and exactly one result commits. If H1 wins
before the connector is first polled, connector work remains lazy and tentative
capacity returns to admission. Once connector polling begins, the pool owns the
establishment attempt through completion even if another H1 serves the
launching waiter. A successful result that loses this race remains available
for later compatible demand; failure releases the attempt's resources without
replacing the result already delivered.

If establishment wins, a concurrent H1 return follows ordinary owning-cell
return handling. If no capacity is available, no establishment attempt starts
and the request waits for bounded capacity or cross-cell reuse. Cancellation
removes the launching waiter but does not cancel a started establishment;
every connection, attempt, lease, and waiter retains one terminal owner.

#### Alternatives

**An origin-keyed map at the top, owning its cells.** Groups a bounded origin's budget with the cells it
governs, which reads well and is how a pool is usually drawn. Its cost is a structure shared by every
partition, growing with the origin count, on the path of every request including local reuse hits: concurrent
small requests across many partitions serialize on it to reach state none of them share. Measured on an
implementation shaped that way, small-object throughput fell by roughly an order of magnitude against the
unpooled client, and the shared structure on the reuse path was the cause.

#### Obligations

* **Local hit locality** [optimization] — a reuse hit performs no origin-wide coordination, reads no other
  partition's state, and does work independent of partition count. Its synchronization is the requesting
  partition's own cell lock, which a peer may also acquire under bounded pressure.
* **Partition-count independence** [optimization] — the work of a reuse hit does not grow with the number of
  partitions.
* **Stale-reuse retry** [safety] — a reused connection that fails before accepting the request is retried on a
  fresh connection; a connection that has already accepted the request is not retried by the pool.
* **Acquisition race ownership** [safety] — a compatible local H1 return and an establishment result commit to
  at most one launching waiter; connector work stays lazy before its first poll, and an attempt that has been
  polled completes under pool ownership independently of that waiter.

### Connection establishment

When a request finds no live connection for its origin on its partition, the cell establishes one. The steps
are the same whether or not a bound is configured: acquire capacity if the origin is bounded, run the
connector to obtain transport, hand the transport to Hyper to get a dispatch handle and a driver, place the
driver, and record the connection in the cell. What follows is where each step runs and who holds the
connection's capacity while it does. The case where a bound is set and no capacity is available is deferred
to [Bounded-capacity coordination](#bounded-capacity-coordination); this section assumes establishment may
proceed.

#### Establishment

A connection's socket, driver, and reactor must all live on one runtime, because Tokio registers a socket with
the I/O reactor of whatever runtime creates it — it captures the current runtime's handle at socket creation
([`poll_evented.rs:111`](https://github.com/tokio-rs/tokio/blob/tokio-1.53.1/tokio/src/io/poll_evented.rs#L111))
— and the connector is what creates the socket. If the driver ran on one runtime while the socket was created
on another, every readiness event would cross runtimes, and the socket's reactor would outlive or predecease
the driver that holds it. So establishment — connector, transport, TLS, ALPN, handshake — and the driver it
produces run on the same runtime.

Connection establishment and installed connection lifetime are separate
ownership phases. The establishment authority exists before DNS and owns the
connector future and any bounded-origin permit. After DNS, socket connection,
proxy negotiation, TLS, and ALPN produce connected I/O with a selected HTTP
protocol, the pool creates `ConnectionState` in `PendingOpen`. At that point
transport establishment and protocol selection are complete, but Hyper has not
produced the request handle required for dispatch. Successful Hyper protocol
setup moves the state to `Open` and attaches the bounded-origin permit before
cell installation makes the connection discoverable. Lifecycle observation
treats failed DNS, transport, and TLS attempts independently from
installed-connection events; failed attempts have no synthetic
`ConnectionState`.

An **explicit partition** names its owner runtime through `DriverSpawner`. The
**anonymous partition** captures the current Tokio runtime on first use. A
request may be polled on another runtime, so every new connection submits the
still-unpolled connector, transport, TLS/ALPN, and Hyper handshake future to
the partition owner. Completion updates the cell and wakes the requesting task;
the resulting driver and pending return work use the same spawner. This policy
costs one task submission and wake per new connection. Local reuse and dispatch
on an established connection do not pay that handoff.

The submitted establishment future carries its own completion guard. If a spawner discards the future before
polling it, or its owner task is dropped after polling begins, that guard completes the waiter with a terminal
error and drops the still-owned establishment permit or attempt. This is a narrow ownership fallback for the
submitted future, not runtime supervision: `DriverSpawner::spawn` retains Hyper's `spawn -> ()` contract and
does not claim to report runtime health synchronously. Once the first poll claims establishment, normal
attempt completion or this guard is responsible for completing the waiter exactly once.

This transfer keeps socket creation, handshake, and driver polling on one runtime while allowing a
partition-specific `Client` to move between independent requester runtimes. Dispatch may cross that boundary
through Hyper's request handle; connection I/O and the driver never do.

Hyper spawns work of its own, and it follows the connection. An HTTP/2 connection hands Hyper a connection
task at handshake and per-stream and upgrade tasks as it runs, through an
[`Executor`](https://github.com/hyperium/hyper/blob/v1.11.0/src/rt/mod.rs#L45) the caller
supplies; the pool supplies one that forwards to the connection's runtime. HTTP/1 uses the partition spawner
for its connection driver and for readiness work that outlives a response body.

#### Connection ownership

A bounded origin has one permit per admitted connection, and a *lease* is what owns it. Admission issues a
lease to the establishing task; a successful handshake transfers the lease to the connection record, which
holds it until logical close releases it. Requests dispatched on the connection take *handles*, not the lease,
so the many concurrent requests on an HTTP/2 connection share the single permit the record's lease owns. An
unbounded origin has no permit, no lease, and no such chain.

The lease is what makes the chain safe to drop: it is an RAII guard, so a connector error, handshake failure,
cancellation, or runtime shutdown returns the permit to admission before the lease passes to a connection
record. After that transfer, the record remains the sole lease owner and its logical-close transition is the
only path that releases the permit.

Admission stores free capacity as a count. Removing one unit creates a non-`Copy` `Permit` with a
never-reused diagnostic identity. Delivery materializes that value into the `CapacityLease`; lease return
increments the free count rather than storing returned permits. The representation avoids an allocation
on capacity return while the permit and lease types preserve linear ownership.

The spawned connection driver is wrapped by a **driver lifecycle guard** armed only after the record and its
generation-specific close authority exist. The guard holds a non-retaining close handle, not the lease. If
the driver completes, the wrapper requests logical close with `ProtocolClosed` and the driver's source error.
If the wrapper is dropped before completion, including when its owning runtime shuts down, the guard's
`Drop` requests logical close with `OwnerRuntimeShutdown`. Either request races through the record's existing
idempotent close transition, so a prior pool drop, poison, reclaim, or protocol close wins without releasing
capacity twice. Because the handle does not retain the pool, driver tasks cannot form a lifetime cycle with
the records they close. `PhysicalConnectionGuard` remains the separate authority for the end of
client-owned physical connection I/O. Dropping it releases the client's transport handle; the operating
system may continue TCP teardown.

This is capacity conservation on the create and driver paths: the permit has one
owner at every step, and cancellation either drops the establishing lease or logically closes the record
that received it.

#### Readiness

The connector is a Tower `Service<Uri>`, and the pool honors that contract: it drives the connector to ready
before calling it, and issues one call per readiness. The dispatch handle Hyper returns is a different thing.
Its readiness — whether the connection can accept another request — is an inherent
[`SendRequest::poll_ready`](https://github.com/hyperium/hyper/blob/v1.11.0/src/client/conn/http1.rs#L156),
not a Tower `Service`: Hyper has no Tower dependency, and the HTTP/2 form ignores its `Context` entirely
([`http2.rs:97`](https://github.com/hyperium/hyper/blob/v1.11.0/src/client/conn/http2.rs#L97)). Connection
readiness is thus a per-protocol dispatch question the pool answers against the connection's state, and does
not share machinery with the connector's Tower readiness. The two are related only by name.

#### Establishment failure classification

A failure while establishing a connection's transport — in DNS, the TCP connect, the TLS handshake, or a
proxy tunnel — happens before any part of a request is sent, so its classification answers one question:
could a new connection succeed? The pool reports a failure in the network as an I/O error, which is
retryable, and any other failure as an unclassified error with no error kind (`Other(None)`), which is
terminal.

| Failure                                                                                  | Result         |
| ---------------------------------------------------------------------------------------- | -------------- |
| Connect timeout                                                                          | timeout        |
| DNS, with any resolver                                                                   | I/O            |
| TCP or socket error of a network kind, with or without an operating-system errno         | I/O            |
| Other operating-system error, except permission, invalid argument, or unsupported        | I/O            |
| Handshake EOF or reset (rustls); s2n `IOError` or `ConnectionClosed`                     | I/O            |
| Proxy: the TCP connect to the proxy fails, or an I/O error on the tunnel                 | I/O            |
| rustls rejection; s2n alert, protocol, or other error                                    | `Other(None)`  |
| Proxy 407, other non-200 answer, response headers too long, or missing host              | `Other(None)`  |
| Bad scheme, URL, or server name                                                          | `Other(None)`  |

A `ConnectorError` returned by the transport keeps its own classification. An I/O error of a network kind —
`ConnectionRefused`, `ConnectionReset`, `ConnectionAborted`, `NotConnected`, `AddrNotAvailable`,
`NetworkUnreachable`, `HostUnreachable`, `NetworkDown`, `BrokenPipe`, `TimedOut`, `UnexpectedEof`, or
`WriteZero` — is a network failure even without an operating-system errno, because connectors and resolvers
also synthesize these errors from a kind alone. Any other operating-system error is a network failure unless
its kind is `PermissionDenied`, `InvalidInput`, or `Unsupported`, which a new connection cannot change; that
includes errors without a stable `ErrorKind`, such as `ENOBUFS`. An I/O error of any other kind without an
errno, such as `InvalidData`, is not a network failure. The pool's default resolver reports getaddrinfo
failures as `ResolveDnsError`, as a custom resolver does, because getaddrinfo's error does not say which
failures are permanent.

This differs from the existing client. The hyper-util legacy pool reports every connector failure as a
connect error, and the existing client classifies every connect error as I/O, so it retries permanent
failures such as a rejected certificate, a proxy's 407, or an unsupported scheme until the retry budget is
spent. The pool fails those at once and retries only failures a new connection can fix.

The TLS provider shims classify their own failures, because only they see the provider's error types. A
shim reports a failed handshake as a `TlsConnectError` whose kind is `Io` when the connection failed or
closed during the handshake and `Protocol` when the TLS layer refused or could not complete it. The rustls
shim tags only rejections, because a rustls I/O failure already arrives as an `io::Error`; the s2n shim tags
both kinds, because s2n reports an I/O failure without one. The pool's classifier reads `TlsConnectError`
and `io::Error` and contains no provider-specific code.

One network failure is terminal. A proxy that closes the connection while answering `CONNECT` produces
hyper-util's `TunnelUnexpectedEof`. That variant belongs to an unexported error type and has no source, so
the pool can recognize it only by its message text, which it does not match. It is reported as
`Other(None)`.

#### HTTP/1 attempts and HTTP/2 flights

One HTTP/2 connection carries many concurrent request streams. The pool calls
one installed incarnation of that connection a *generation*. Replacements
receive new generation identities so delayed routes, GOAWAY handling, close
work, and request completion cannot affect a newer connection. An
`H2Activation` is authority for a prospective request on one exact
generation; it becomes an accepted request claim only after Hyper accepts the
request.

The pool supports HTTP/1.1 and HTTP/2 and lets connector ALPN select the protocol. Each request has one of
three requirements. `H1Required` covers accepted request forms that require HTTP/1 wire semantics, including
HTTP/1.0, ordinary `CONNECT`, and Upgrade. `H1Compatible` may use either protocol. `H2Required` cannot dispatch
on HTTP/1. Request protocol is not part of the origin key.

The default rustls connector offers only `http/1.1` for `H1Required` and offers `h2, http/1.1` otherwise.
The s2n connector has a fixed HTTP ALPN offer. Connectors injected through the unstable test utility own
their protocol configuration and do not receive the pool's offer. Those paths may therefore negotiate H2
for `H1Required`; the pool rejects that result as a non-retryable protocol mismatch before Hyper establishment.
`H2Required` does not make an H2-only offer. If the server
selects HTTP/1, the pool retains the useful H1 connection for compatible demand and returns an
unsupported-version error with that connection's metadata to the launching request.

The two protocols establish differently because they reuse differently. An HTTP/1 connection carries one
request at a time, so a cell that needs more concurrency needs more connections: each establishment is an
independent *attempt*, and several may run at once, each producing a connection for the request that launched
it. An HTTP/2 connection multiplexes, so one connection serves the whole cell; a second connection to an
origin the cell already serves is nearly pure waste. HTTP/2 establishment is therefore a *flight*,
coordinated so the cell keeps at most one accepting HTTP/2 generation — a request arriving during a flight
waits for it rather than starting its own, and the resulting generation is published to compatible waiters.

##### Post-ALPN convergence

The initial API imposes no pool-wide HTTP/1-only or HTTP/2-only policy. Connector ALPN resolves a transport's
protocol only after connect, so concurrent misses may each own a transport until then. Compatible attempts
remain independent when they negotiate H1 and converge on the cell's one flight only when they negotiate H2.
An H1-required attempt that nevertheless negotiates H2 terminates with a protocol-mismatch error.

The logical owner carried through this decision is an *establishment authority*: the transport and, on a
bounded origin, its capacity lease. The authority has exactly one owner even though the request waiting for
its result does not own it; cancellation of that request cannot silently drop the transport or permit. After
ALPN, the owner performs one cell-local select-or-join transition before starting the Hyper protocol handshake:

```text
automatic attempt owns transport + optional capacity lease + launching waiter
  |
  +-- ALPN = H1
  |     `-- run H1 handshake
  |           +-- success -> install H1 record; record takes capacity lease
  |           |     +-- launching waiter accepts H1 -> dispatch
  |           |     `-- waiter requires H2 -> retain H1 for compatible demand;
  |           |                              return the existing unsupported-version error
  |           `-- error -> close transport; return lease; fail launching waiter
  |
  `-- ALPN = H2
        +-- H1 required -> close transport; return lease; fail launching waiter
        |
        `-- H2 compatible
              `-- atomically inspect this cell's accepting generation and H2 flight
                    +-- accepting generation -> register waiter against generation
                    |                           close losing transport; return its lease
                    +-- flight exists --------> register waiter as participant
                    |                           close losing transport; return its lease
                    `-- neither exists -------> register flight; owner task drives it
                                                owner task retains transport + lease
                                                  |
                                                  +-- H2 handshake succeeds
                                                  |     -> install record; record takes lease
                                                  |     -> make generation visible; serve participants
                                                  `-- error
                                                        -> close transport; return lease
                                                        -> fail participants
```

The inspection and either registration or flight installation are one transition under the cell's
coordination. This is the linearization point: many automatic attempts may reach H2 ALPN, but at most one
becomes the flight. A losing authority has not started a Hyper driver, so its guard closes the negotiated
transport and returns its capacity lease; its launching waiter remains represented exactly once, as a
generation user or flight participant. The owner task completes the handshake, transfers capacity to the
connection record, and installs the generation before submitting the driver. A request activated during that
submission interval is retained by Hyper's dispatch channel until the driver is polled. Activation
revalidates the generation identity and accepting state; if either changed after registration, the waiter
returns to acquisition rather than dispatching through stale state.

A waiter that joins a flight or generation retains its original waiter sequence. When the generation becomes
visible, the [generation gate](#http2-peer-routing) orders flight participants and already committed
compatible local waiters together; joining an accepting generation follows that same activation path.
Post-ALPN convergence
therefore cannot let a later attempt barge ahead of an older compatible waiter.

Request version changes only compatibility. When an automatic attempt launched by an HTTP/2-marked request
negotiates HTTP/1, the H1 connection remains useful and is handed to compatible local demand or the idle set.
The launching request receives the same unsupported-version classification and connection metadata as the
existing client; it neither dispatches on H1 nor loops establishing until ALPN happens to choose H2.

A flight record owns its identity and participant waiter identities. The
connection-partition owner task separately owns the transport and optional
capacity lease while it drives the handshake. Cancelling a participant removes
only that waiter; it does not cancel the owner task. If every participant
cancels, the task may still install an idle generation, which retains bounded
capacity until ordinary close or idle expiration. Task drop closes the
transport, returns capacity, and fails every participant still retained by the
exact flight. Completion from an old flight identity is stale and cannot clear
or replace a successor.

The ownership transfer is therefore fixed at each boundary:

| State               | Transport owner           | Capacity owner                              | Waiting-request owner              |
| ------------------- | ------------------------- | ------------------------------------------- | ---------------------------------- |
| independent attempt | establishment authority   | its optional capacity lease                 | cell waiter entry                  |
| H1 installed        | H1 connection record      | record's capacity lease                     | checked-out H1 guard or cell queue |
| H2 flight driver    | connection owner task     | owner task until record installation        | flight participant entry           |
| H2 joiner           | existing flight or record | existing owner; own capacity lease returned | participant or activation          |
| H2 installed        | H2 connection generation  | generation's capacity lease                 | H2 request claim after activation  |
| failure or drop     | cleanup guard             | guard until admission return                | terminal result or re-acquisition  |

This is the mechanism behind the miss policy from
[Local connection selection](#local-connection-selection): a partition that misses locally establishes its
own connection. It reaches for a peer's connection only when it cannot establish, which the next section
covers.

#### Obligations

* **Establishment placement** [safety] — the connector, transport setup, TLS, ALPN, and handshake for a
  connection are polled only on its placement runtime: the runtime bound by the anonymous partition's first
  establishment or the runtime named by an explicit partition.
* **Hyper task placement** [safety] — tasks Hyper spawns for a connection run on that connection's partition
  runtime.
* **Single permit owner** [safety] — at every point in establishment a bounded connection's permit has
  exactly one owner, and a failed or dropped establishment returns its permit to admission.
* **Driver termination closes the record** [safety] — normal driver completion and cancellation of the
  guarded driver task both request the record's exactly-once logical close without owning or retaining its
  capacity lease.
* **Connector readiness** [safety] — the connector is driven to ready before each call, and each readiness
  admits one call.
* **Single generation** [safety] — a cell keeps at most one accepting HTTP/2 generation; establishments that
  resolve HTTP/2 collapse to one, and the losing transports close and return their permits.
* **Atomic H2 convergence** [safety] — after automatic ALPN selects H2, one cell-local transition either
  activates an accepting generation, joins the one current flight, or installs the caller as that flight's
  driver before any H2 handshake or generation installation begins.
* **Losing-attempt cleanup** [safety] — an automatic H2 attempt that joins existing state closes its
  unhandshaken transport and returns its capacity lease while retaining its launching waiter exactly once.
* **Compatibility-preserving H1 result** [safety] — an H1 result is retained for compatible demand; an
  H2-required launching waiter receives the existing unsupported-version result with connection metadata and
  never dispatches on H1 or loops establishment for a different ALPN outcome.
* **Required-H1 negotiation** [safety] — an H1-required request never dispatches on H2. A connector that
  accepts per-attempt ALPN offers receives an HTTP/1-only offer for that request.
* **Flight cancellation** [safety] — participant cancellation removes only that participant; terminal flight
  drop closes its transport, returns its lease, and leaves no live waiter attached to the retired flight
  identity.
* **Transport failure classification** [safety] — a failure before the transport is established is an I/O
  error only when the network failed; a TLS refusal, a proxy's refusal, and a configuration error are
  terminal, as [Establishment failure classification](#establishment-failure-classification) records.

### Bounded-capacity coordination

A bounded origin at its limit cannot establish: admission has no permit to issue. The permit it needs may
still exist — held by a connection this waiter cannot use, or held in another partition's cell — but a
present permit is not a usable one. So two things have to happen. A waiter parks until a permit it can use
becomes available, and capacity that exists elsewhere moves to where the demand is. This section covers how a
cell signals demand, how the pool decides which capacity a waiter may take, the two operations that move
capacity, and how a freed resource is delivered to a parked waiter. All of it is bounded-mode only; an
unbounded origin never reaches here.

#### Demand

A cell's demand is one aggregate state, not a per-request count: a cell either wants another connection or it
does not. The state is active while any request waits and inactive when none do, so it is a single standing
ticket per cell rather than one ticket per waiting request.

The waiting requests queue behind the ticket in arrival order. An arriving permit goes to the request at the
head, which stops waiting; the next becomes the head. The ticket is how returning connections and other
cells find a cell with unmet demand; the queue is how that cell chooses whom to serve first.

The ticket carries one thing beyond its presence: the head request's protocol requirement. Admission must know
whether an HTTP/1 return, an HTTP/2 route, or either can satisfy the head before it reserves a resource.
This is the minimum the signal must distinguish; finer matching remains a dispatch-time question.

The logical snapshot published to admission is:

```rust
enum ProtocolRequirement {
    H1Required,
    H1Compatible,
    H2Required,
}

enum DemandState {
    Active {
        head: ProtocolRequirement,
        eligibility_group: EligibilityGroup,
    },
    Inactive,
}

struct DemandSnapshot {
    id: DemandId,
    version: SnapshotVersion,
    state: DemandState,
}
```

A `DemandId` names one generation for the cell's current queue head and may receive at most one terminal
acquisition outcome. Its protocol requirement is stable. Serving or cancelling that head retires the
generation; if useful demand remains, the cell creates a successor ID for the new head. `SnapshotVersion` orders
complete replacements for the same ID, so a delayed active publication cannot overwrite retirement. An
inactive snapshot retires the demand. When it must queue, each active demand joins the applicable scheduling
orders at their tails; checked identity allocation does not reuse a demand ID after wraparound.

#### Bounded miss

A bounded miss registers local demand before it asks admission to act. This ordering makes a waiter visible
before a permit, return, or route can target it. An immediately available permit still follows the same
delivery and acknowledgement path as a later one; there is no separate pre-registration fast path whose races
would need a second proof.

```text
local selection misses on cell C
  |
  +-- register waiter and current DemandSnapshot R under C's lock
  `-- publish complete snapshot R to OriginAdmission
        |
        +-- free permit
        |     `-- assign capacity to R
        |
        `-- no free permit
              `-- queue R in its origin order and applicable eligibility-group views
                    |
                    +-- compatible local H1/H2 appears
                    |     `-- satisfy locally; retire or replace R
                    |
                    +-- eligible peer H2 appears
                    |     `-- install generation route in C
                    |
                    +-- peer H1 is available
                    |     `-- borrow or reclaim for the oldest origin demand
                    |
                    `-- permit is released or reclaimed
                          `-- assign capacity to R

capacity delivery reaches C
  |
  +-- R is stale, cancelled, or already satisfied -> refunnel permit
  `-- R is live
        +-- final local probe now succeeds -> use local resource; refunnel permit
        `-- still needs a connection ------> move lease to establishing task

borrow delivery reaches C
  |
  +-- R or its reserved waiter is stale -> return H1 to connection-owning cell
  `-- still compatible -----------------> move checked-out H1 guard to waiter

terminal outcome for R
  +-- no useful demand remains -> ticket becomes idle
  `-- useful demand remains ---> publish successor generation at applicable tails
```

The final local probe is part of accepting capacity, under the cell lock. It prevents a permit delivered
concurrently with local return or H2 route installation from causing an unnecessary new connection. Local progress,
waiter cancellation, and host delivery therefore race through generation and snapshot-version validation rather than
recall a payload already extracted under another lock.

#### Eligibility and capacity

Two independent questions decide how a waiter is served, and they apply to different actions. *Eligibility*
asks whether this partition may dispatch through a particular connection, which
`ConnectionReuseScope` decides: under the default scope, partitions sharing a network
interface are eligible for each other's connections and partitions on different interfaces are not.
*Capacity* asks whether the origin may open another connection — one budget of `N` for the whole origin,
regardless of how many partitions or interfaces exist.

```rust
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum ConnectionReuseScope {
    Partition,
    #[default]
    NetworkInterface,
    Pool,
}
```

`Partition` permits no cross-cell dispatch. `NetworkInterface` groups partitions by the exact configured
interface, with all unbound partitions in one group. `Pool` permits reuse across every partition. Scope
controls only dispatch eligibility: it does not constrain reclaim, which closes a connection and transfers
capacity rather than moving I/O authority.

An **eligibility group** is the exact set of partitions whose reuse policies
allow them to share a connection. Each partition belongs to one such group for
an origin, derived from the configured scope above.

Dispatching through a connection that already exists consumes no capacity; that connection already holds one
of the admitted permits. So an eligible reusable connection serves the waiter whether or not a permit is
free — a pool at its limit still dispatches freely through all `N` of its connections. Capacity is consulted
only when no eligible connection can serve the waiter and the cell must open one. The two questions do not
read each other's state, and only that second case reaches admission: eligibility decides reuse, capacity
decides establishment. Keeping them separate is what lets the two operations below relieve a shortage of one
without disturbing the other.

#### Borrow and reclaim

A bounded miss may use another cell's HTTP/1 connection in two ways.

**Borrow** moves the exclusive Hyper request handle to the requesting cell for
one dispatch. The connection record, driver, socket, runtime, and interface stay
with the connection-owning cell. Borrow is therefore limited to cells in the
same reuse eligibility group.

**Reclaim** logically closes a reusable HTTP/1 connection and returns its
capacity lease to admission. The requesting cell can then establish on its own
partition. Reclaim moves no dispatch or I/O authority and is not limited by the
reuse scope.

`H1SupplyIndex` retains one origin-wide FIFO and one FIFO per eligibility
group over cells that own an HTTP/1 connection that is idle or may return.
A cell appears at most once in each applicable view. It is removed from both
views while an H1 match is nonterminal or while a usable local fairness turn
is owed, and is reinserted from its next complete supply revision.

Every `SupplyRevision<H1SupplyStatus>` carries a monotonic revision assigned
under the connection-owning cell lock. Admission ignores an equal or older
revision, so values crossing the unlocked cell-to-admission boundary cannot
hide newer state. The status is a policy input, not a connection handle or
capacity owner.

Borrow and reclaim share one retained H1 match lifecycle:

```rust
enum H1MatchKind {
    BorrowSender,
    ReclaimCapacity,
}

enum H1MatchState {
    ProbingIdle,
    Reserving,
    WaitingForSender,
    Resolving,
    Cancelling,
}

struct H1Match {
    supplier: PartitionId,
    requester: PartitionId,
    demand: DemandId,
    kind: H1MatchKind,
    state: H1MatchState,
    cancelled: bool,
}

enum H1ReservationState {
    Available,
    Installed(H1MatchId),
    Resolving(H1MatchId),
}

struct H1Reservation {
    state: H1ReservationState,
    local_turn_owed: bool,
}

struct H1CellState {
    records: HashMap<ConnectionId, H1Record>,
    idle_order: VecDeque<ConnectionId>,
    peer_reservation: H1Reservation,
}
```

Admission owns `H1Match`; the supplier cell owns `H1CellState` and its
`H1Reservation`. Each supplier and requesting cell participates in at most one
nonterminal match at a time. The match kind is fixed at selection. A reclaim
selected from the origin-wide order cannot become a borrow because its cells
may belong to different eligibility groups. A borrow could become a reclaim
without violating eligibility, but keeping both kinds fixed avoids adding a
second terminal path after reservation; a rejected match
returns to admission for a fresh selection.

A match reserves and resolves without nesting the admission, supplier-cell, or
requesting-cell locks:

```text
OriginAdmission owns queued demand R
  |
  `-- H1SupplyIndex selects peer supplier cell C
        +-- one eligible supplier -> retain H1Match K in Reserving
        |
        `-- several eligible suppliers -> retain K in ProbingIdle
              `-- check C for an immediately idle sender under C's lock

ProbingIdle(K)
  |
  +-- idle H1 available
  |     `-- C reservation -> Resolving(K); H1Candidate owns sender
  |
  +-- C is busy or unavailable and probe bound remains
  |     `-- select a distinct eligible supplier and repeat
  |
  `-- final supplier check
        `-- choose one exact supplier; K -> Reserving
              `-- reserve K under that supplier's lock

Reserving(K)
  |
  +-- supplier owes a usable local turn
  |     `-- reject K; R stays queued
  |
  +-- supplier has an older local H1 candidate
  |     `-- reject K; R stays queued
  |
  +-- idle H1 available
  |     `-- supplier reservation -> Resolving(K); H1Candidate owns sender
  |
  +-- active or returning H1 exists
  |     `-- supplier reservation -> Installed(K); reserve next reusable return
  |
  `-- no H1 can return
        `-- reject K; R stays queued

Installed(K) + reusable return at C
  `-- C reservation -> Resolving(K); H1Candidate owns sender

candidate reaches OriginAdmission
  |
  +-- K or R is stale, cancelled, or already satisfied
  |     `-- H1Candidate returns sender through C's ordinary return path
  |
  +-- K is BorrowSender
  |     `-- create DemandAssignment D for R
  |           `-- commit C's candidate before reserving requesting-cell waiter
  |                 +-- accepted -> waiter owns H1 selection
  |                 `-- refused -> return H1 to C, then retry D
  |
  `-- K is ReclaimCapacity
        `-- revalidate under C's lock and attempt logical close
              `-- released permit enters ordinary capacity delivery

terminal H1SupplyOutcome
  `-- remove K; apply C's latest supply revision; schedule next action
```

A provisional candidate revalidates the connection generation, logical-close
state, idle policy, and matching cell reservation before it becomes an
`H1Selection` or is reclaimed. A failed revalidation returns the request handle
through ordinary owning-cell policy. If another close wins the reclaim race,
the released permit still follows its normal exactly-once admission path.

An installed reservation intercepts a reusable return under the supplier-cell
lock before the handle can become locally idle. Local
H1-compatible demand that arrives after installation may therefore be
overtaken once. An irreversible borrow or successful reclaim records one local
fairness turn when compatible local demand exists. The next local H1 service
consumes that turn; if compatible demand disappears first, the turn clears.
An H2-required local head cannot consume the turn and does not block a reuse
match that can make progress.

Cancellation marks a reserving or resolving match stale. An installed
reservation crosses back to the supplier cell and is cleared. A
candidate already outside the cell lock returns through ordinary owning-cell
policy. Cancellation after irreversible transfer does not revoke an earned
fairness turn.

Every cross-lock action owns a typed fallback. Dropping an `H1IdleProbeAction`
returns the supplier to its index and advances or settles the match. Dropping
an `H1ReservationAction` or `H1CancellationAction` clears the cell reservation
and settles the admission match. Dropping an `H1Candidate` returns its sender
before the supplier cell becomes selectable again. Dropping a `DeliveryGuard`
returns the permit to admission. Fallbacks run no connector, protocol, wake, or
listener code while a pool lock is held.

A fallback invoked from `Drop` may synchronously acquire a bounded sequence of pool locks to publish its
terminal state, but it holds at most one pool lock at a time. Each lock transition produces the next typed
action only after releasing the previous lock, and each action retains an idempotent fallback until the next
domain owns the terminal state. The chain performs no await, retry loop, connector or protocol work, or work
proportional to waiter or partition count, and schedules wakes or callbacks only after releasing its lock.

Pool coordination uses a crate-level synchronization facade so production and Loom tests compile the same
lock-bearing code. Production lock wrappers retain access to guarded state after standard-library poisoning so
poisoning alone cannot prevent a later `Drop` fallback from returning a permit or settling a demand assignment.
This does not make an interrupted state transition valid: code under a pool lock still preserves its
invariants without relying on poison recovery. Test builds also assert that a thread holds at most one pool
lock, turning the no-nesting rule into an executable check across the ordinary suite.

Both stay within one origin, and neither moves a driver, so the I/O-placement guarantee from
[Connection placement follows declared topology](#connection-placement-follows-declared-topology)
holds under both. This is why `max_connections_per_host` below the partition count is valid rather than an
error: a partition with no permit of its own may dispatch through an eligible
peer HTTP/1 sender or receive capacity through reclaim. The
default `NetworkInterface` scope uses both; `Partition` and `Pool` are the same machinery with a narrower or
wider eligibility group.

#### Ordering across cells

Each cell orders its own requests. Across cells, admission keeps one
origin-wide demand FIFO and separate origin and eligibility-group views over
cells with HTTP/1 connections:

```text
OriginAdmission(O)
  demand order:
    oldest -> C2/R8(H1) -> C0/R3(H2) -> C3/R5(H2) -> C1/R9(H1)

  selectable H1 supplier cells:
    origin view:       C0 -> C2 -> C3
    group eth0 view:   C0 -> C3
    group eth1 view:   C2
```

The demand order contains the current head generation from every requesting
cell waiting for origin capacity. An H1 supply view contains a supplier cell at
most once while it has an H1 record that may return or be reclaimed, has no
nonterminal match, and owes no usable local turn or older local H1-compatible
waiter. Removing a cell repairs both views immediately, so admission does not
drain stale supply entries.

HTTP/1 selection begins with the oldest origin demand:

```text
oldest origin demand R from requesting cell Q
  |
  +-- R accepts H1 and an eligible peer cell C exists
  |     `-- cross at most MAX_H1_SUPPLIER_CHECKS distinct suppliers
  |           +-- idle sender found -> resolve BorrowSender match(C, Q, R)
  |           `-- final check -> reserve one exact eligible supplier
  |
  +-- another peer H1 cell C exists
  |     `-- retain ReclaimCapacity match(C, Q, R)
  |
  `-- no peer connection
        `-- wait for capacity, local service, or a later supply revision
```

The supply selector skips the requesting cell. Same-cell idle selection and
return are resolved under that cell's lock and do not create a cross-cell
match. Borrow crosses at most `MAX_H1_SUPPLIER_CHECKS` distinct eligible
supplier cells. With one supplier, admission reserves it directly. With
several suppliers, preliminary crossings take only an immediately idle sender;
the final crossing installs the ordinary exact reservation, which takes an
idle sender when present or intercepts that supplier's next reusable return.
If eligible supply disappears during the crossings, ordinary reclaim selection
resumes. The fixed-size probe set prevents a supplier that re-enters the FIFO
from being checked twice in one search. Reclaim takes the oldest origin-wide
peer.

The origin demand head is no younger than any eligibility-group demand head.
Selecting that origin head first therefore preserves eligible H1 ordering
without merging two demand heads. Eligibility changes only whether the
selected peer is borrowed or reclaimed.

The oldest origin demand therefore receives first use of every peer H1. If it
can borrow the selected connection, the warm connection remains open.
Otherwise reclaim closes it and returns capacity for the same oldest demand.
A younger demand in the connection's eligibility group cannot bypass the
older origin demand. One owning-cell fairness turn may follow an irreversible
transfer, bounding local overtaking without allowing peer traffic to consume
every return.

HTTP/2 peer routing uses eligibility-group demand because a route is
non-destructive and cannot satisfy an ineligible target. Those all-protocol
group views do not change HTTP/1's origin-head rule. Every cell choice remains
a stored-head operation, so the work to
grant one resource is independent of the number of cells and partitions.

#### Delivery

A released permit or provisional H1 can serve only one waiter. It must cross
from admission to a cell without being lost, copied, or left attached to a
cancelled demand generation. `DemandAssignment` names the exact requester and
demand while that resource crosses lock domains:

```rust
struct DemandAssignment {
    id: DemandAssignmentId,
    requester: PartitionId,
    demand: DemandId,
}

enum DemandScheduleState {
    Unscheduled,
    Queued {
        demand: DemandId,
        links: DemandLinks,
    },
    PendingAssignment {
        assignment: DemandAssignment,
        version: SnapshotVersion,
        selected_order: DemandOrder,
        links: DemandLinks,
    },
}

enum DeliveryPayload {
    Capacity(CapacityPermit),
    BorrowedH1 {
        match_id: H1MatchId,
        supplier: PartitionId,
        candidate: H1Candidate,
    },
}

enum DeliveryState {
    Pending(DeliveryPayload),
    Ready {
        step: AcquisitionStep,
        settlement: DeliverySettlementKind,
    },
    Disarmed,
}

struct DeliveryGuard {
    admission: Arc<OriginAdmission>,
    assignment: DemandAssignment,
    state: DeliveryState,
}

struct DeliverySettlement {
    admission: Arc<OriginAdmission>,
    assignment: DemandAssignment,
    successor: Option<DemandSnapshot>,
    kind: Option<DeliverySettlementKind>,
}

enum DeliverySettlementKind {
    Capacity,
    BorrowedH1 {
        connection_id: ConnectionId,
        match_id: H1MatchId,
        supplier: PartitionId,
    },
}

enum DemandAssignmentOutcome {
    Accepted { successor: Option<DemandSnapshot> },
    RetrySamePosition,
    Refused { successor: Option<DemandSnapshot> },
}
```

`Queued` and `PendingAssignment` retain the same origin and eligibility-group
links. A pending assignment excludes another resource from selecting that
demand without moving it to the back of either order. An H1 match does not
assign demand while supplier reservation crosses locks. Only a resolved borrow,
or an available permit, creates an assignment.

One `DeliveryGuard` carries either capacity or a borrowed H1. It materializes
every fallible connection-owning-cell transition before reserving the
requesting waiter. Capacity becomes an `EstablishmentPermit`; a borrowed
candidate revalidates its owning-cell reservation and becomes an
`H1Selection`. If candidate commit fails, the guard returns the handle and
retries the assignment without changing requesting-cell state.

An owned one-to-one delivery follows this sequence:

```text
OriginAdmission lock
  Queued(R)
    -> PendingAssignment(D)
    -> extract DeliveryGuard::Pending(payload)
unlock OriginAdmission
  |
  +-- resolve payload
  |     +-- failure -> refunnel payload; retry D; requesting cell unchanged
  |     `-- success -> DeliveryGuard::Ready
  |
  `-- lock requesting cell
        +-- R and its oldest compatible waiter are live -> reserve waiter
        `-- stale, cancelled, satisfied, or incompatible -> reject guard
      unlock requesting cell
        `-- convert payload into AcquisitionStep + DeliverySettlement
              `-- lock requesting cell
                    +-- accepted -> waiter owns step; settle D as accepted
                    `-- cancelled -> return step; settle D as refused
```

The admission, connection-owning-cell, and requesting-cell locks are never
nested. Between them, the delivery guard is the only payload owner. After
requesting-cell installation, `DeliverySettlement` owns assignment settlement
and any supplier-cell completion still owed; the requesting waiter owns the
establishment permit or H1 selection.

The guard makes every drop point terminal. Dropping `Pending` returns its raw
payload before retrying the assignment. Dropping `Ready` drops the
establishment permit or returns the selected H1 to its owning cell, then retries
the assignment. Once the requesting cell owns the step, dropping
`DeliverySettlement` records acceptance because cell state is authoritative.
Explicit refusal returns the step before settling the assignment as refused.

`Accepted` consumes the generation and either unschedules it or installs its
successor at the applicable tails. `RetrySamePosition` preserves the same
generation and both order positions. `Refused` ends the old generation after
the requesting cell has refunnelled any owned payload and carries the complete
current successor, if one. A complete newer demand snapshot may retire or
replace an assignment before its action reaches the requesting cell; local
generation validation then rejects the late action without resurrecting old
demand.

#### HTTP/2 peer routing

An H2 route carries no sender or capacity payload. The connection-owning generation continues to own the capacity
lease. The route names a connection-owning cell and generation from which compatible requests may take
H2 request claims. Installing a new local generation first installs the accepting generation under
the connection-owning cell lock, then makes that identity visible to compatible local waiters. They are woken
and admitted in bounded local turns; route installation does not scan or synchronously wake an unbounded queue.

Generation visibility also installs a local fairness gate:

```rust
enum H2ActivationGate {
    Closed,
    Prioritizing {
        generation: H2GenerationId,
        cutoff: WaiterId,
        active_turn: Option<WaiterId>,
    },
    Open { generation: H2GenerationId },
}
```

`Closed` names no usable generation. The cutoff is the newest compatible
waiter committed when a generation becomes visible. While the gate is
`Prioritizing`, those waiters receive activation oldest first, and
`active_turn` retains the one priority turn crossing to Hyper acceptance or
cancellation. When no waiter at or before the cutoff remains, the gate becomes
`Open`. Open generations issue concurrent prospective activations; Hyper owns
stream credit and flow control. Generation invalidation closes the gate and
returns unserved transferred waiters to acquisition. Each successful
activation creates its own H2 request claim.

A requesting cell retains peer-route crossing state separately:

```rust
struct PeerH2Route {
    route: H2Route,
    activation_gate: H2ActivationGate,
    crossing_waiter: Option<WaiterId>,
}
```

Queued peer-route activations cross the connection-cell lock one at a time so a failed exact-generation
reservation can restore the requesting cell's oldest turn. Once the route gate is open, direct arrivals
reserve independent prospective claims concurrently; `crossing_waiter` does not serialize them.

`H2SupplyIndex` holds one group-scoped record for each cell whose current supply status names an accepting
generation. Generation installation precedes the supply revision that makes it visible; transition out of
accepting removes it from route selection. Either a changed supply revision or new group demand schedules a
bounded route turn from stored supplier and demand heads. The supply record carries only connection-owning
cell and generation identity, not a dispatch handle or capacity lease. The route guard revalidates it at the
connection-owning and requesting cells. A stale record is removed or updated before the next turn, so peer H2
discovery does not scan cells.

Under bounded pressure, an accepting generation may also be offered to the
head of its eligibility-group all-protocol view. Admission creates a
`DemandAssignment` for that exact generation; the route action carries only the
assignment, connection-owning cell, and generation identities plus a settlement
fallback, never the record's capacity lease.
The requesting cell revalidates the generation identity, accepting state, demand
generation, and reuse scope.
Acceptance makes the generation visible to compatible waiters there; rejection discards the stale notice
while the generation and
permit remain at the connection-owning cell. Acceptance acknowledges after requesting-cell visibility and the
named head's activation opportunity are committed, not after every local waiter has activated. Remaining local
waiters proceed through the generation gate in bounded turns and no longer advertise a connection need while
that generation remains usable. Later group tickets are handled by subsequent bounded route turns, so
one-to-many visibility does not turn one host action into work proportional to partition count.

An H1-required head cannot use an HTTP/2 supply record. If bounded origin capacity is exhausted,
admission may instead reserve the oldest indexed idle generation for reclaim. The connection cell
revalidates the exact generation, accepting residence, and zero prospective or active request counts before
closing it. Logical close returns its capacity to origin admission, which can then grant establishment to the
H1-required head. A generation with request work is not reclaimed. Its transition to idle submits a new
supply revision and makes reclaim eligible.

Admission attempts this reclaim only when the transport factory guarantees that
an H1-required establishment uses HTTP/1. Without that guarantee, admission
could close a healthy H2 connection, negotiate H2 again, reject the result, and
repeat without satisfying the waiting request. The default rustls and cleartext
connectors provide the guarantee. The fixed-ALPN s2n connector and connectors
injected through the unstable test utility do not, so H1-required demand
remains queued behind their healthy H2 capacity until an ordinary close
releases a permit.

For bounded origins, a cell submits H2 supply only when generation identity, idle state, or availability
changes. Intermediate multiplexed request counts remain cell-local. Unbounded origins have no admission route
selection.

Dropping a pending route guard submits its fallback so the assignment retries or settles;
there is no single-owner payload to refunnel. Committing route installation stores the requesting-cell
acknowledgement, which is submitted before the guard disarms.

This separates route installation from single delivery: a permit or H1 has one owner and one requesting cell, while
an H2 generation remains owned by its connection cell and may be announced repeatedly. The transitions must
be model checked; their invariants are stated in [Correctness invariants](#correctness-invariants), with the
checks specified in [Appendix B](#appendix-b-validation).

#### Obligations

* **Bounded demand** [safety] — a cell carries at most one active demand generation regardless of how many
  requests wait, so demand accumulates no deficit and one residence receives at most one terminal outcome.
* **Snapshot ordering** [safety] — admission retains the newest complete demand version and rejects an action
  for a retired generation, so out-of-order publication cannot resurrect cancelled or satisfied demand.
* **Eligibility and capacity independence** [safety] — the capacity decision and the eligibility decision do
  not read each other's state.
* **Placement under transfer** [safety] — neither borrow nor reclaim moves a connection's driver or I/O off
  its owning partition.
* **Reclaim scope independence** [safety] — reclaim moves a permit without dispatching across a partition
  boundary, so it is not constrained by the reuse scope.
* **Protocol-compatible reclaim** [safety] — admission reclaims idle H2 capacity for H1-required demand only
  when the transport factory guarantees an HTTP/1 establishment result.
* **H2 supply revision** [safety, optimization] — bounded cells submit a new supply revision only when
  generation identity, idle state, or availability changes; intermediate stream counts require no admission
  update.
* **Single delivery** [safety] — one demand assignment carries at most one permit or provisional H1, commits it
  to at most one requesting waiter, and remains pending until requesting-cell settlement.
* **Refunnelling** [safety] — rejection, supersession, cancellation, task drop, or panic returns every
  undelivered permit to admission and every undelivered H1 to its connection-owning cell exactly once.
* **Route ownership** [safety] — an H2 route carries generation identity, never the connection's
  capacity lease; request activation takes a request claim while the connection generation remains the capacity owner.
* **Generation visibility priority** [liveness] — making a local generation or peer route visible closes the
  generation gate and serves previously committed waiters in bounded oldest-first turns before later direct
  arrivals.
* **Cross-cell order** [safety] — H1 borrow and reclaim both serve the current origin head, preferring an
  eligible peer connection for borrow and otherwise reclaiming an origin peer. Peer H2 routing uses its
  all-protocol eligibility-group head. Same-cell H1 service remains cell-local.
* **H1 match completion** [safety] — one H1 match reserves at most one supplier cell and one requesting cell; it
  remains authoritative until supplier-cell completion and any borrowed demand assignment settles its terminal
  state.
* **Bounded H1 probing** [optimization] — one H1 borrow match crosses at most
  `MAX_H1_SUPPLIER_CHECKS` distinct eligible supplier cells. Preliminary crossings take only an immediately
  idle sender; the final crossing installs one exact supplier reservation. The work does not grow with the
  number of partitions.
* **Return interception** [liveness] — an installed H1 reservation intercepts the next reusable H1 before it
  becomes idle, so a connection cycling continuously between active and reusable cannot strand requesting
  demand.
* **Owning-cell fairness turn** [liveness] — one irreversible cross-cell transfer creates one local turn when
  compatible local demand exists; the turn clears only when that demand is served or disappears.
* **Acknowledged progress** [liveness] — every extracted delivery or detached action either settles a terminal
  transition or executes its typed fallback, so a demand assignment cannot remain pending solely because the
  executing future was dropped.
* **Cross-lock isolation** [safety] — admission and cell locks are never nested, and no pool lock is held
  across an await or while running connector, protocol, wake, or listener code. A synchronous fallback may
  visit a bounded sequence of lock domains but holds at most one pool lock at a time; each transition retains
  an idempotent fallback, and wakes and callbacks remain deferred until after unlock.
* **Bounded peer discovery** [optimization] — reuse and route work select connection state from stored
  origin or group heads and validate one cell rather than scanning cells or connections. H1 supply is linked
  once and repaired eagerly; a cell submits again only when its complete returnable or peer-blocked status
  changes, and admission ignores older supply revisions.

### Liveness

A cell's queue orders its requests, and same-cell H1 service is resolved under
that cell's lock. Across cells, origin admission orders capacity demand. H1
reuse serves the current origin head: it borrows the oldest eligible peer
connection when available and otherwise reclaims the oldest origin peer.

HTTP/2 peer routing adds eligibility-group demand views because a route can be installed only where the
generation is reusable, and routing does not consume origin capacity. A terminal outcome sends any successor
generation to
the applicable tails. An owning-cell fairness turn permits one local overtake after an irreversible
cross-cell transfer, but repeated H1 matches cannot keep that cell or an older peer from progressing.

Within a cell, the generation gate offers a newly visible H2 generation to already committed compatible
waiters in bounded oldest-first turns before later direct arrivals. Scheduling is work-conserving among
eligible waiters: if a resource a waiter could use is free, some eligible waiter is served rather than the
resource sitting idle. Each choice is a dequeue or stored-head comparison, so the work to grant one resource
does not grow with the number of waiters or partitions.

Progress requires that a permit become reachable: an eligible connection returns reusable, an
HTTP/1 connection becomes reclaimable, or a permit is released. It is not promised while every permit for the
origin is held indefinitely by active HTTP/2 work that the waiter is not eligible to use. The pool does not
forcibly drain a live HTTP/2 connection to free such a permit; doing so would abort in-flight requests to
serve a waiter. A waiter in this state parks until eligibility or capacity changes.

#### Obligations

* **Bounded overtaking** [liveness] — a committed cell is not passed indefinitely by later arrivals; permits
  and H1 reuse use the origin-wide order, with eligible borrow preferred over reclaim for that head; peer H2
  routing uses the all-protocol group view; one owning-cell fairness turn may create only the documented
  bounded overtake.
* **Work-conserving service** [liveness] — while an eligible waiter and a resource it may use both exist,
  some eligible waiter is served.
* **Bounded grant work** [optimization] — the work to grant one resource does not grow with the waiter or
  partition count.

### Dispatching and completing a request

Acquisition ends with one request and one selected dispatch authority. Dispatch turns those into either a
terminal error or a response whose body, or upgrade path, owns the request's remaining protocol lifecycle.
This section defines that ownership transfer. HTTP framing, stream state, and flow-control behavior remain
Hyper's responsibility.

#### Preparing the request

The request future retains the original absolute URI while the request is in the pool. The origin key and
proxy decision are made from that URI before any request-target rewrite. Existing request validation and proxy
authentication behavior is preserved, including rejection of unsupported HTTP versions and HTTP/1.0
`CONNECT`, and insertion of proxy authorization only for an applicable cleartext HTTP proxy when the caller
did not supply it.

Protocol compatibility is checked against the selected connection before the request is moved into Hyper. An
HTTP/2-marked request cannot use H1; an HTTP/1.1-marked request may use H2. An incompatible H1 selection is
returned to its connection-owning cell if it remains usable, and the request receives the existing
unsupported-version error.
This applies to a fresh automatic-ALPN attempt that resolves to H1: the pool keeps the H1 for compatible
demand and does not establish repeatedly in hope of negotiating H2. The compatibility error and connection
capture both identify the H1 connection that was selected.

For H1, dispatch preserves Hyper's existing wire form. The pool inserts `Host` when configured and absent,
using the non-default port when one exists. `CONNECT` uses authority form, a request sent to a cleartext HTTP
proxy uses absolute form, and a direct or tunneled request uses origin form. H2 receives the form Hyper expects
for its codec. The retained absolute URI, not the temporary wire form, is the authority for retry, diagnostics,
and error return.

Before the Hyper call, the request's `CaptureSmithyConnection` backchannel is bound to metadata for the
selected physical connection: proxy state, local and remote addresses, and a poison callback naming that exact
connection generation. A stale-reuse retry replaces the binding with the replacement connection. The
callback is idempotent, becomes a no-op after its generation is gone, and does not keep a retired connection
or the pool alive. Connection metadata attached by the connector is also copied to the response before the
response head is exposed.

#### Readiness and Hyper acceptance

HTTP/1 readiness is a condition for entering reusable storage, not another state in request dispatch. The
request handle produced by a successful Hyper handshake may send the connection's first request. A returning
handle is not made idle or handed to a waiter until Hyper reports it ready for another request. Selection
therefore yields exclusive ownership of the handle that may call `try_send_request` directly.

Immediately before that call, the connection record commits dispatch against logical close. Dispatch commit
and logical close are mutually exclusive linearization points. If close wins, the request remains locally
owned, the stale selection retires, and acquisition runs again. If commit wins, `try_send_request` is invoked
on the same sender without publishing an intermediate state or holding a pool lock. Hyper polling,
request-body polling, callbacks, wakes, and destructive drops all occur outside pool locks.

```text
Acquired H1 sender (fresh or previously proven ready)
  -> Prepared
  -> DispatchCommit races logical close
       +-- close wins  -> retire selection; request remains unsent; reacquire
       `-- commit wins -> try_send_request on the same sender
             +-- Hyper returns original request -> UnsentReturned
             +-- Hyper accepts request ---------> WaitingForHeaders
                                                    +-- error -> TerminalError
                                                    `-- head  -> BodyGuardTransferred
```

`try_send_request` is still allowed to reject the envelope. That result is different from a pool-side stale
selection: only Hyper can certify that the original request remains unsent, and only a reused connection turns
that certification into transparent retry. A fresh connection returning the request is a terminal error.

For HTTP/2, activation of a request claim includes the generation checks needed before
calling its sender. The same general boundary holds: pool state commits one dispatch before Hyper accepts the
envelope, and Hyper's returned-message behavior remains the only replay authority.

Once `try_send_request` accepts the request envelope, Hyper owns the request and its body. That point
discharges any H2 generation-gate opportunity; selecting or cloning a sender is not enough. The request future
continues to own the Hyper response future, the H1 exchange or H2 response guard, and a strong pool
reference while it waits for response headers; an accepted H2 request-body adapter owns the matching upload
guard.

An accepted H2 request has two independently owned sides. A request-body adapter owns the upload guard while
Hyper may still poll an upload; the request future owns the response guard until it transfers that guard to the
response body or upgrade bridge. The request claim returns to the generation only after both sides finish. A
response arriving before a streaming upload finishes therefore cannot make the request idle. Before acceptance,
the request-body guard is inert. If Hyper returns the request unsent, the pool disarms that guard and can rearm
the same adapter for a later selection without retaining the rejected generation.

#### Retry, timeout, and errors

There is one authority for transparent dispatch retry: Hyper must return the original, unsent request from
`try_send_request`, and the selected connection must have been reused. The pool restores the absolute URI,
disarms any unaccepted H2 upload guard, retires or invalidates the stale selected connection, and sends that
same request through acquisition again. An unsent failure from a fresh connection is terminal and still
retires a sender Hyper reported unable to accept the request; its inert upload guard and selected authority cannot
survive the error. The pool does not clone a request or replay one Hyper accepted.

`AcquisitionOutcome::RetryAcquisition` is the internal transition for an unserved waiter whose flight or generation
closes before it receives dispatch authority. It carries no request copy and is not an SDK retry attempt: the
original request remains owned by the pool future and re-enters acquisition. A request returned unsent by Hyper uses the
same loop only under the reused-connection rule above. Negotiated-protocol mismatch and a fresh Hyper error
without the original envelope are terminal.

After dispatch receives an H2 activation, the request may select at most two replacement H2 generations.
A third pre-acceptance rejection returns its retained connection error. This bound applies only after an H2
target reaches dispatch. It is independent of caller timeout and cancellation and prevents stale selected
targets from retaining one request in the dispatch loop indefinitely.

With Hyper 1.11, the returned-request path after `try_send_request` is reachable for H1; H2 dispatch errors
after call do not carry the original request and are terminal. An H2 readiness failure observed before call is
different: the request is still locally owned and may return to acquisition without being replayed.

Every error after Hyper accepts the envelope is terminal for this dispatch. H1 conservatively retires because
the request may have reached the wire. H2 releases or resets the affected stream and finishes its upload and
response guards; a stream-local reset does not by itself invalidate the accepting
generation. GOAWAY, a closed dispatcher, a connection error, or other connection-fatal evidence does
invalidate that generation. The resulting `ConnectorError` preserves the current source chain and
classification, including timeout, user, I/O, incomplete-message transient,
GOAWAY, and `REFUSED_STREAM` behavior.

The read timeout keeps its current scope: `PoolConnector::call` starts it around the client request operation,
including acquisition, establishment when needed, dispatch, and the wait for response headers, and ends it
when headers arrive. It does not time body reads or an upgraded stream. A connect timeout covers only the
connector operation for one establishment attempt; a participant waiting on another request's HTTP/2 flight
has no connector operation to time. When either timeout wins, dropping the operation follows the cancellation
rule for its current dispatch stage; timeout classification does not bypass lifecycle cleanup.

#### Response and cancellation ownership

Before returning response headers, the request future installs the response lifecycle guard and response
metadata. That transfer is atomic from the caller's perspective: after a successful call, the returned body
or upgrade path owns cleanup; before it, the request future does. A panic or cancellation cannot land between
the two with no owner.

| Stage               | Request                   | Dispatch handle                               | Connection / stream guard                                              | Response body            | Retry authority        |
| ------------------- | ------------------------- | --------------------------------------------- | ---------------------------------------------------------------------- | ------------------------ | ---------------------- |
| Acquiring           | Request future            | None                                          | Waiter or delivery fallback                                            | None                     | None                   |
| Prepared / selected | Request future            | Selected sender                               | H1 selection or prospective H2 activation                              | None                     | None                   |
| Unsent returned     | Request future regains it | Stale sender retires                          | Selection resolves; H2 upload guard is inert                           | None                     | Reused connection only |
| Accepted / headers  | Hyper                     | H1 request future; H2 local sender clone      | H1 request future; H2 upload and response guards                       | None                     | None                   |
| Headers delivered   | Consumed                  | H1 exchange; no H2 local sender clone         | Body or upgrade owns H1 exchange or H2 response; request adapter may own H2 upload | Caller owns guarded body | None                   |
| Terminal            | None                      | H1 owning cell or retired; H2 generation      | H1 returned or closing; H2 claim released                              | Completed or dropped     | None                   |

The open connection record owns its capacity lease throughout this table. Dispatch never moves the permit into
the request, sender, body, or request claim; only logical close returns it to admission.

Dropping during acquisition uses the waiter, delivery, and refunnelling rules already defined. Dropping after
selection but before call returns a still-usable H1 through its connection-owning cell's ordinary return path
or cancels the prospective H2 request claim. Dropping after Hyper accepts but before headers closes H1 through
Hyper's supported cancellation path; on H2 it resets only the stream with `CANCEL`, finishes the response guard,
and lets the request-body adapter finish the upload guard before releasing the request claim. Dropping
after headers follows the body rules in [Returning a connection](#returning-a-connection).

Poisoning is monotonic and generation-specific. Invoking captured metadata's poison callback immediately
removes the named H1 record or H2 generation from new dispatch, but does not abort an accepted request merely
to accelerate replacement. An active H1 begins logical close immediately and tears down after its exchange
reaches a terminal boundary. H2 stops accepting new claims while existing streams drain. A concurrent driver
error, GOAWAY, idle timeout, reclaim, or repeated poison races through the same exactly-once logical-close
transition.

#### Upgrades

An upgrade changes which object owns protocol completion. H1 drivers run with upgrade support. A `101`
response or successful HTTP/1 `CONNECT` logically closes the checked-out record before the response is
exposed: its sender cannot return to the pool. The connection retains bounded capacity while the transferred
root I/O remains client-owned. There is no separate reusable upgrade-pending pool residence.

Hyper's upgrade-capable driver owns the subsequent transport transfer. It moves the wrapped transport and any
bytes read past the HTTP message into the `Upgraded` object. The caller then owns that I/O; dropping it completes
the client's physical connection ownership through the transport wrapper. If `OnUpgrade` is dropped or the
transfer fails, Hyper closes the transport instead.

Hyper may complete its HTTP/1 driver in the same poll that delivers the upgrading response head. The driver
guard can therefore record `ProtocolClosed` before the request task observes the response. While one committed
exchange can still prove an upgrade, the connection retains its permit until that exchange supplies the final
classification. A confirmed `101` or successful `CONNECT` refines the close reason to `Upgraded` and retains the
permit until physical completion. A non-upgrade result returns the permit when classification completes. In
either poll order the H1 record was already logically closed and can never return as an HTTP connection.

For H2 extended `CONNECT`, the physical H2 connection remains pooled but that stream is no longer represented
by an ordinary response body. An upgrade lifecycle bridge takes the response guard and retains it
with the upgraded stream until both upgraded directions are terminal. The owner-partition executor may attach
the bridge to Hyper's `UpgradedSendStreamTask`, but task completion alone is sufficient
only when it proves the receive half is also done; otherwise the Hyper integration needs a narrow full-stream
completion hook. The original request-body upload guard must also finish before the claim releases.
Releasing the claim resets or completes that stream only. Other streams and future requests
may continue on the same accepting generation. Ordinary response-body completion must not release the
transferred claim early.

#### Obligations

* **Same-instance dispatch** [safety] — one selected H1 sender commits against logical close and calls
  `try_send_request` directly, without publishing or transferring an intermediate dispatch state.
* **Wire-form restoration** [safety] — temporary request-target rewriting never replaces the retained absolute
  URI used for retry, diagnostics, or errors.
* **Certified retry** [safety] — transparent retry uses only the original request Hyper returned unsent from a
  reused connection; an accepted request and a fresh-connection failure are never replayed.
* **Continuous response ownership** [safety] — the request future owns cleanup through response headers or
  terminal error, then transfers it to the response body or upgrade bridge before exposing the response.
* **Stage-local cancellation** [safety] — cancellation returns or retires H1 according to whether Hyper
  accepted it, and releases or resets only the selected H2 request claim.
* **Compatibility surface** [safety] — request validation, target form, proxy authentication, metadata,
  timeout scope, source chain, and error classification preserve the current client behavior, except
  transport-stage error classification, which follows
  [Establishment failure classification](#establishment-failure-classification).
* **Upgrade transfer** [safety] — an H1 upgrade transfers root I/O and cannot return to the HTTP pool; an H2
  extended `CONNECT` transfers its response guard to an upgrade bridge until both stream directions terminate.
* **Two-sided request claim** [safety] — an accepted H2 request releases its claim only after both its
  upload and response guards finish.
* **Poisoned generation** [safety] — poisoning prevents every later dispatch on the named record or generation
  without aborting already accepted work solely for replacement.

### Connection retirement and maintenance

After an accepted request reaches a terminal protocol boundary, its connection either returns to service or
leaves the pool. This section defines that decision and the two-step close that retires a connection.

#### Returning a connection

Response headers do not make a connection reusable. The guarded body owns H1 lifecycle or the H2 response
guard until the response reaches end-of-stream, fails, or is dropped.

For H1, end-of-stream begins return processing; the checked-out sender returns to its owning cell only after Hyper
also reports it ready for another request. Dropping an incomplete body is not itself evidence of reusability.
Hyper may synchronously consume an already-buffered remainder and prove the message boundary; if it does, the
same ready check may return the connection. If the remainder is unavailable, cancellation, body error, or
protocol state cannot prove the boundary, the connection logically closes. The pool does not parse or drain
HTTP independently of Hyper.

The response path polls H1 readiness once. If readiness is pending after the response reaches a reusable
protocol boundary, it transfers an `H1Exchange` into a readiness task spawned through the connection's
owner-partition `DriverSpawner`. The response body does not retain responsibility for polling
that sender, and `Drop` never waits. The task enters owning-cell return only after Hyper proves both the message
boundary and readiness for another request. Closed, poisoned, upgraded, or owner-runtime-shutdown outcomes
logically close the record; dropping the task owns the same connection-close fallback.

For H2, body end-of-stream or a stream-local error finishes the response guard. Dropping an incomplete
body does the same and asks Hyper to send `RST_STREAM(CANCEL)`; the request claim releases after the
upload guard also finishes. Neither outcome retires an otherwise healthy generation. GOAWAY,
connection failure, or explicit poisoning may independently have moved the generation to draining, in which
case the last request claim completes drain instead of returning it to accepting
state. An H2 extended `CONNECT` follows its upgrade lifecycle bridge rather than the ordinary body terminal.

Every H1 return revalidates the record's generation, poison state, and idle
policy under the connection-owning cell lock. An unbounded origin serves
compatible local demand or installs the connection as idle directly because it
has no admission state or cross-cell reuse. For a bounded origin, the same
transition also checks its installed peer reservation. An installed
reservation extracts the sender into a provisional candidate for borrow or
reclaim. Without a reservation, the owning cell first serves compatible local
demand and otherwise installs the sender as idle. This complete decision is
cell-local; a returning sender does not synchronously consult admission.

After the cell transition, a bounded connection-owning cell submits an H1 supply
revision only when `has_returnable_connection` or `peer_use_blocked` changes.
Demand-driven admission may then retain a future H1 match, but it cannot
interpose between the just-completed local return decision and its sender
ownership. `ReservedForPeer` is counted as active rather than idle because no
request may select it. Every transition revalidates retirement
state, so a body that finishes concurrently with poison, reclaim, driver failure, or pool shutdown cannot
make a connection selectable after retirement.

#### Logical close and physical connection ownership

A connection that leaves the pool has two independently tracked transitions. At **logical close** it stops
accepting new work. An ordinary draining connection returns its permit at that transition, so a replacement can
be admitted while the old transport is still draining or tearing down. An H1 connection with one committed
exchange that may have transferred upgraded I/O retains its permit until the exchange classifies the outcome.
A non-upgrade result then returns the permit; a confirmed upgrade retains it until physical completion.

At **physical connection completion** the client releases its transport handle, whether because a driver
dropped the root I/O or an upgraded protocol finished with it. This does not assert that the peer, kernel, or
TCP teardown has completed.

Returning capacity when an ordinary connection stops accepting work keeps a slow teardown from stalling the
pool. A connection's socket does not close instantly: TLS sends `close_notify`, TCP exchanges FIN, and the OS
may linger the socket after that. Holding every permit until the socket was gone would block a waiter for a
teardown the pool does not control. The H1 upgrade exception prevents a different failure: repeated upgrades
cannot bypass `max_connections_per_host` while caller-owned transports remain open.

The complete connection lifecycle is:

```text
establishing (attempt or flight owns capacity lease)
  |
  +-- failure/cancel -------------------------------> permit refunnelled; no connection
  |
  `-- handshake -> record owns capacity lease; guarded driver task armed
        |
        +-- H1 open/idle <---- successful return ---- H1 checked out
        |       |                                      |
        |       |                                      +-- response body complete
        |       |                                      |     `-- Hyper ready -> return
        |       |                                      +-- incomplete body dropped/error
        |       |                                      |     +-- Hyper proves boundary -> return
        |       |                                      |     `-- otherwise -> logical close
        |       |                                      `-- upgrade -> logical close + transfer I/O
        |       |
        |       `-- idle/reclaim/poison/driver close --------> logical close
        |
        `-- H2 accepting generation
                |
                +-- accept request claim -> upload guard + response guard
                |                           `-- both finish -> release request claim
                |
                `-- GOAWAY/poison/driver close -------------> logical close

logical close (once: remove reuse eligibility; normally release capacity)
  |
  +-- no accepted work ------------------------------> transport teardown
  +-- H1 accepted exchange --------------------------> classify; release if not upgraded
  +-- H2 request claims remain ----------------------> drain to zero, then teardown
  `-- H1 upgraded I/O transferred ------------------> retain capacity; caller owns transport
                                                        |
transport root is dropped <-----------------------------+
  `-- physical connection ownership ends
```

The consequence is that live sockets can outnumber admitted connections: a
replacement admitted at logical close can coexist with a victim whose transport
or kernel socket has not ended. `max_connections_per_host` bounds admitted
connections; **no finite general bound on live sockets follows from it.** How
long a socket lingers after logical close depends on peer and OS behavior, and a
busy origin can keep admitting replacements while earlier sockets are still
draining, so the count of live sockets for one origin has no bound expressible
in `N` alone. A caller who needs a file-descriptor ceiling sets it at the OS,
not through this option. Accepted H2 streams that outlive their connection's
logical close are draining, not admitted: they hold no permit and accept no new
requests, and they too close on their own schedule.

#### Why a connection retires

The reason recorded by the logical-close transition is part of the observation surface:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CloseReason {
    IdleTimeout,
    Poisoned,
    ProtocolClosed,
    IncompleteH1Exchange,
    Upgraded,
    Reclaimed,
    PoolDropped,
    OwnerRuntimeShutdown,
}
```

A connection retires for one of a few reasons:

* **Idle timeout** — an H1 deadline starts when its sender becomes reusable in the idle set and is absent while
  the record is selected, active, or resolving return. A new H1 therefore begins aging only after its first
  idle installation. An H2 deadline starts when a generation becomes accepting, including a fresh generation
  that has not dispatched, and resets whenever a request claim commits to dispatch. Active streams do not
  suspend that deadline. H2 expiration moves the generation out of accepting state and begins logical close;
  accepted claims continue draining and retain the physical transport. A connection with idle timeout disabled
  is kept.

  Closing an otherwise-quiescent connection cannot wait for the next request, so each partition runs a
  maintenance task on its own runtime. Every task uses the pool's builder-injected `TimeSource` and
  `AsyncSleep`, so tests drive idle age with a fake clock. Because that time source is `SystemTime`, which can
  step backward or forward, idle age is measured against the scheduled sleep deadline the task already holds,
  not by subtracting two `SystemTime` readings. The deadline gives a monotonic floor: a clock jump changes when
  the task wakes but not whether a connection idle since a completed deadline has expired.

  The scheduler records the deadline represented by its current sleep. A newly idle connection wakes it only
  for an earlier deadline; ordinary checkout does not force a partition scan. An atomic start gate submits at
  most one task per partition. The task retains only weak cell registrations between scans, drops each strong
  scan snapshot before waiting, and exits on explicit partition shutdown even when no cell or deadline remains.
  At the start of a scan, it atomically retires the deadline that triggered the scan while capturing the
  scheduler revision. A connection returned to idle during the unlocked scan therefore advances the revision
  and forces a retry rather than being hidden behind an already elapsed deadline. Shutdown and earlier-deadline
  publication detach the waker under the scheduler lock and wake it after unlock. If the submitted maintenance
  future is dropped before normal completion, its task guard reopens the start gate so a later request can
  submit maintenance again.

* **Poisoning** — an explicit poison signal through captured connection metadata removes the named record or
  generation from future dispatch. Accepted work may finish, but the connection does not return to accepting
  or idle state.
* **Protocol close** — peer close, HTTP/2 `GOAWAY`, driver or dispatcher completion, and transport- or
  protocol-level connection failure retire the affected connection. Streams the peer still accepts may
  finish; later requests use a replacement. An H2 stream-local reset is not connection-fatal.
* **Incomplete H1 exchange** — cancellation, response-body drop, or an exchange error retires H1 when no
  independent connection-fatal signal has already selected `ProtocolClosed`, unless Hyper proves that it
  recovered the complete message boundary.
* **Upgrade** — an H1 connection leaves HTTP pool ownership when its transport transfers to the upgraded
  protocol. H2 extended `CONNECT` follows the request stream's upgrade bridge and does not retire the physical
  H2 connection.
* **Reclaim** — a bounded origin at its limit closes a connection to move its permit to a waiting cell, as
  [Bounded-capacity coordination](#bounded-capacity-coordination) describes. Reclaim never interrupts an in-flight
  request: it closes a connection that is idle now, or reserves one as it returns from its current request and
  closes it before it serves another. It does not abort active work to free a permit.
* **Pool or owner-runtime shutdown** — pool drop logically closes every remaining record. If an owning runtime
  drops a connection's guarded driver task, the driver lifecycle guard requests logical close with
  `OwnerRuntimeShutdown`; dropping the driver also closes its root transport unless an H1 upgrade already
  transferred that transport. Per-stream and upgrade tasks use their request-lifecycle cleanup and do not by
  themselves close a healthy H2 connection. Outstanding request and body guards run their normal terminal
  cleanup, and every close request races through the same exactly-once transition.

The first trigger to begin logical close removes reuse eligibility, records the close cause, and normally
releases capacity. An H1 connection with one committed exchange that can still prove an upgrade retains capacity
until the exchange resolves that ambiguity. A non-upgrade resolution returns it then; an upgrade retains it
until physical completion. Later close triggers observe the same terminal transition and cannot release
capacity or report close again. Hyper can complete the protocol driver in the same poll that delivers the
upgrade response, so a request path that later confirms the upgrade may change `ProtocolClosed` to `Upgraded`
without repeating logical close or its callback. The refinement emits a structured diagnostic record carrying
both reasons so tracing reflects the final classification even when driver completion won the close race.

`Poisoned` is reserved for an explicit poison signal. `ProtocolClosed` is final when it reflects independently
observed connection-level termination; only later confirmation that the same H1 exchange upgraded may refine
it. The close event carries the source error when one exists. Other concurrent signals still race through
first-trigger-wins, but one initiating signal does not match both categories. Every reason ends at the same
physical connection ownership transition, and the H1 disposition determines the one valid capacity-release
point.

#### Obligations

* **Capacity ownership on close** [safety] — an ordinary draining connection releases its permit at logical
  close. An H1 exchange that may prove an upgrade retains the permit until classification; a confirmed upgrade
  retains it until physical completion. Every path releases the permit exactly once.
* **No dispatch after logical close** [safety] — dispatch commit and logical close race through mutually
  exclusive per-record or per-generation gates; a close that wins leaves the request locally owned, while a
  commit that wins is recorded as an in-flight dispatch and calls Hyper immediately without holding a pool
  lock.
* **Poison on retire** [safety] — a connection retired as unsafe is not returned to the idle set.
* **Reclaim spares active work** [safety] — reclaim closes a connection only when it is idle, whether idle
  now or on return from its current request; it never interrupts an in-flight request.
* **H1 boundary return** [safety] — H1 returns only after response end-of-stream or a Hyper-proven drain and a
  successful sender-ready check; pending readiness is owned by an owner-partition task, and dropping a body
  alone never returns the connection.
* **H2 stream isolation** [safety] — completion, error, or cancellation releases or resets one H2 request
  claim, after both sides finish, without retiring a healthy accepting generation.
* **Return revalidation** [safety] — an H1 return checks generation and retirement state under its
  connection-owning cell before becoming visible; a sender awaiting admission remains owned by that cell and
  non-dispatchable in `ReservedForPeer`, so a late completion cannot reverse logical close or bypass return
  ordering.
* **Physical connection completion** [safety] — dropping the wrapped root I/O, not logical close or
  driver-future completion alone, releases the client's physical connection handle, including after H1
  upgrade. The operating system may continue TCP teardown afterward.

### Telemetry

Request results do not reveal whether a request reused a connection, opened a
new one, waited for capacity, or observed a connection closing. The pool exposes
three complementary observations:

- request-attempt telemetry reports dispatch and connection selection facts
  through an optional request extension;
- lifecycle events report completed transitions; and
- statistics report current origin or partition-origin state.

None participates in admission, reuse, reclaim, or dispatch.

#### Lifecycle events

One listener receives a non-exhaustive event enum:

```rust
pub trait ConnectionEventListener: Send + Sync + 'static {
    fn on_event(&self, event: &ConnectionEvent<'_>);
}

#[non_exhaustive]
pub enum ConnectionEvent<'a> {
    EstablishmentFailed(ConnectionEstablishmentFailed<'a>),
    Opened(ConnectionOpened<'a>),
    LogicalClose(ConnectionLogicalClose<'a>),
    PhysicalClose(ConnectionPhysicalClose<'a>),
}
```

The pool assigns each establishment a `ConnectionEstablishmentId`. A retry
starts another establishment and receives another identity; earlier failed
attempts are not folded into a later successful event. Establishment events
carry the canonical origin, owner partition, stage, selected protocol and
remote address when known, and timing for transport and Hyper protocol
handshake work.

`Opened` is emitted after Hyper produces the protocol request handle and the
connection is installed as pool supply. It carries the establishment
observation and the installed connection's immutable `ConnectionInfo`.
Successful establishment measurements are frozen after protocol installation
and before the connection is published to waiting demand:

```text
ConnectionEstablishmentInfo
  establishment ID
  origin
  owner partition

ConnectionInfo
  connection ID
  origin
  owner partition
  HTTP protocol
  local and remote addresses, when known
  direct, forward-proxy, or proxy-tunnel path
```

The installed lifetime has two close observations:

```text
Opened
  |
  v
LogicalClose     connection no longer accepts pool dispatch
  |
  v
PhysicalClose    client released root transport ownership
```

`LogicalCloseCause` records the first policy or protocol cause that ended pool
dispatch ownership. It deliberately maps both HTTP/1 upgrade transfer and
ordinary protocol-driver completion to `ProtocolEnded`; at logical close the
pool may not yet know which final physical disposition applies.

`PhysicalClose` carries the final `CloseReason`. For an HTTP/1 upgrade this
event occurs when caller-owned upgraded I/O releases the wrapped root transport,
not when the connection first leaves the pool. The operating system may
continue TCP teardown after the client releases that handle.

Connection state can close while the synchronous `Opened` callback is running.
The authoritative close transition is not delayed, but callback progress
preserves:

```text
Opened -> LogicalClose -> PhysicalClose
```

Callbacks run after the transition releases every pool lock. Counter updates
also precede the corresponding callback, so a listener may query statistics
that already reflect the event. Concurrent connections have no total event
order.

#### Connection statistics

Origin statistics expose the authoritative capacity budget for one canonical
origin:

```rust
#[non_exhaustive]
pub struct OriginConnectionStats {
    capacity: Option<ConnectionCapacityStats>,
}

#[non_exhaustive]
pub struct ConnectionCapacityStats {
    limit: usize,
    in_use: usize,
}

impl ConnectionPool {
    pub fn origin_stats(&self, origin: &OriginKey) -> OriginConnectionStats;
}
```

`capacity` is `None` when the origin is unbounded. For a bounded origin,
`in_use` is read from admission's conserved capacity budget rather than
reconstructed from protocol counts. It includes permits held by establishment,
logically open connections, and detached HTTP/1 upgrades. Draining connections
that already returned capacity are excluded.

Partition statistics expose one exact `(PartitionId, OriginKey)` cell:

```rust
#[non_exhaustive]
pub struct PartitionConnectionStats {
    pending_acquisitions: usize,
    establishing_connections: usize,
    h1: Http1ConnectionStats,
    h2: Http2ConnectionStats,
    physically_live_connections: usize,
}

#[non_exhaustive]
pub struct Http1ConnectionStats {
    idle: usize,
    active: usize,
    draining: usize,
    upgraded: usize,
}

#[non_exhaustive]
pub struct Http2ConnectionStats {
    accepting: usize,
    draining: usize,
    active_requests: usize,
}

impl ConnectionPool {
    pub fn partition_stats(
        &self,
        partition: PartitionId,
        origin: &OriginKey,
    ) -> Option<PartitionConnectionStats>;
}
```

An unknown partition returns `None`. A configured partition without a retained
cell for the origin returns zeroed statistics. Neither query creates admission
or cell state, and the pool does not scan all partitions to construct an
aggregate.

The fields have these meanings:

- `pending_acquisitions` counts requests still waiting for a terminal
  acquisition outcome, including queued demand, crossing delivery, capacity
  ready to establish, and submitted establishment;
- `establishing_connections` counts establishments that have not reported
  failure, successful installation, or internal supersession;
- H1 `idle` counts senders available for immediate local selection;
- H1 `active` counts senders selected or reserved outside idle storage, not
  necessarily requests currently transferring bytes;
- H2 `accepting` counts generations that may issue new activations;
- H2 `active_requests` counts accepted requests whose upload and response sides
  have not both finished;
- `draining` counts logically closed protocol connections that still own root
  transport I/O;
- H1 `upgraded` counts transferred upgraded connections whose root transport is
  still caller-owned; and
- `physically_live_connections` counts every root transport handle retained by
  the client, including handshaking, open, draining, and upgraded connections.

The snapshot combines two ownership domains without nesting locks:

1. one cell lock provides exact waiter, H1 sender, H2 generation, and H2 request
   counts;
2. relaxed atomics provide establishment, drain, upgrade, and physical-lifetime
   counts that may outlive a protocol record.

The complete result is not one atomic snapshot across those domains. Each
lock-owned observation is exact while its lock is held; relaxed counts are
nonnegative and converge after concurrent transitions settle. Statistics are
diagnostics, not a synchronization API or routing authority.

#### The listener contract

A listener runs synchronously on the request, establishment, maintenance, or
driver task that produces an event. It must not block on work that requires
progress from that same task.

Listener invocation occurs outside every pool lock. On unwind-capable builds,
the pool catches and logs listener panics after the authoritative transition;
the panic does not alter pool state or prevent required cleanup. Abort-on-panic
builds retain their normal process-abort semantics.

Installing no listener avoids establishment identity allocation, listener
cloning, and callback work. Successful establishment timing remains available
as immutable connection metadata. Request-attempt timing is read only when the
request carries its capture extension. Diagnostic connection counts remain
available independently of event configuration.

#### Obligations

* **Transition ownership** [safety] — each event is emitted by the transition
  that owns its final state, after releasing pool locks.
* **Installed event order** [safety] — one connection reports `Opened`, then
  `LogicalClose`, then `PhysicalClose`, with each event at most once.
* **Establishment terminality** [safety] — each establishment ends as failed,
  opened, or internally superseded exactly once.
* **Diagnostic isolation** [safety] — statistics never participate in
  admission, selection, reclaim, or dispatch.
* **No local-path coordination** [performance] — ordinary request dispatch does
  not update shared diagnostic counters or enter origin admission for
  statistics.
* **Non-creating queries** [performance] — statistics queries do not create an
  admission authority or partition-origin cell.

## Terminology

Terms whose everyday meaning would otherwise mislead. Everything else is defined where it is first used.

**Partition** — an establishment/driver placement and optional network-interface binding, identified by a
caller-owned `PartitionId` when explicit or `PartitionId::ANONYMOUS` in the default topology. An explicit
partition names its runtime at construction; the anonymous partition binds one on first establishment.

**Origin** — a canonicalized scheme, host, and port, `OriginKey`. The granularity at which connections are
interchangeable. Discovered at runtime.

**Cell** — one partition's connections for one origin, `OriginCell`. The two axes' intersection, and where
connections live.

**Connection-owning cell** and **requesting cell** — roles in cross-cell reuse. The connection-owning cell
retains the record, driver, socket, and placement. The requesting cell owns the demand that may borrow the
HTTP/1 request handle or receive reclaimed capacity.

**Permit** — the conserved unit of connection capacity. It has exactly one owner at a time and is moved or
released, never copied. *Capacity* is the aggregate quantity permits account for, used in sums and bounds.

**Demand** — a cell's standing signal that it could use one more connection. One fixed ticket per cell,
not one per request, so demand cannot accumulate.

**Borrow** — moving an exclusive HTTP/1 request handle to a peer cell for dispatch while leaving the
connection record, driver, and socket with the connection-owning cell.

**Reclaim** — closing a connection so its permit can move to another cell. Transfers capacity, not I/O.

**Capacity lease**, **request claim**, and **handle** — a capacity lease is the exclusive hold on one permit
and moves from establishment to the connection record. An H2 request claim owns one prospective or accepted
request's two-sided lifecycle but no permit. A dispatch handle can address a connection and owns neither kind
of capacity.

**Retry authority** — proof that the same request may be dispatched again. Only Hyper returning the original
request unsent from a reused connection creates this authority; request clonability does not.

**Retry acquisition** — returning a still pool-owned request to acquisition after its selected protocol state
becomes stale before dispatch. It is an internal state transition, not an SDK retry or authority to clone an
accepted request.

**Snapshot**, **revision**, and **delivery** — a demand snapshot or supply revision is a complete versioned
value submitted across lock domains. Delivery hands one owned permit or provisional H1 to one assigned demand.
`DemandScheduleState::PendingAssignment` excludes a second selection while `DeliveryGuard` owns the
one-to-one payload crossing toward the requesting cell.

**Attempt** and **flight** — an HTTP/1 establishment is an attempt, independent of other attempts; an
HTTP/2 establishment is a flight. Automatic attempts remain independent before ALPN, then atomically join or
install at most one post-ALPN H2 flight per cell. The flight record owns participant identities; the
connection-partition owner task owns handshake completion.

**Supply status** and **route** — supply status is a cell's versioned admission-facing projection of usable
connection state. Admission may install an H2 route in a requesting cell. The route names one exact
connection-owning cell and generation but owns no sender, socket, driver, or capacity.

**Logical close** — a connection stops accepting new work and releases its permit. **Physical connection
completion** — the client releases its transport handle. It may follow logical close by an unbounded interval
and does not assert that kernel socket teardown is complete, so live sockets can outnumber admitted connections;
`max_connections_per_host` bounds admitted connections, not file descriptors, and no finite bound on live
sockets follows from it.

**Draining generation** and **draining connection** — a draining H2 generation accepts no new request claims
while already accepted streams finish. A draining connection has logically closed and released its permit but
still has physically live root I/O, and is counted by `h1_draining` or `h2_draining`. An H2 record may be both
while its accepted streams and transport finish.

**Generation** — one uniquely identified incarnation of an installed HTTP/2
connection. It owns the authoritative request handle, accepting or draining
state, request counts, and connection capacity. A replacement connection has a
new generation identity so delayed work for the previous incarnation is
rejected.

**Demand generation** — one cell queue head's `DemandId`. A **snapshot version** orders complete publications
for that generation. Readers retain the newest publication and reject work for a retired generation.

**Obligation** — a duty a component owes, stated as one sentence an implementation either satisfies or does
not. `[safety]` obligations forbid a state; `[liveness]` obligations require an outcome; `[optimization]`
obligations bound a cost, and violating one is a regression rather than a defect.

## Correctness invariants

Each invariant states the property, what it rules out, and the obligations that enforce it. The obligations
themselves are defined in the Architecture sections; an invariant names them rather than restating them.
Optimization obligations are not invariants — violating one is a regression, not a defect — so they appear
here only where a cost bound is load-bearing for a correctness property.

**Capacity is conserved.** A bounded origin admits at most `max_connections_per_host` connections across all
partitions, and every permit has exactly one owner at all times. *Rules out:* admitting past the bound; a
leaked permit, which is capacity the pool can never reissue; a duplicated permit, which would let two
connections occupy one unit of capacity. *Enforced by:* Single permit owner, Driver termination closes the
record, and Capacity on logical close (one owner from admission through release, released exactly once),
Single delivery and Refunnelling (a permit crossing locks commits once or returns to admission), and
Losing-attempt cleanup and Flight cancellation (post-ALPN termination returns the establishment lease).
Capacity gates whether a new connection may be *established*; it does not gate dispatch on a connection
already admitted.

**Origins are total and canonical.** Every dispatched request maps to exactly one origin, and equivalent
spellings of one server map to one origin. *Rules out:* a request with no cell to resolve to; one server
splitting into two `OriginAdmission`s that each admit the full bound. *Enforced by:* Key totality,
Canonical key, and Version independence.

**Cell identity is stable.** A cell is never destroyed while its origin is reachable, and at most one cell
exists per (partition, origin). *Rules out:* a peer reference to a cell that has been freed or whose slot has
been reused; two cells racing into existence for one pair. *Enforced by:* Cell stability, Reference validity,
and Cell uniqueness.

**I/O stays on its owning partition.** A connection's socket, driver, and Hyper-spawned tasks run only on the
partition that established it, for the connection's life; reuse moves a dispatch handle and no I/O. *Rules
out:* a socket registered on one runtime and driven from another, which produces a cross-runtime wakeup on
every read and couples the connection's lifetime to a runtime that does not own it; bytes leaving an
interface the caller did not choose. *Enforced by:* Establishment placement, Driver placement, Hyper task
placement, Binding immutability, and Placement under transfer.

**No dispatch on an unusable connection.** No request is dispatched on a connection that has begun closing or
has been retired as unsafe. *Rules out:* use of a connection after logical close; drawing a connection a
prior error poisoned. *Enforced by:* No dispatch after logical close, Poison on retire, Reclaim spares active
work (reclaim closes a connection only while idle, never interrupting an in-flight request), Poisoned
generation, Return revalidation, and Same-instance dispatch.

**Dispatch and response ownership are continuous.** From selection through terminal response, exactly one
component owns the request and exactly one component owns the selected connection or stream cleanup. *Rules
out:* replaying a request that may have reached the wire; returning H1 while its response is still framed;
losing a checked-out connection or H2 request claim when a future is dropped; releasing an extended
`CONNECT` lease when its empty response body completes. *Enforced by:* Same-instance dispatch, Certified
retry, Continuous response ownership, Stage-local cancellation, Upgrade transfer, H1 boundary return, H2
stream isolation, Full-stream lease, and Return revalidation.

**A one-to-one resource is delivered exactly once.** A provisional H1 or capacity lease has one owner until it
is committed to one eligible waiter or refunnelled. *Rules out:* a lost resource while an eligible waiter
sleeps; a double delivery where one resource serves two waiters; a cancelled waiter retaining capacity.
*Enforced by:* Bounded demand and Snapshot ordering identify the live generation; Single delivery retains its
assignment through settlement; Refunnelling and Acknowledged progress give every rejection and drop a terminal
path. An H2 generation is not a one-to-one resource: its connection record retains capacity while generation
identity is visible to compatible local waiters and announced to eligible peer cells in bounded turns.

**No committed waiter starves.** An eligible committed waiter is served whenever a permit it may use becomes
reachable, and is not passed indefinitely by later arrivals. *Rules out:* unbounded overtaking; a resource
sitting idle while an eligible waiter waits; capacity stranded on a peer connection that returns reusable
without ever going observably idle; a newly visible H2 generation serving newer local arrivals while older
compatible waiters remain parked. *Enforced by:* Cross-cell order and Bounded overtaking (the oldest eligible
residence comes from a stored head), Return interception and Owning-cell fairness turn (a returning connection
reaches an older peer without starving its owning cell), Generation visibility priority (the generation gate
serves committed local waiters before newer arrivals), and Work-conserving service, with Bounded grant work
bounding the
coordination cost and Bounded peer discovery preventing peer searches from growing with partition count.
This holds only while progress is possible — it is not promised while every permit for the origin is held
indefinitely by active HTTP/2 work that the waiter is not eligible to use, a limit stated under
[Eligible requests make progress](#eligible-requests-make-progress).

**Observation cannot corrupt pool state.** Listener code runs outside pool locks and only after its triggering
transition is complete, so pool invariants do not depend on a listener succeeding. *Rules out:* a listener
observing or holding partially transitioned state; a panicking listener leaving committed state inconsistent;
a listener blocking coordination by retaining a pool lock. *Does not rule out:* a listener delaying or
ending the task that invokes it, or delaying work sequenced after its return. The creation callback is a
barrier before request visibility. *Enforced by:* Report locality, Panic
containment, and Creation before visibility.

## Future work

### Reclaiming quiescent origins

The initial pool retains each origin and its cells until pool drop. Retained route memory therefore grows with
partitions × origins ever touched even after the connections and waiters for those origins are gone. Keeping
the identities stable is safe: it preserves peer references and guarantees that a bounded origin has exactly
one admission authority, with no reclamation race on the local acquisition path.

Whole-origin reclamation is the viable future granularity because nothing outside an origin refers to one of
its cells. It still needs a protocol that makes a request which has resolved the old origin visible before a
concurrent removal can declare every cell quiescent; otherwise old and replacement admissions can coexist and
each admit the full bound. Revisit this after measuring the retained size of an empty cell and realistic
partition-by-origin cardinality. A cell-count ceiling is not an alternative because it converts memory growth
into request failure.

### Active HTTP/2 drain for cross-scope reclaim

A waiter may remain parked when an origin is bounded, every permit is held indefinitely by active H2
generations outside that waiter's reuse eligibility group, and no eligible connection, reclaimable H1 return,
or released permit appears. This requires sustained cross-scope H2 work, such as different network-interface
groups, and is visible through waiting, H2-active-stream, and per-partition admission statistics.

The initial pool does not forcibly drain active H2 work to recover that capacity. A future implementation could
mark an out-of-scope generation draining so it accepts no new streams, begin logical close, and release its
permit while accepted streams finish. Choosing a victim and balancing connection churn against new demand adds
a second fairness policy. Add it only if practical bounds and production-shaped multi-interface H2 traffic
reproduce the stall and provide evidence for that policy.

### HTTP/2 stream-credit pooling

The initial design keeps one accepting H2 generation per cell and does not pool peer
`SETTINGS_MAX_CONCURRENT_STREAMS` credit or open additional generations when that credit is exhausted. Hyper
continues to own stream-level readiness and flow control. The consequence is a possible throughput cliff
behind one generation rather than a correctness failure; configured partitions remain the explicit scaling
unit. Add stream-credit accounting or connection sets only after benchmarks demonstrate a material
stream-limit, flow-control, congestion-window, or throughput cliff and the Hyper integration can expose the
credit needed to make admission authoritative.

### Legacy builder shimming

The new pool and partitioned client can ship through additive APIs before they replace the existing
`Builder` and `ConnectorBuilder` internals. The later compatibility path should implement those legacy
builders as adapters onto this pool wherever their observable behavior can be represented faithfully, rather
than retaining a second connection-pool architecture behind deprecated entry points.

That shim requires a field-by-field audit of connector settings, idle defaults and nested-option semantics,
TCP and interface settings, proxy and TLS assembly, DNS overrides, runtime components, and custom connector
entry points. The implementation-neutral client behavior suites are the acceptance baseline. A legacy option
that cannot be mapped exactly remains on the old path until an explicit compatibility decision is made; the
shim must not silently approximate it. Once the surface is covered, the hyper-util legacy pool can be removed
as an implementation dependency rather than kept alive solely by old builders.

---

## Appendix A: Public API and module structure

Appendix A assembles the callable construction surface. Types that define a mechanism remain with the
Architecture section that explains them: [partitions](#partitions), [origins](#origins-and-cells),
[reuse scope](#eligibility-and-capacity), [connection retirement](#why-a-connection-retires), and
[telemetry](#telemetry).

### Construction

A pool is built once and shared; clients are cheap handles onto one resolved partition.

```rust
#[derive(Clone)]
pub struct ConnectionPool {
    /* private: shared configuration and partition/origin state */
}

impl ConnectionPool {
    pub fn builder() -> Builder<TlsUnset>;
}

#[derive(Clone)]
pub struct Client {
    /* private: shared pool and resolved partition */
}

impl Client {
    pub fn new(pool: &ConnectionPool) -> Result<Self, ClientBuildError>;
    pub fn from_partition(
        pool: &ConnectionPool,
        id: PartitionId,
    ) -> Result<Self, ClientBuildError>;
}

#[derive(Debug)]
pub struct ClientBuildError { /* private */ }

impl ClientBuildError {
    pub fn partition(&self) -> Option<PartitionId>;
}
```

`Client` implements the smithy runtime's `HttpClient` through the
[Smithy client boundary](#smithy-client-boundary): each returned HTTP connector carries operation policy while
sharing this client's pool and resolved partition. `Client::new` resolves `PartitionId::ANONYMOUS`; it succeeds
only for a pool built without explicit partitions. `Client::from_partition` resolves the supplied identity,
including the anonymous identity when it exists. Either returns `ClientBuildError` rather than panicking when
the pool has no such partition. Resolution happens once at client construction, so a request performs no
partition lookup. `ClientBuildError` implements `Error`; `partition` returns the unresolved identity for the
current error kind without making that kind exhaustive.

[`ConnectionPool::origin_stats`], [`ConnectionPool::partition_stats`], and the event API are specified with
telemetry rather than repeated here.

### Builder

TLS provider selection is the only typestate transition, and it gates only TLS configuration. Every other
setting is available in either state. Each setter has a `set_*` mirror taking `&mut self` and an `Option`, for
callers assembling configuration programmatically.

```rust
pub struct Builder<Tls = TlsUnset> {
    /* private: pool configuration and TLS typestate */
}

#[derive(Debug)]
pub struct BuildError { /* private */ }

impl<Tls> Builder<Tls> {
    pub fn idle_timeout(self, timeout: impl Into<Option<Duration>>) -> Self;
    pub fn set_idle_timeout(&mut self, timeout: Option<Option<Duration>>) -> &mut Self;
    pub fn time_source(self, source: impl TimeSource + 'static) -> Self;
    pub fn sleep_impl(self, sleep: impl AsyncSleep + 'static) -> Self;
    pub fn tcp_nodelay(self, nodelay: bool) -> Self;
    pub fn tcp_keepalive(self, time: impl Into<Option<Duration>>) -> Self;
    pub fn max_connections_per_host(self, n: usize) -> Self;
    pub fn connection_reuse_scope(self, scope: ConnectionReuseScope) -> Self;
    pub fn proxy_config(self, config: ProxyConfig) -> Self;
    pub fn dns_resolver(self, resolver: impl ResolveDns + 'static) -> Self;
    pub fn event_listener(self, listener: impl ConnectionEventListener) -> Self;
    pub fn set_event_listener(&mut self, listener: Option<SharedConnectionEventListener>) -> &mut Self;
    pub fn partitions(self, partitions: impl IntoIterator<Item = Partition>) -> Self;
}

impl Builder<TlsUnset> {
    pub fn tls_provider(self, provider: tls::Provider) -> Builder<TlsProviderSelected>;
    pub fn build_http(self) -> Result<ConnectionPool, BuildError>;

    // Test-only: gated behind `test-util` + `aws_sdk_unstable`. Injects a TCP-level
    // transport for tests; not general public surface, and does not honor interface binding.
    #[cfg(all(feature = "test-util", aws_sdk_unstable))]
    pub fn build_http_with_tcp_connector<C, IO>(
        self,
        connector: C,
    ) -> Result<ConnectionPool, BuildError>;
}

impl Builder<TlsProviderSelected> {
    pub fn tls_context(self, context: TlsContext) -> Self;
    pub fn build_https(self) -> Result<ConnectionPool, BuildError>;
}
```

Setters retain the supplied configuration and do not have eager and mutable forms with different validation
behavior. A terminal build validates the complete configuration and returns `BuildError` for a zero
`max_connections_per_host`, an explicitly supplied empty partition set, duplicate partition identifiers, or
an explicit partition using the reserved anonymous identity. `BuildError` reports the setting and value that
failed and implements `Error`; callers are not expected to branch on an exhaustive variant set.

When `partitions` is never set, construction creates the one anonymous, unbound partition. Once set, the
supplied nonempty set is the complete explicit topology and no anonymous partition is added.
`max_connections_per_host` is unset by default, and an unset bound constructs no admission machinery. When
set, it bounds one scheme-host-port origin across all partitions and interface groups, not per partition. HTTP
and HTTPS and distinct non-default ports are bounded separately. The limit counts every establishing, idle, and
active connection rather than only idle connections.

The initial builder exposes no pool-wide HTTP/1-only or HTTP/2-only policy. Accepted request forms that require
HTTP/1 wire semantics narrow the per-attempt offer to `http/1.1` when the connector supports it; other requests
use the default `h2, http/1.1` offer. An HTTP/2-marked request does not force an H2-only offer. A future
protocol-policy API requires a concrete Smithy caller and a definition of how that policy composes when
multiple clients share one pool.

The default rustls connector applies the per-attempt offer. The s2n connector's
fixed HTTP ALPN list cannot be narrowed by the pool. A connector injected
through the unstable test utility also owns its own protocol configuration and
does not receive the pool's offer. The pool validates the negotiated result but
does not replace either connector's TLS implementation. Making the s2n offer
configurable is an upstream connector follow-up.

Because those paths cannot force HTTP/1, bounded admission does not reclaim a
healthy idle H2 generation solely to serve H1-required demand. Closing that
generation could negotiate H2 again and discard useful capacity without
satisfying the request. The request remains queued until ordinary close
releases capacity.

An unset `idle_timeout` uses a 90-second default. Passing `None` to the fluent setter disables idle
timeout. The mutable setter preserves all three configuration states: outer `None` restores the default,
`Some(None)` disables the timeout, and `Some(Some(duration))` selects a duration. The builder's time source and
sleep implementation drive pool maintenance; their defaults are the production system clock and Tokio sleep.
They are pool inputs and are not replaced by per-operation `RuntimeComponents`.

Building retains configuration and assembles the reusable transport factory but opens no socket. Native trust
loading is deferred to the idempotent connector preflight invoked when the `Client` is selected by smithy, or
to first establishment when used without smithy validation. Whether a configured interface exists and can be
used is therefore reported later as a connector error on establishment, not `BuildError`. Likewise,
obtaining a Tokio handle for `DriverSpawner::tokio` is the caller's step and keeps Tokio's own panic
outside a runtime; it happens before the spawner is passed to the pool.

### Module structure

```text
aws-smithy-http-client/src/client/
  pool.rs              — ConnectionPool ownership and public re-exports
  pool/
    builder.rs         — Builder typestate, validation, connector assembly
    client.rs          — Client, PoolConnector, and ClientBuildError
    partition.rs       — partition declarations and runtime/interface placement
    origin.rs          — owned OriginKey, borrowed lookup, and canonicalization
    registry.rs        — PartitionRegistry, PartitionState, and stable cell ownership
    cell.rs            — OriginCell and cell-level acquisition coordination
    cell/
      h1.rs            — H1CellState, sender ownership, and peer reservation
      h2.rs            — HTTP/2 flights, generations, routes, and activation gates
      h2/
        request.rs     — HTTP/2 activation authority and two-sided request claims
      waiters.rs       — local acquisition queue and waiter resolution
    admission.rs       — bounded-origin capacity and unlocked action driving
    admission/
      demand.rs        — versioned demand order and assignments
      order.rs         — checked intrusive order shared by admission indexes
      h1.rs            — H1 supply indexes, retained matches, and detached actions
      delivery.rs      — capacity/H1 assignment handoff and settlement
      h2.rs            — H2 supply indexes, peer routing, and reclaim
    establish.rs       — negotiated-protocol routing and connection identity
    establish/
      h1.rs            — HTTP/1 handshake, installation, and driver
      h2.rs            — post-ALPN flight convergence, handshake, and driver
      transport.rs     — connector selection, placement, timeout, and ALPN inputs
    dispatch.rs        — protocol-neutral request routing
    dispatch/
      h1.rs            — HTTP/1 dispatch, non-acceptance, and response ownership
      h2.rs            — HTTP/2 dispatch and two-ended request completion
    maintenance.rs     — idle-deadline scheduling and partition task lifetime
    connection.rs      — connection identity, logical close, dispatch, and physical connection ownership
    events.rs          — listener and lifecycle event types
    stats.rs           — origin/partition snapshots and lifecycle gauges
aws-smithy-http-client/src/
  sync/
    mod.rs              — standard-library and Loom backend selection
    std.rs              — production synchronization facade
    loom.rs             — modeled synchronization facade
```

The inventory describes ownership boundaries, not implementation order. The `pool` module re-exports every
public type above; private modules may be split or combined without changing the architecture so long as the
lock, ownership, and hot-path boundaries remain intact.

The transport-connector contract below the pool is unchanged: it is a `Service<Uri>` yielding
`(IO, Connected)`. The pool composes transport connectors and does not replace that contract; the smithy
`HttpConnector` above the pool is the request-policy facade described earlier.

---

## Appendix B: Validation

Validation supplies evidence for the contracts above; it does not redefine
them.

| Concern | Required evidence |
| --- | --- |
| State and ownership | Focused transition tests and executable invariant checks. |
| Concurrency | Loom models using the production synchronization-bearing state machines. |
| Protocol behavior | Controlled-runtime and wire tests covering cancellation, close, reuse, multiplexing, upgrades, and failure. |
| Client compatibility | Implementation-neutral differential tests for request behavior, metadata, timeout scope, and error classification. |
| Performance and scaling | Allocation, contention, stress, and benchmark measurements under the relevant concurrency and partition topology. |

A correctness claim requires the applicable state, concurrency, protocol, and
compatibility evidence. A performance, bounded-work, or topology-scaling claim
requires measurements that exercise the claimed workload and bound. Focused
state-space enumeration identifies its operation alphabet and bound; an
ordinary transition test does not claim exhaustiveness.

---

## Appendix C: FAQ

### Why not build the pool from composable connector layers?

Hyper's ecosystem offers pooling as connector middleware — a cache layer, a connection-limit layer, a
negotiate layer, each a `Service` wrapping the one below. The pool owns the coordination layer because the
state it coordinates is not local to any one layer, and stacked layers give no layer the whole picture.

Reuse and admission illustrate it. A connection limit as a middleware layer parks a request until a permit
frees. A permit normally frees on logical close; an H1 connection that may have transferred upgraded I/O keeps
it until final classification or physical completion. Reuse is a different layer, and it wakes a waiter when
a connection returns to the idle set. Nothing connects the two: a request parked for a permit is not waiting
on the idle set, so an idle return does not wake it, and a request parked for reuse is not waiting on a permit,
so a close does not wake it. This already breaks in a single partition — a capacity-bound waiter is not woken
by the idle return that should satisfy it — and it is not a tuning bug in one layer but a consequence of the
lifecycle being split across layers that do not share a view of it. Coordinating capacity across partitions,
where a permit freed in one partition must wake a waiter another parked, is a further step the layered
stack has no structure to take at all; borrow and reclaim exist precisely because that path has to be a
first-class operation.

The [composable-pool prototype](https://github.com/smithy-lang/smithy-rs/pull/4708) had to vendor the cache
layer and carry SDK-specific modifications. Owning the cache, limit, and negotiate layers as one unit gives
them one lifecycle view. The pool therefore forgoes future upstream improvements to those layers, so its
equivalents must be as strong or stronger. The connector contract below the pool and Hyper's protocol
implementation above it remain unchanged.
