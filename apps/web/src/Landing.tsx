import { useState } from "react";
import { Arrow, Brand, Mark } from "./Brand";
import "./landing.css";

const source = "https://github.com/mansi0xc/monocle";
const modes = {
  Verify: {
    title: "Every check, accounted for.",
    label: "Fast finality",
    verdict: "Verified",
    body: "Follow the path from original bytes to a scoped conclusion. Inspect the signature, signer weights, and assumptions behind the result.",
    rows: [
      ["Certificate encoding", "Pass"],
      ["BLS aggregate signature", "Pass"],
      ["Signer stake threshold", "Pass"],
      ["Finality composition", "Pass"],
    ],
    foot: "Under an explicitly selected synthetic checkpoint.",
  },
  Tamper: {
    title: "One byte. A different result.",
    label: "Changed signature",
    verdict: "Rejected",
    body: "Change a signed byte while preserving the original. Compare fresh reports and find the first check that no longer holds.",
    rows: [
      ["Certificate encoding", "Pass"],
      ["BLS aggregate signature", "Fail"],
      ["Dependent checks", "Not checked"],
      ["Supplied finality claim", "Rejected"],
    ],
    foot: "A rejected claim does not prove that a block is non-final.",
  },
  Replay: {
    title: "Same evidence. Fresh verification.",
    label: "Imported replay",
    verdict: "Trust required",
    body: "Take the exact evidence and experiment with you. Re-import, choose the checkpoint again, and recompute every conclusion.",
    rows: [
      ["Original evidence", "Preserved"],
      ["Mutation history", "Preserved"],
      ["Saved report", "Not trusted"],
      ["Checkpoint selection", "Required"],
    ],
    foot: "An imported report never becomes a trusted result.",
  },
} as const;
type Mode = keyof typeof modes;

function OpticalView() {
  return (
    <div className="optical-scene" aria-hidden="true">
      <svg viewBox="0 0 620 570" className="optical-drawing" fill="none">
        <defs>
          <radialGradient id="optic-surface" cx=".36" cy=".3" r=".8">
            <stop stopColor="#315f6a" />
            <stop offset="1" stopColor="#153e4a" />
          </radialGradient>
          <linearGradient
            id="optic-rim"
            x1="170"
            y1="70"
            x2="460"
            y2="460"
            gradientUnits="userSpaceOnUse"
          >
            <stop stopColor="#c0dedb" />
            <stop offset=".37" stopColor="#638f97" />
            <stop offset=".72" stopColor="#204d59" />
            <stop offset="1" stopColor="#dcece5" />
          </linearGradient>
          <linearGradient
            id="optic-beam"
            x1="70"
            y1="430"
            x2="520"
            y2="50"
            gradientUnits="userSpaceOnUse"
          >
            <stop stopColor="#f18d62" stopOpacity="0" />
            <stop offset=".54" stopColor="#f18d62" stopOpacity=".16" />
            <stop offset="1" stopColor="#f18d62" stopOpacity="0" />
          </linearGradient>
        </defs>
        <circle
          cx="332"
          cy="269"
          r="214"
          stroke="#8bb4ba"
          strokeOpacity=".14"
          strokeDasharray="2 9"
        />
        <circle
          cx="332"
          cy="269"
          r="245"
          stroke="#8bb4ba"
          strokeOpacity=".12"
        />
        <path d="M64 425 425 28l100 109L159 502Z" fill="url(#optic-beam)" />
        <g className="optic-lens">
          <ellipse
            cx="318"
            cy="296"
            rx="177"
            ry="188"
            transform="rotate(31 318 296)"
            fill="#0b2f3b"
          />
          <ellipse
            cx="328"
            cy="277"
            rx="177"
            ry="188"
            transform="rotate(31 328 277)"
            fill="url(#optic-surface)"
            stroke="url(#optic-rim)"
            strokeWidth="3"
          />
          <ellipse
            cx="328"
            cy="277"
            rx="164"
            ry="175"
            transform="rotate(31 328 277)"
            stroke="#96bcc2"
            strokeOpacity=".32"
          />
          <ellipse
            cx="328"
            cy="277"
            rx="138"
            ry="150"
            transform="rotate(31 328 277)"
            stroke="#96bcc2"
            strokeOpacity=".19"
          />
          <path
            d="m202 374 252-227M211 180l218 186"
            stroke="#a6c9ca"
            strokeOpacity=".16"
          />
          <g stroke="#aecbd0" strokeOpacity=".3">
            <path d="M245 153v12m22-23v12m24-18v12m25-14v12m25-9v12m25-5v12m23-1v12m23 4v12m20 9v12" />
            <path d="M202 230h12m-17 25h12m-16 25h12m-10 25h12m-6 25h12m1 25h12" />
          </g>
          <g className="optic-focus">
            <circle
              cx="328"
              cy="277"
              r="63"
              stroke="#f09572"
              strokeWidth="1.5"
            />
            <circle
              cx="328"
              cy="277"
              r="48"
              stroke="#f09572"
              strokeOpacity=".32"
            />
            <path
              d="M307 277h42m-21-21v42"
              stroke="#f8b798"
              strokeWidth="1.5"
            />
            <circle cx="328" cy="277" r="5" fill="#f09572" />
          </g>
        </g>
        <g stroke="#98c1c6" strokeOpacity=".65">
          <path d="M393 196h74l32-32h48" />
          <circle cx="393" cy="196" r="3" fill="#b8d6d6" />
          <path d="M266 361h-73l-45 45H65" />
          <circle cx="266" cy="361" r="3" fill="#b8d6d6" />
        </g>
        <g fill="#c0d6da" fontFamily="'Public Sans', sans-serif" fontSize="12">
          <text x="459" y="150">
            The signed bytes
          </text>
          <text x="65" y="428">
            The selected context
          </text>
        </g>
        <path
          d="M94 113h20m-10-10v20M505 427h20m-10-10v20"
          stroke="#779da6"
          strokeWidth="1"
        />
        <circle cx="536" cy="311" r="4" fill="#f09572" />
        <circle cx="167" cy="119" r="3" fill="#96c2c5" />
      </svg>
      <div className="optic-caption">
        <span className="optic-caption-dot" />A closer look at consensus.
      </div>
    </div>
  );
}

export function Landing() {
  const [mode, setMode] = useState<Mode>("Verify");
  const current = modes[mode];
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
          <a href="#workflow">How it works</a>
          <a
            href={`${source}/tree/main/docs`}
            target="_blank"
            rel="noopener noreferrer"
          >
            Documentation <Arrow diagonal />
          </a>
          <a
            href={source}
            target="_blank"
            rel="noopener noreferrer"
            className="source-link"
          >
            GitHub <Arrow diagonal />
          </a>
        </nav>
        <a className="landing-nav-cta" href="#/inspect">
          Open inspector <Arrow />
        </a>
      </header>
      <main id="landing-main" tabIndex={-1}>
        <section className="landing-hero" aria-labelledby="hero-title">
          <div className="hero-copy">
            <div className="hero-intro">
              <span />
              An open-source lens on Alpenglow
            </div>
            <h1 id="hero-title">
              <span>Finality,</span> <span>in full view.</span>
            </h1>
            <p className="hero-description">
              Look inside the evidence behind Solana’s consensus. Verify
              certificates, question assumptions, and understand every result.
            </p>
            <div className="hero-actions">
              <a className="landing-button" href="#/inspect">
                Open inspector <Arrow />
              </a>
              <a className="workflow-link" href="#workflow">
                Explore the workflow <span aria-hidden="true">↓</span>
              </a>
            </div>
            <p className="hero-note">
              Runs in your browser. No wallet required.
            </p>
          </div>
          <OpticalView />
        </section>
        <div className="landing-principles" aria-label="Product principles">
          <p>
            <span aria-hidden="true">⌘</span> Local by design
          </p>
          <p>
            <span aria-hidden="true">◎</span> Explicit trust
          </p>
          <p>
            <span aria-hidden="true">↗</span> Reproducible evidence
          </p>
          <a href={source} target="_blank" rel="noopener noreferrer">
            Built in the open <Arrow diagonal />
          </a>
        </div>
        <section
          className="workflow-section"
          id="workflow"
          aria-labelledby="workflow-title"
        >
          <div className="section-intro">
            <h2 id="workflow-title">
              A verdict is only
              <br />
              the beginning.
            </h2>
            <p>
              Get closer to the why. From a first inspection to a reproducible
              experiment, every step leaves a trace you can follow.
            </p>
          </div>
          <div className="workflow-layout">
            <div className="workflow-copy">
              <div
                className="workflow-modes"
                role="group"
                aria-label="Explore Monocle’s workflow"
              >
                {(Object.keys(modes) as Mode[]).map((item, index) => (
                  <button
                    key={item}
                    aria-pressed={mode === item}
                    onClick={() => setMode(item)}
                  >
                    <span className="mode-number" aria-hidden="true">
                      0{index + 1}
                    </span>
                    {item}
                    <Arrow />
                  </button>
                ))}
              </div>
              <div className="mode-copy" key={mode}>
                <h3>{current.title}</h3>
                <p>{current.body}</p>
              </div>
              <a className="text-link" href="#/inspect">
                Try it with a signed example <Arrow />
              </a>
            </div>
            <section
              className={`inspector-preview preview-${mode.toLowerCase()}`}
              aria-label="Illustrative inspector preview"
            >
              <div className="preview-toolbar">
                <span>
                  <Mark /> Evidence inspector
                </span>
                <span className="preview-local">Local session</span>
              </div>
              <div
                className="preview-main"
                aria-live="polite"
                aria-atomic="true"
              >
                <div className="preview-context">
                  <span>Synthetic example</span>
                  <span>Checkpoint · epoch 42</span>
                </div>
                <div className="preview-verdict">
                  <div>
                    <p>{current.label}</p>
                    <strong>{current.verdict}</strong>
                  </div>
                  <span className="preview-verdict-symbol" aria-hidden="true">
                    {mode === "Verify" ? "✓" : mode === "Tamper" ? "×" : "—"}
                  </span>
                </div>
                <div className="preview-checks">
                  {current.rows.map(([label, value]) => (
                    <div key={label}>
                      <span>{label}</span>
                      <span
                        className={
                          value === "Fail" || value === "Rejected"
                            ? "preview-fail"
                            : ""
                        }
                      >
                        <span aria-hidden="true">
                          {value === "Pass"
                            ? "✓ "
                            : value === "Fail"
                              ? "× "
                              : ""}
                        </span>
                        {value}
                      </span>
                    </div>
                  ))}
                </div>
                <p className="preview-footnote">{current.foot}</p>
              </div>
              <p className="preview-caption">
                Illustrative preview · synthetic evidence
              </p>
            </section>
          </div>
        </section>
        <section
          className="experiment-section"
          aria-labelledby="experiment-title"
        >
          <div className="experiment-visual" aria-hidden="true">
            <div className="byte-caption">
              <span>Original</span>
              <span>Experiment</span>
            </div>
            <div className="byte-line">
              <span>8a</span>
              <span>2f</span>
              <span className="byte-original">01</span>
              <span className="byte-connector">→</span>
              <span>8a</span>
              <span>2f</span>
              <span className="byte-mutated">00</span>
            </div>
            <div className="byte-underlines">
              <span>Original bytes preserved</span>
              <span>One controlled change</span>
            </div>
          </div>
          <div className="experiment-copy">
            <h2 id="experiment-title">
              Small change.
              <br />
              Clear consequence.
            </h2>
            <p>
              Good research asks what happens next. Change a signature byte,
              inspect the failure, and share an experiment someone else can
              reproduce.
            </p>
            <a href="#/inspect" className="text-link">
              Enter the tamper lab <Arrow />
            </a>
          </div>
        </section>
        <section className="trust-section" aria-labelledby="trust-title">
          <div className="trust-heading">
            <Mark />
            <h2 id="trust-title">
              Your evidence.
              <br />
              Your machine.
            </h2>
            <p>A focused research tool, with its assumptions in the open.</p>
          </div>
          <div className="trust-details">
            <article>
              <h3>Verification stays local.</h3>
              <p>
                The Rust verifier runs in your browser through WebAssembly.
                Evidence files are processed on your device.
              </p>
            </article>
            <article>
              <h3>You choose the trust basis.</h3>
              <p>
                A checkpoint is an explicit input. Monocle shows what a result
                establishes, and what still depends on external trust.
              </p>
            </article>
            <article>
              <h3>Missing evidence stays visible.</h3>
              <p>
                Synthetic examples are labeled. Recorded RPC evidence with
                missing context stays inconclusive. The limits stay beside the
                result.
              </p>
            </article>
          </div>
        </section>
        <section className="landing-closing" aria-labelledby="closing-title">
          <div>
            <p>Bring your curiosity. Start with an example.</p>
            <h2 id="closing-title">See what holds up.</h2>
          </div>
          <a className="landing-button light-button" href="#/inspect">
            Open inspector <Arrow />
          </a>
        </section>
      </main>
      <footer className="landing-footer">
        <a href="#/" aria-label="Monocle home" className="landing-brand">
          <Brand />
        </a>
        <p>A fixed-checkpoint research tool for Alpenglow.</p>
        <a href={source} target="_blank" rel="noopener noreferrer">
          View source <Arrow diagonal />
        </a>
      </footer>
    </div>
  );
}
