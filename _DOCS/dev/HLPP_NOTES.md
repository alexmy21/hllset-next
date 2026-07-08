# HLPP Notes

## Token level

### Token

```text
<token | TF>
```

### hash

```text
<token | TF> -> <hash>
```

In previous design we used TF associated with hash. The reason was simple - we cannot distinguish tokens under the same hash but we still have a problem of choosing one - assigning TF to specific token can help with this choice.

Hash TF equal $\sum {\text {TF}_{token}}$.

### Hash to HLLSet register

Each hash is associated with a position in HLLSet: P leading bit of the hash value points to 32-bit register in HLLSet; number of trailing zeros - points to the position in this 32-bit vector.

The number of all possible <reg, zeros> is known and fixed by $2^P \times 32$, in case of P=10 it's equal $32 \times 1024$ bits.

These bits are shared by all HLLSets, so we can use single rank vector (TF) that would serve them. Bit ranks can be then split by token's hashes, and final split is splitting hash rank by actual tokens.

### LUT as hierarchy from HLLSet bits to collected tokens

- <reg, zeros> layer (L2);
- collections of hashes for each <reg, zeros> bit in HLLSet (L1);
- collection of tokens (1 in trivial case) for each hash value (L0).

Each node in LUT hierarchy associated with the rank. Any monotonic function of the accumulated rank can be used as a measurement of the node rank.

LUT structure is a tree - it means that all nodes in the same layer and branches are mutually exclusive.

## Commit

Commit is temporal decomposition of the HLLSet System. Different commits composed from mutually exclusive HLLSet building elements: hashes and other composite (compound) HLLSets. So, new hashes ingested in different sessions are mutually exclusive as well as new original HLLSets and compound HLLSets built from them.

```text
    H(t)   =   (S(t),    H(t-1),   D(t-1),    R(t-1),   N(t))
     ^           ^         ^          ^          ^       ^
     |           |         |          |          |       |
    HLLSet     HLLSet    HLLSet    HLLSet     HLLSet   HLLSet 
```

where:

- H(t) - current state of HLLSet System;
- S(t) - is the temporal slice of the t-commit, everything new that created in current session (all new hashes and new HLLSets);
- H(t-1) - previous state of HLLSet System;
- D, R, N - DRN difference between current and previous state.

## HLLSet

HLLSet is a sample from the system bit-vector - "bits are shared by all HLLSets". Immediate update of ranks for all registers in all HLLSets after updating ranks of the bits in shared bit-vector.

## HLLSet Lattice

Append only HLLSet structure. With commit lattice has explicit temporal presentation of HLLSet system dynamics.

IICA (Immutable, Idempotent, Content Addressed) HLLSet architecture provides great flexibility in writing memory to persistent store, including  IPFS -  no specific order and unlimited retry. We can consider WAL (Write Ahead Log) pattern to manage commit transaction, or any other similar mechanism that can guaranty consistency.

In HLLSet system memory is just sub-structure of the whole HLLSet structure in IPFS.

## Temporal objects

Any object that is needed to maintain temporal changes with the same object ID is a temporal object. Commit is temporal object - it uses timestamp based unique ID that is not CA. The other examples of temporal objects are global HLLSets - G1, G2, G3. HLLSets representing metadata (as in case DB metadata).
