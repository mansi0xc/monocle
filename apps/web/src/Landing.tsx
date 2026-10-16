import { Arrow, Brand } from "./Brand";
import "./landing.css";

const source = "https://github.com/mansi0xc/monocle";

/*
 * Values below restate results produced by Monocle's signed synthetic fixtures
 * (fixtures/finality/corpus.json, public test keys and invented stake). The
 * landing page never loads evidence or runs the verifier.
 */
const outcomes = [
  {
    name: "Fast finality",
    conclusion: "VERIFIED",
    code: "FAST_FINALITY_ESTABLISHED",
    why: "One fast-finalization certificate with 80 of 100 stake binds the block.",
  },
  {
    name: "Slow finality",
    conclusion: "VERIFIED",
    code: "SLOW_FINALITY_ESTABLISHED",
    why: "Notarization and finalization each reach 60%. Their stakes are never added.",
  },
  {
    name: "Missing finalization",
    conclusion: "INCONCLUSIVE",
    code: "MISSING_FINALIZATION",
    why: "A valid notarization alone leaves the block claim incomplete.",
  },
  {
    name: "Changed signature",
    conclusion: "REJECTED",
    code: "INVALID_SIGNATURE",
    why: "The aggregate no longer verifies for the exact signed payload.",
  },
  {
    name: "Insufficient stake",
    conclusion: "REJECTED",
    code: "INSUFFICIENT_STAKE",
    why: "A genuine signature from 65 of 100 stake misses the 80% fast path.",
  },
  {
    name: "Genesis anchor",
    conclusion: "VERIFIED",
    code: "GENESIS_ANCHOR_ESTABLISHED",
    why: "82% signs the migration anchor. It is reported separately from block finality.",
  },
  {
    name: "Recorded RPC response",
    conclusion: "INCONCLUSIVE",
    code: "CAPTURE_CONTEXT_MISSING",
    why: "A recorded getAgGenesisCert response is inspected but stays inconclusive without its historical checkpoint.",
  },
] as const;
const glyph = { VERIFIED: "✓", REJECTED: "×", INCONCLUSIVE: "—" } as const;

/* Ridge geometry: x in a 1440-wide box, stake percentage mapped to height. */
const H = 560;
const y = (stake: number) => 540 - stake * 4.6;
const ridge: [number, number][] = [
  [0, y(2)],
  [420, y(6)],
  [700, y(40)],
  [880, y(65)],
  [1040, y(80)],
  [1130, y(70)],
  [1260, y(50)],
  [1360, y(34)],
  [1440, y(28)],
];
const ridgeLine = ridge.map((p) => p.join(" ")).join(" L");
const pct = (value: number, of: number) => `${(value / of) * 100}%`;

function Ridge() {
  return (
    <div className="ridge" aria-hidden="true">
      <svg viewBox={`0 0 1440 ${H}`} preserveAspectRatio="none">
        <defs>
          <linearGradient id="ridge-rock" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0" stopColor="var(--ridge-top)" />
            <stop offset="1" stopColor="var(--night)" />
          </linearGradient>
          <linearGradient id="ridge-edge" x1="0" y1="0" x2="1" y2="0">
            <stop offset="0" stopColor="var(--amber)" stopOpacity="0" />
            <stop offset=".55" stopColor="var(--amber)" />
            <stop offset=".72" stopColor="var(--rose)" />
            <stop offset=".9" stopColor="var(--orchid)" stopOpacity=".2" />
          </linearGradient>
          <radialGradient
            id="ridge-summit"
            cx="1040"
            cy={y(80)}
            r="380"
            gradientUnits="userSpaceOnUse"
          >
            <stop offset="0" stopColor="var(--amber)" stopOpacity=".5" />
            <stop offset=".45" stopColor="var(--rose)" stopOpacity=".14" />
            <stop offset="1" stopColor="var(--rose)" stopOpacity="0" />
          </radialGradient>
        </defs>
        <circle
          className="summit-glow"
          cx="1040"
          cy={y(80)}
          r="380"
          fill="url(#ridge-summit)"
        />
        <path
          d="M0 560 L0 520 L300 500 L560 430 L760 360 L980 300 L1180 330 L1440 280 L1440 560 Z"
          fill="var(--ridge-back)"
        />
        <line
          x1="600"
          x2="1440"
          y1={y(60)}
          y2={y(60)}
          className="horizon slow"
          vectorEffect="non-scaling-stroke"
        />
        <line
          x1="640"
          x2="1440"
          y1={y(80)}
          y2={y(80)}
          className="horizon fast"
          vectorEffect="non-scaling-stroke"
        />
        <path
          d={`M0 ${H} L${ridgeLine} L1440 ${H} Z`}
          fill="url(#ridge-rock)"
        />
        <path
          d={`M${ridgeLine}`}
          fill="none"
          stroke="url(#ridge-edge)"
          strokeWidth="2"
          vectorEffect="non-scaling-stroke"
        />
      </svg>
      {ridge.slice(2, 5).map(([x, py]) => (
        <span
          key={x}
          className="ridge-point"
          style={{ left: pct(x, 1440), top: pct(py, H) }}
        />
      ))}
      <span className="ridge-tag keep hot" style={{ top: pct(y(80), H) }}>
        fast path: 80% in one round
      </span>
      <span className="ridge-tag right" style={{ top: pct(y(60), H) }}>
        slow path: 60% in each of two rounds
      </span>
      <span
        className="ridge-tag"
        style={{ left: pct(640, 1440), top: pct(y(46), H) }}
      >
        rank 0 · 40
      </span>
      <span
        className="ridge-tag"
        style={{ left: pct(820, 1440), top: pct(y(71), H) }}
      >
        + rank 1 · 65
      </span>
      <span
        className="ridge-tag hot"
        style={{ left: pct(985, 1440), top: pct(y(88), H) }}
      >
        + rank 3 · 80
      </span>
    </div>
  );
}

export function Landing() {
  return (
    <div className="landing">
      <a className="skip-link" href="#landing-main">
        Skip to content
      </a>
      <header className="landing-header">
        <a href="#/" aria-label="Monocle home" className="landing-brand">
          <Brand />
        </a>
        <nav aria-label="Main navigation">
          <a href="#outcomes">What it checks</a>
          <a
            href={`${source}/tree/main/docs`}
            target="_blank"
            rel="noopener noreferrer"
          >
            Documentation <Arrow diagonal />
          </a>
          <a href={source} target="_blank" rel="noopener noreferrer">
            GitHub <Arrow diagonal />
          </a>
        </nav>
        <a className="landing-button ghost small" href="#/inspect">
          Open inspector
        </a>
      </header>
      <main id="landing-main" tabIndex={-1}>
        <section className="landing-hero" aria-labelledby="hero-title">
          <div className="hero-copy">
            <h1 id="hero-title">See finality reach the summit.</h1>
            <p className="hero-description">
              Monocle verifies Alpenglow certificates in your browser, covering{" "}
              <strong>
                BLS signatures, key proofs, exact signer stake and the fast or
                slow path
              </strong>
              , against a validator checkpoint you choose. When evidence falls
              short, it names the check that stopped it.
            </p>
            <div className="hero-actions">
              <a className="landing-button glow" href="#/inspect">
                Open the inspector
              </a>
              <a className="landing-button ghost" href="#outcomes">
                See what a run reports
              </a>
            </div>
          </div>
          <Ridge />
          <p className="ridge-caption">
            The ridge is fixture data. Ranks 0, 1 and 3 sign 40 + 25 + 15 of 100
            stake and reach the 80% fast-path line exactly. Synthetic keys,
            invented stake.
          </p>
        </section>

        <section
          className="outcomes"
          id="outcomes"
          aria-labelledby="outcomes-title"
        >
          <div className="section-head">
            <h2 id="outcomes-title">Every run ends with a reason.</h2>
            <p>
              These results come from Monocle’s signed synthetic fixtures. Each
              conclusion carries a stable diagnostic code, and a rejection never
              claims that a block is non-final.
            </p>
          </div>
          <ul className="outcome-list">
            {outcomes.map((o) => (
              <li key={o.name}>
                <span className="outcome-name">{o.name}</span>
                <span className="outcome-verdict" data-status={o.conclusion}>
                  <span aria-hidden="true">{glyph[o.conclusion]}</span>{" "}
                  {o.conclusion}
                </span>
                <span className="outcome-why">
                  <code>{o.code}</code> {o.why}
                </span>
              </li>
            ))}
          </ul>
        </section>

        <section className="capabilities" aria-label="Capabilities">
          <article>
            <h3>Verify on your machine</h3>
            <p>
              The Rust verifier runs as WebAssembly in a browser worker. Once
              the inspector has loaded, evidence never leaves the page and no
              network is needed.
            </p>
            <span>
              Checkpoint authenticity stays your assumption, recorded beside the
              result.
            </span>
          </article>
          <article>
            <h3>Change one byte</h3>
            <p>
              Derive an experiment from the immutable original, verify both
              again under the same checkpoint, and see the first check that
              breaks.
            </p>
            <span>
              One equal-length replacement per experiment, never stacked.
            </span>
          </article>
          <article>
            <h3>Hand someone the replay</h3>
            <p>
              Export the original bytes, the change and their digests. On import
              every verdict is recomputed; saved reports grant nothing.
            </p>
            <span>The recipient selects trust again.</span>
          </article>
        </section>

        <section className="ledger" aria-labelledby="ledger-title">
          <h2 id="ledger-title">
            What a result establishes, and what stays with you.
          </h2>
          <div className="ledger-columns">
            <section aria-labelledby="ledger-established">
              <h3 id="ledger-established">Monocle establishes</h3>
              <ul>
                <li>
                  The aggregate signature verifies for the exact signed bytes
                  <small>
                    BLS12-381, with each selected key’s proof of possession
                    checked
                  </small>
                </li>
                <li>
                  The signers carry enough stake
                  <small>
                    Exact integers against the checkpoint’s original denominator
                  </small>
                </li>
                <li>
                  Which finality path holds, if any
                  <small>
                    Fast, slow and genesis scopes are reported separately
                  </small>
                </li>
                <li>
                  What one changed byte breaks
                  <small>
                    A fresh comparison against the immutable original
                  </small>
                </li>
              </ul>
            </section>
            <section aria-labelledby="ledger-yours">
              <h3 id="ledger-yours">You remain responsible for</h3>
              <ul>
                <li>
                  Trusting the validator checkpoint
                  <small>
                    Monocle records your reason; it does not authenticate the
                    set
                  </small>
                </li>
                <li>
                  Context a raw RPC response lacks
                  <small>Recorded responses stay inconclusive without it</small>
                </li>
                <li>
                  Ancestry, execution and epoch transitions
                  <small>None of these are evaluated</small>
                </li>
                <li>
                  Reading a rejection correctly
                  <small>
                    Rejected evidence is not proof that a block is non-final
                  </small>
                </li>
              </ul>
            </section>
          </div>
        </section>

        <section className="landing-closing" aria-labelledby="closing-title">
          <h2 id="closing-title">Start with a signed example.</h2>
          <p>
            Six synthetic cases cover fast, slow and genesis evidence, a changed
            signature and too little stake.
          </p>
          <a className="landing-button glow" href="#/inspect">
            Open the inspector
          </a>
        </section>
      </main>
      <footer className="landing-footer">
        <a href="#/" aria-label="Monocle home" className="landing-brand">
          <Brand />
        </a>
        <p>
          An open-source finality debugger for Alpenglow. Not a light client.
        </p>
        <a href={source} target="_blank" rel="noopener noreferrer">
          View source <Arrow diagonal />
        </a>
      </footer>
    </div>
  );
}
