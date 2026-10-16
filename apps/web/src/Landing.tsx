import { useState } from "react";
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

/*
 * Ridge scenarios restate recorded results of Monocle's signed synthetic
 * fixtures: validator stakes 40, 25, 18, 15 and 2 of 100. Each ridge climbs
 * through the signers' cumulative stake toward its path threshold.
 */
type Scenario = {
  id: string;
  label: string;
  signers: { rank: number; total: number }[];
  threshold: number;
  thresholdName: string;
  conclusion: "VERIFIED" | "REJECTED";
  code: string;
  caption: string;
};
const scenarios: Scenario[] = [
  {
    id: "fast",
    label: "Fast path",
    signers: [
      { rank: 0, total: 40 },
      { rank: 1, total: 65 },
      { rank: 3, total: 80 },
    ],
    threshold: 80,
    thresholdName: "fast path: 80% in one round",
    conclusion: "VERIFIED",
    code: "FAST_FINALITY_ESTABLISHED",
    caption:
      "The ridge is fixture data. Ranks 0, 1 and 3 sign 40 + 25 + 15 of 100 stake and reach the 80% fast-path line exactly. Synthetic keys, invented stake.",
  },
  {
    id: "short",
    label: "Too little stake",
    signers: [
      { rank: 0, total: 40 },
      { rank: 1, total: 65 },
    ],
    threshold: 80,
    thresholdName: "fast path: 80% in one round",
    conclusion: "REJECTED",
    code: "INSUFFICIENT_STAKE",
    caption:
      "A genuine signature from ranks 0 and 1 carries 65 of 100 stake. It stops below the 80% line, so the fast-finalization claim is rejected. Synthetic keys, invented stake.",
  },
  {
    id: "slow",
    label: "Slow path",
    signers: [
      { rank: 0, total: 40 },
      { rank: 1, total: 65 },
    ],
    threshold: 60,
    thresholdName: "slow path: 60% in each of two rounds",
    conclusion: "VERIFIED",
    code: "SLOW_FINALITY_ESTABLISHED",
    caption:
      "Notarization and finalization each carry 65 of 100 stake, clearing 60% in both rounds. Their stakes are never added together. Synthetic keys, invented stake.",
  },
  {
    id: "genesis",
    label: "Genesis anchor",
    signers: [
      { rank: 0, total: 40 },
      { rank: 1, total: 65 },
      { rank: 3, total: 80 },
      { rank: 4, total: 82 },
    ],
    threshold: 82,
    thresholdName: "genesis anchor: 82%",
    conclusion: "VERIFIED",
    code: "GENESIS_ANCHOR_ESTABLISHED",
    caption:
      "Ranks 0, 1, 3 and 4 sign 82 of 100 stake for the migration anchor. It is reported separately from block finality. Synthetic keys, invented stake.",
  },
];

/* Geometry: x in a 1440-wide box, stake percentage mapped to height. */
const H = 560;
const y = (stake: number) => 540 - stake * 4.6;
const CLIMB_START = 640;
const SUMMIT_X = 1040;
const pct = (value: number, of: number) => `${(value / of) * 100}%`;
const signerX = (s: Scenario, i: number) =>
  s.signers.length === 1
    ? SUMMIT_X
    : CLIMB_START + (i * (SUMMIT_X - CLIMB_START)) / (s.signers.length - 1);
/*
 * Every ridge has the same point count so the browser can morph between them:
 * the climb passes exactly through each signer, and short climbs repeat the
 * summit point.
 */
function ridgePoints(s: Scenario): [number, number][] {
  const climb = s.signers.map((p, i): [number, number] => [
    signerX(s, i),
    y(p.total),
  ]);
  while (climb.length < 4) climb.push(climb[climb.length - 1]);
  const peak = s.signers[s.signers.length - 1].total;
  return [
    [0, y(2)],
    [420, y(6)],
    ...climb,
    [1130, y(peak - 10)],
    [1260, y(peak - 30)],
    [1360, y(Math.max(peak - 46, 8))],
    [1440, y(Math.max(peak - 52, 6))],
  ];
}
const line = (pts: [number, number][]) =>
  pts.map((p) => p.join(" ")).join(" L");

function StakeRidge() {
  const [active, setActive] = useState(scenarios[0]);
  const [touched, setTouched] = useState(false);
  const pts = ridgePoints(active);
  const peak = active.signers[active.signers.length - 1].total;
  const reached = peak >= active.threshold;
  const fill = `M0 ${H} L${line(pts)} L1440 ${H} Z`;
  const edge = `M${line(pts)}`;
  return (
    <>
      <div
        className="ridge"
        data-reached={reached}
        data-touched={touched || undefined}
        aria-hidden="true"
      >
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
              cx={SUMMIT_X}
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
            cx={SUMMIT_X}
            cy={y(80)}
            r="380"
            fill="url(#ridge-summit)"
          />
          <path
            d="M0 560 L0 520 L300 500 L560 430 L760 360 L980 300 L1180 330 L1440 280 L1440 560 Z"
            fill="var(--ridge-back)"
          />
          {[60, 80, 82].map((t) => (
            <line
              key={t}
              x1="600"
              x2="1440"
              y1={y(t)}
              y2={y(t)}
              className="horizon"
              data-active={t === active.threshold}
              vectorEffect="non-scaling-stroke"
            />
          ))}
          <path
            className="ridge-fill"
            d={fill}
            style={{ d: `path("${fill}")` }}
            fill="url(#ridge-rock)"
          />
          <path
            className="ridge-edge"
            d={edge}
            style={{ d: `path("${edge}")` }}
            pathLength={1}
            fill="none"
            stroke="url(#ridge-edge)"
            strokeWidth="2"
            vectorEffect="non-scaling-stroke"
          />
        </svg>
        {active.signers.map((p, i) => (
          <span
            key={`${active.id}-${p.rank}`}
            className={`ridge-point${i === active.signers.length - 1 ? " summit" : ""}`}
            style={{
              left: pct(signerX(active, i), 1440),
              top: pct(y(p.total), H),
              animationDelay: `${(touched ? 0.1 : 0.35) + i * (touched ? 0.12 : 0.32)}s`,
            }}
          >
            <span className="ridge-tag point-tag">
              {i === 0 ? "" : "+ "}rank {p.rank} · {p.total}
            </span>
          </span>
        ))}
        <span
          className="ridge-tag keep threshold-tag"
          style={{ top: pct(y(active.threshold), H) }}
        >
          {active.thresholdName}
        </span>
        <span
          key={active.id}
          className="ridge-result"
          data-status={active.conclusion}
          style={{
            left: pct(SUMMIT_X, 1440),
            top: pct(y(peak), H),
          }}
        >
          {active.conclusion === "VERIFIED" ? "✓" : "×"} {active.code}
        </span>
      </div>
      <div className="ridge-controls">
        <div
          className="ridge-switch"
          role="group"
          aria-label="Fixture certificate shown on the ridge"
        >
          {scenarios.map((s) => (
            <button
              key={s.id}
              type="button"
              aria-pressed={s.id === active.id}
              onClick={() => {
                setTouched(true);
                setActive(s);
              }}
            >
              {s.label}
            </button>
          ))}
        </div>
        <p className="ridge-caption" aria-live="polite">
          {active.caption}
        </p>
      </div>
    </>
  );
}

/*
 * Recorded result of the signature-bit experiment on the fast fixture: byte 42
 * of certificate "fast" changes from 01 to 00. Nothing is verified here; the
 * inspector reproduces this with the real verifier.
 */
const certBytes =
  "42 42 42 42 42 42 42 42 01 81 84 e8 3b 37 7a 43 21 ae 29 e4 b9 65".split(
    " ",
  );
const FIRST_BYTE = 34;
const demoChecks = [
  {
    stage: "Certificate decoding",
    before: ["PASS", "OK"],
    after: ["PASS", "OK"],
  },
  { stage: "Signers", before: ["PASS", "OK"], after: ["PASS", "OK"] },
  {
    stage: "Key qualification",
    before: ["PASS", "OK"],
    after: ["PASS", "OK"],
  },
  {
    stage: "Signature",
    before: ["PASS", "OK"],
    after: ["FAIL", "INVALID_SIGNATURE"],
  },
  {
    stage: "Quorum",
    before: ["PASS", "OK"],
    after: ["NOT_CHECKED", "PREREQUISITE_MISSING"],
  },
  {
    stage: "Finality",
    before: ["PASS", "FAST_FINALITY_ESTABLISHED"],
    after: ["NOT_CHECKED", "CERTIFICATE_CHECKS_FAILED"],
  },
] as const;
const statusWord = { PASS: "pass", FAIL: "fail", NOT_CHECKED: "not run" };
const statusGlyph = { PASS: "✓", FAIL: "×", NOT_CHECKED: "—" };

function TamperDemo() {
  const [flipped, setFlipped] = useState(false);
  return (
    <section className="tamper-demo" aria-labelledby="tamper-title">
      <div className="tamper-copy">
        <h2 id="tamper-title">Break it on purpose.</h2>
        <p>
          A finality claim is only as strong as its exact bytes. Flip one byte
          of a signed certificate and watch which check gives way first, and
          which checks never get to run.
        </p>
        <p className="tamper-note">
          This replays the recorded result of Monocle’s signature-bit experiment
          on the synthetic fast-finality fixture. Nothing is verified on this
          page; the inspector reproduces it with the real verifier.
        </p>
      </div>
      <div className="tamper-stage" data-flipped={flipped}>
        <div className="byte-row" aria-label="Certificate fast, bytes 34 to 55">
          {certBytes.map((b, i) =>
            FIRST_BYTE + i === 42 ? (
              <button
                key={i}
                type="button"
                className="byte target"
                aria-pressed={flipped}
                aria-label={`Byte 42: ${flipped ? "00, changed from 01" : "01"}. ${flipped ? "Restore" : "Flip"} it`}
                onClick={() => setFlipped(!flipped)}
              >
                <span className="byte-old" aria-hidden="true">
                  01
                </span>
                <span className="byte-new" aria-hidden="true">
                  00
                </span>
              </button>
            ) : (
              <span key={i} className="byte" aria-hidden="true">
                {b}
              </span>
            ),
          )}
        </div>
        <p className="byte-hint">
          {flipped
            ? "Byte 42 now reads 00. Select it again to restore the original."
            : "Select the highlighted byte 42 to flip it from 01 to 00."}
        </p>
        <ol className="demo-trace" aria-live="polite">
          {demoChecks.map((c, i) => {
            const [status, code] = flipped ? c.after : c.before;
            const changed = c.before[0] !== c.after[0];
            return (
              <li
                key={c.stage}
                data-status={status}
                data-changed={flipped && changed}
                style={{ transitionDelay: `${changed ? i * 0.12 : 0}s` }}
              >
                <span className="demo-node" aria-hidden="true">
                  {statusGlyph[status]}
                </span>
                <span className="demo-stage">{c.stage}</span>
                <span className="demo-status">{statusWord[status]}</span>
                <code>{code}</code>
              </li>
            );
          })}
        </ol>
        <p className="demo-verdict" data-flipped={flipped}>
          {flipped
            ? "× REJECTED. The changed evidence fails; this does not prove the block is non-final."
            : "✓ VERIFIED. Fast finality under the selected synthetic checkpoint."}
        </p>
      </div>
    </section>
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
          <StakeRidge />
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

        <TamperDemo />

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
