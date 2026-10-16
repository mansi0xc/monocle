// Generated from a real local verification run of Monocle synthetic fixtures.
// Public test keys and invented context; not network evidence.
window.MONOCLE = {
 "note": "Captured from a real local Rust/WASM verification run of Monocle synthetic fixtures (public test keys, invented context). Not network evidence.",
 "example": {
  "title": "Fast finality",
  "description": "One aggregate certificate meets the fast-path threshold.",
  "provenance": "C4 synthetic composition case: fast_finality; C3 public keys/context.",
  "bundleDigest": "sha256:a6274a1674edbf3fb8691318418ee92265104c3ba5a55ca90f6555f613bea371",
  "slot": "42",
  "epoch": "42",
  "blockId": "4242424242424242424242424242424242424242424242424242424242424242",
  "profile": "agave-98535e50-base2-v1",
  "policy": "agave-98535e50-direct-finality-v1",
  "runId": "a88ad4cc-3eda-4507-b8a6-ce73351b672e",
  "genesisHash": "1111111111111111111111111111111111111111111111111111111111111111",
  "shredVersion": 1234,
  "rawHex": "01082a000000000000004242424242424242424242424242424242424242424242424242424242424242018184e83b377a4321ae29e4b965caee4deb473d3d165156d31f847c0d761d907c2f63a3ae0f37e015210b074dda0c56127015a0dea35a1cb6852f25e9afe892d621ca04bf2f20b189739f96b49f66ef1d60121b012ac325ade1711d2681b01f04740933c8799dd56d82de8d9b576a70f1fa70aa909645063ef6a13c81a2bb4e41dc3bdf57ccae2035f38177a131092216bdf7f165a226584bca47f3af8aabab6bb63f7a23b00e60a648fbb7a74cebc674986139995e53773450c27f03c9942504000000000000000005000bd204",
  "format": "agave-versioned-consensus-v1"
 },
 "checkpoint": {
  "name": "Synthetic epoch 42",
  "digest": "sha256:239781156fdc02b1ea0b1823f40068625c6e8074b7918cc140e8b7b9e08908dd",
  "total": "100",
  "provenance": "Public seeds [11;32] through [15;32]; invented epoch/stake context; no operational keys.",
  "basis": "Synthetic fixture only",
  "validators": [
   {
    "rank": 0,
    "stake": "40",
    "identity": "0101010101010101010101010101010101010101010101010101010101010101",
    "bls": "157467ef01661798515186269581fd323fc8fd8fb05215d6944b2f2e742400c92778fe84fd1347b97f4159be4451966b0752ce29e65b5f605c929b6925d758220c1acb86800c5dc7947f37323458d9242428a4fabcde400a84e74e8a8262cdc1",
    "pop": true
   },
   {
    "rank": 1,
    "stake": "25",
    "identity": "0202020202020202020202020202020202020202020202020202020202020202",
    "bls": "15c232890a833355c7f53d6189ba790c5bf90cc1ba28b5c5e21c83b66dea2581d5c4f28ddd40ef4d57da5fdb8d0e60c4183bfed4dfe2c3613f4594acbf8ba2409d80944d996e6a4ef0954c5419a9b95e72f54ea6c741604d7d4c6719684ededf",
    "pop": true
   },
   {
    "rank": 2,
    "stake": "18",
    "identity": "0303030303030303030303030303030303030303030303030303030303030303",
    "bls": "0542f41118f4ad70678c7730ec6442288c6d0037e7e6d13fe24474b77a29bee3db41e1a250b32a655bf8fa75709d026315bec606cf2e23678b9284665557f0afdf552b84712aeb973f8e133c4115e5d1bc304edcd35939f35670412eb59983ea",
    "pop": true
   },
   {
    "rank": 3,
    "stake": "15",
    "identity": "0404040404040404040404040404040404040404040404040404040404040404",
    "bls": "0592ec12e3a78129393f3c3a615aec8a0d1ccfe48c18f7d87b501c3f3b00fc6d7be6b9aedfffe66ff6d4d19bc57aa38c144dc26d509e2ae18298a04b06ffc3328354f3f7db0dd03c8aff77ea707ecd5de3b86244d8c7eef9de4f20402a629387",
    "pop": true
   },
   {
    "rank": 4,
    "stake": "2",
    "identity": "0505050505050505050505050505050505050505050505050505050505050505",
    "bls": "0a8350ae13aaa4c1eda3bf88b2ff4dcf7a5d9d78bbca70ba38b63781eacb72f7813b4f447f812a56805cb3967d7a900717cd1253fb89db9f5bc7a7faa2d5b6a9d2cda224f74d4e464feffaad224ee050f95db3d853ce9e91e298e6a4685ae616",
    "pop": true
   }
  ]
 },
 "report": {
  "conclusion": "VERIFIED",
  "title": "Fast finality verified",
  "scope": "block_finality",
  "path": "fast",
  "checks": [
   {
    "stage": "input",
    "status": "PASS",
    "code": "OK",
    "message": "Bounded schema and transport checks passed; certificate bytes remain unverified"
   },
   {
    "stage": "profile",
    "status": "PASS",
    "code": "OK",
    "message": "Recognized profile metadata; no certificate verification is performed by preflight"
   },
   {
    "stage": "context",
    "status": "PASS",
    "code": "OK",
    "message": "Explicit local selection matches declared context; authenticity remains the caller's assumption"
   },
   {
    "stage": "finality",
    "status": "PASS",
    "code": "FAST_FINALITY_ESTABLISHED",
    "message": "A valid fast-finalization certificate binds this block and meets the 80% threshold"
   }
  ],
  "assumptions": [
   "Local synthetic fixture selection; no real validator-set authenticity claim",
   "Trust selection alone does not qualify BLS keys; later checks must validate each selected signer's PoP",
   "PoP policy: verify each selected key's standard compressed-public-key-bound proof with no additional application payload. This does not authenticate the checkpoint or prove a vote-account registration.",
   "Direct evidence only, conditional on the pinned protocol's consensus safety assumptions and the selected checkpoint. No ancestry, execution, epoch-transition, or network-activation proof is evaluated. REJECTED describes supplied evidence, not proof that a block is non-final."
  ],
  "cert": {
   "id": "fast",
   "kind": "fast_finalization",
   "conclusion": "VERIFIED",
   "selected": "80",
   "total": "100",
   "threshold": 80,
   "ranks": [
    0,
    1,
    3
   ],
   "rawDigest": "sha256:9137e83de93b68d4c0169725cc1eb878b9baa1d552c8c6f80e632e2928dca6fb",
   "payload": "012a000000000000004242424242424242424242424242424242424242424242424242424242424242d204",
   "checks": [
    {
     "stage": "certificate_decoding",
     "status": "PASS",
     "code": "OK",
     "message": "Supported complete wire message and declared context agree"
    },
    {
     "stage": "signers",
     "status": "PASS",
     "code": "OK",
     "message": "Unique Base2 signer positions resolve in the selected immutable rank map"
    },
    {
     "stage": "key_qualification",
     "status": "PASS",
     "code": "OK",
     "message": "All selected keys passed checked point decoding and actual public-key-bound PoP verification"
    },
    {
     "stage": "signature",
     "status": "PASS",
     "code": "OK",
     "message": "BLS aggregate verifies against the reconstructed source vote bytes"
    },
    {
     "stage": "quorum",
     "status": "PASS",
     "code": "OK",
     "message": "Exact integer stake threshold is met"
    }
   ]
  }
 },
 "experiment": {
  "mutation": {
   "after_hex": "00",
   "before_hex": "01",
   "certificate_id": "fast",
   "offset": "42",
   "operation": "replace_bytes",
   "parent_certificate_digest": "sha256:9137e83de93b68d4c0169725cc1eb878b9baa1d552c8c6f80e632e2928dca6fb",
   "result_certificate_digest": "sha256:7a293a8f12bd993206320d4bd77b9490eb16c60db2cacfa27b77c45bc29aa46c",
   "schema_version": 1
  },
  "digest": "sha256:6f87d76152c94071f912e2c6cff6359815fb275dfa547c6841c3a7aad75e8982",
  "conclusion": "REJECTED",
  "checks": [
   {
    "stage": "input",
    "status": "PASS",
    "code": "OK",
    "message": "Bounded schema and transport checks passed; certificate bytes remain unverified"
   },
   {
    "stage": "profile",
    "status": "PASS",
    "code": "OK",
    "message": "Recognized profile metadata; no certificate verification is performed by preflight"
   },
   {
    "stage": "context",
    "status": "PASS",
    "code": "OK",
    "message": "Explicit local selection matches declared context; authenticity remains the caller's assumption"
   },
   {
    "stage": "finality",
    "status": "NOT_CHECKED",
    "code": "CERTIFICATE_CHECKS_FAILED",
    "message": "A selected certificate failed; inspect its check trace. Finality composition was not performed"
   }
  ],
  "certChecks": [
   {
    "stage": "certificate_decoding",
    "status": "PASS",
    "code": "OK",
    "message": "Supported complete wire message and declared context agree"
   },
   {
    "stage": "signers",
    "status": "PASS",
    "code": "OK",
    "message": "Unique Base2 signer positions resolve in the selected immutable rank map"
   },
   {
    "stage": "key_qualification",
    "status": "PASS",
    "code": "OK",
    "message": "All selected keys passed checked point decoding and actual public-key-bound PoP verification"
   },
   {
    "stage": "signature",
    "status": "FAIL",
    "code": "INVALID_SIGNATURE",
    "message": "Signature is malformed, off curve, or outside the prime-order subgroup"
   },
   {
    "stage": "quorum",
    "status": "NOT_CHECKED",
    "code": "PREREQUISITE_MISSING",
    "message": "Earlier certificate prerequisites have not passed"
   }
  ]
 },
 "others": {
  "slow_finality": {
   "conclusion": "VERIFIED",
   "scope": "block_finality",
   "finality": {
    "stage": "finality",
    "status": "PASS",
    "code": "SLOW_FINALITY_ESTABLISHED",
    "message": "Notarization and slot-only finalization each meet 60%, agree on slot/context, and notarization binds the claimed block. Their stakes are not added"
   },
   "certs": [
    {
     "id": "notar",
     "kind": "notarization",
     "conclusion": "VERIFIED",
     "selected": "65",
     "total": "100",
     "threshold": 60,
     "ranks": [
      0,
      1
     ],
     "failing": null
    },
    {
     "id": "finalize",
     "kind": "finalization",
     "conclusion": "VERIFIED",
     "selected": "65",
     "total": "100",
     "threshold": 60,
     "ranks": [
      0,
      1
     ],
     "failing": null
    }
   ]
  },
  "notar_only": {
   "conclusion": "INCONCLUSIVE",
   "scope": "unestablished",
   "finality": {
    "stage": "finality",
    "status": "NOT_CHECKED",
    "code": "MISSING_FINALIZATION",
    "message": "Notarization alone is insufficient; provide matching finalization or a supported fast-finalization certificate"
   },
   "certs": [
    {
     "id": "notar",
     "kind": "notarization",
     "conclusion": "VERIFIED",
     "selected": "65",
     "total": "100",
     "threshold": 60,
     "ranks": [
      0,
      1
     ],
     "failing": null
    }
   ]
  },
  "changed_signature": {
   "conclusion": "REJECTED",
   "scope": "unestablished",
   "finality": {
    "stage": "finality",
    "status": "NOT_CHECKED",
    "code": "CERTIFICATE_CHECKS_FAILED",
    "message": "A selected certificate failed; inspect its check trace. Finality composition was not performed"
   },
   "certs": [
    {
     "id": "notar",
     "kind": "notarization",
     "conclusion": "REJECTED",
     "selected": "58",
     "total": "100",
     "threshold": 60,
     "ranks": [
      0,
      2
     ],
     "failing": {
      "stage": "signature",
      "status": "FAIL",
      "code": "INVALID_SIGNATURE",
      "message": "Aggregate signature does not verify for the exact signed payload and selected keys"
     }
    },
    {
     "id": "finalize",
     "kind": "finalization",
     "conclusion": "VERIFIED",
     "selected": "65",
     "total": "100",
     "threshold": 60,
     "ranks": [
      0,
      1
     ],
     "failing": null
    }
   ]
  },
  "fast_under_quorum": {
   "conclusion": "REJECTED",
   "scope": "unestablished",
   "finality": {
    "stage": "finality",
    "status": "NOT_CHECKED",
    "code": "CERTIFICATE_CHECKS_FAILED",
    "message": "A selected certificate failed; inspect its check trace. Finality composition was not performed"
   },
   "certs": [
    {
     "id": "fast",
     "kind": "fast_finalization",
     "conclusion": "REJECTED",
     "selected": "65",
     "total": "100",
     "threshold": 80,
     "ranks": [
      0,
      1
     ],
     "failing": {
      "stage": "quorum",
      "status": "FAIL",
      "code": "INSUFFICIENT_STAKE",
      "message": "Valid signature has insufficient stake under the original checkpoint denominator"
     }
    }
   ]
  },
  "genesis_anchor": {
   "conclusion": "VERIFIED",
   "scope": "genesis_migration",
   "finality": {
    "stage": "finality",
    "status": "PASS",
    "code": "GENESIS_ANCHOR_ESTABLISHED",
    "message": "A valid genesis certificate binds the claimed migration anchor and meets 82%; this is not ordinary block finality"
   },
   "certs": [
    {
     "id": "genesis",
     "kind": "genesis",
     "conclusion": "VERIFIED",
     "selected": "82",
     "total": "100",
     "threshold": 82,
     "ranks": [
      0,
      1,
      3,
      4
     ],
     "failing": null
    }
   ]
  }
 },
 "capture": {
  "slot": "444625255",
  "conclusion": "INCONCLUSIVE",
  "adapter": "agave-genesis-rpc-v1",
  "digest": "sha256:fb00f5a8912f597f25b8a840655270f0912681201aeb7744263d9aeb77630f7d",
  "blockId": "59ad7c6f9810400fb7746fb84393a776d3080069d06224ace01bb3ae98a1196e",
  "checks": [
   {
    "stage": "input",
    "status": "PASS",
    "code": "OK",
    "message": "JSON-RPC envelope parsed; endpoint identity is not authenticated"
   },
   {
    "stage": "certificate_decoding",
    "status": "PASS",
    "code": "OK",
    "message": "Extracted result.block.slot, block ID, signature bytes and bitmap; cryptographic validity has not been checked"
   },
   {
    "stage": "profile",
    "status": "NOT_CHECKED",
    "code": "CAPTURE_CONTEXT_MISSING",
    "message": "Raw RPC inspection lacks authenticated historical context and a confirmed verification profile; this stage did not run"
   },
   {
    "stage": "context",
    "status": "NOT_CHECKED",
    "code": "CAPTURE_CONTEXT_MISSING",
    "message": "Raw RPC inspection lacks authenticated historical context and a confirmed verification profile; this stage did not run"
   },
   {
    "stage": "signers",
    "status": "NOT_CHECKED",
    "code": "CAPTURE_CONTEXT_MISSING",
    "message": "Raw RPC inspection lacks authenticated historical context and a confirmed verification profile; this stage did not run"
   },
   {
    "stage": "key_qualification",
    "status": "NOT_CHECKED",
    "code": "CAPTURE_CONTEXT_MISSING",
    "message": "Raw RPC inspection lacks authenticated historical context and a confirmed verification profile; this stage did not run"
   },
   {
    "stage": "signature",
    "status": "NOT_CHECKED",
    "code": "CAPTURE_CONTEXT_MISSING",
    "message": "Raw RPC inspection lacks authenticated historical context and a confirmed verification profile; this stage did not run"
   },
   {
    "stage": "quorum",
    "status": "NOT_CHECKED",
    "code": "CAPTURE_CONTEXT_MISSING",
    "message": "Raw RPC inspection lacks authenticated historical context and a confirmed verification profile; this stage did not run"
   },
   {
    "stage": "finality",
    "status": "NOT_CHECKED",
    "code": "CAPTURE_CONTEXT_MISSING",
    "message": "Raw RPC inspection lacks authenticated historical context and a confirmed verification profile; this stage did not run"
   }
  ],
  "requirements": [
   "Historical validator checkpoint: canonical rank order, BLS public keys, exact stakes and original denominator, authenticated independently.",
   "Historical signing context: shred version, epoch and genesis hash, with evidence linking them to this certificate; do not infer them from a current RPC bank.",
   "Key qualification compatible with the verifier's PoP policy; a registration flag or different proof payload is not a substitute.",
   "Producer/profile compatibility: confirm the capture's vote serialization and signer encoding against the pinned implementation before constructing verification input."
  ],
  "assumptions": [
   "The response is untrusted local evidence. Its digest identifies this file, not its sender or capture time.",
   "The selected adapter describes getAgGenesisCert; a JSON-RPC response does not itself name the method.",
   "Capture inspection never selects a checkpoint, reconstructs a signed payload, or establishes network finality."
  ]
 }
};
