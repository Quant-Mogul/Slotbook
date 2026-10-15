import { AngularButton } from "@/components/AngularButton";
import { LandingMotion } from "@/components/LandingMotion";
import { SiteHeader } from "@/components/SiteHeader";

const stages = [
  [
    "01",
    "DECLARE",
    "An issuer commits to a future record slot and distribution amount.",
    "/docs/workflow/declare",
  ],
  [
    "02",
    "REPLAY",
    "An attestor rebuilds balances from the complete transfer history.",
    "/docs/workflow/commit",
  ],
  [
    "03",
    "CHALLENGE",
    "Anyone can challenge a leaf while the root is still disputable.",
    "/docs/workflow/challenge",
  ],
  [
    "04",
    "CLAIM",
    "Finalized holders claim their share from the issuer-funded vault.",
    "/docs/workflow/claim",
  ],
];

export default function Home() {
  return (
    <main>
      <section className="hero">
        <div className="hero-lens hero-lens-a" aria-hidden="true" />
        <div className="hero-lens hero-lens-b" aria-hidden="true" />
        <div className="hero-lens hero-lens-c" aria-hidden="true" />
        <div className="hero-inner shell">
          <SiteHeader />
          <LandingMotion />
          <div className="hero-copy">
            <p className="eyebrow">SLOTBOOK / RECORD-DATE PROTOCOL</p>
            <h1>
              VERIFIABLE
              <br />
              <em>DISTRIBUTIONS</em>
              <br />
              AT EVERY SLOT.
            </h1>
            <p className="hero-lede">
              A holder register for permissioned tokens. Replay the history, commit the root,
              resolve the dispute, release the value.
            </p>
            <div className="hero-actions">
              <AngularButton href="/docs">Explore the docs</AngularButton>
              <AngularButton href="#flow" secondary>
                See the flow
              </AngularButton>
            </div>
          </div>
        </div>
      </section>
      <section className="ticker">
        <div>PERMISSIONED TOKEN REGISTRIES</div>
        <div>MERKLE ROOTS</div>
        <div>RECORD-DATE CLAIMS</div>
        <div>ON SOLANA</div>
      </section>
      <div className="tape-separator" aria-hidden="true">
        <span />
        <span />
        <span />
        <div className="pattern-dots tape-dots" />
      </div>
      <section className="section shell" id="protocol">
        <div className="section-intro">
          <p className="eyebrow">[ 001 ] / THE REGISTER</p>
          <h2>
            History becomes
            <br />
            <span>claimable state.</span>
          </h2>
          <p>
            Slotbook gives issuers a precise record date without asking the program to mint or own
            their token. The issuer controls the asset; Slotbook records who held it.
          </p>
        </div>
        <div className="metric-grid">
          <div className="metric">
            <strong>01</strong>
            <span>issuer-owned mint</span>
          </div>
          <div className="metric">
            <strong>∞</strong>
            <span>replayed transfers</span>
          </div>
          <div className="metric">
            <strong>01</strong>
            <span>disputable root</span>
          </div>
          <div className="metric">
            <strong>0</strong>
            <span>trusted shortcuts</span>
          </div>
        </div>
      </section>
      <section className="section flow-section shell" id="flow">
        <div className="pattern-dots flow-dots" aria-hidden="true" />
        <div className="section-kicker">
          <p className="eyebrow">[ 002 ] / THE LIFECYCLE</p>
          <p className="mono">DECLARE → RECORD → SETTLE</p>
        </div>
        <div className="stage-grid">
          {stages.map(([num, title, copy, href]) => (
            <a className="stage" href={href} key={num}>
              <span className="stage-num">{num}</span>
              <h3>{title}</h3>
              <p>{copy}</p>
              <span className="stage-arrow">↗</span>
            </a>
          ))}
        </div>
      </section>
      <section className="section shell data-section" id="read-state">
        <div className="data-card">
          <div className="card-top">
            <span>REGISTER STATUS</span>
            <span className="status">
              <i /> LIVE
            </span>
          </div>
          <div className="data-row main-row">
            <span>distribution_004</span>
            <strong>◎ 12,500.00 USDC</strong>
          </div>
          <div className="data-columns">
            <div>
              <small>RECORD SLOT</small>
              <strong>284,019,441</strong>
            </div>
            <div>
              <small>MERKLE ROOT</small>
              <strong>7F4A…C19D</strong>
            </div>
            <div>
              <small>CHALLENGE WINDOW</small>
              <strong>OPEN / 18H</strong>
            </div>
          </div>
          <div className="progress">
            <span />
          </div>
          <div className="card-footer">
            <span>ATTESTOR REPLAY COMPLETE</span>
            <span>08 / 08 CHECKS</span>
          </div>
        </div>
        <div className="data-copy">
          <p className="eyebrow">[ 003 ] / PROOF SURFACE</p>
          <h2>
            Read the state.
            <br />
            <span>Challenge the record.</span>
          </h2>
          <p>
            Every claim resolves against a committed Merkle root. Frozen holders remain pending
            until the issuer’s gate allows settlement.
          </p>
          <AngularButton href="/docs/program">View program methods</AngularButton>
        </div>
      </section>
      <section className="closing shell">
        <p className="eyebrow">SLOTBOOK / OPEN PROTOCOL SURFACE</p>
        <h2>
          Make the record
          <br />
          <em>worth trusting.</em>
        </h2>
        <AngularButton href="/docs">Start with the docs</AngularButton>
      </section>
      <footer className="footer shell">
        <div className="pattern-dots footer-dots" aria-hidden="true" />
        <div className="pattern-stripes footer-stripes" aria-hidden="true" />
        <span>© 2026 SLOTBOOK</span>
        <span>BUILT FOR DISPUTABLE DISTRIBUTIONS</span>
        <a href="https://github.com" rel="noreferrer">
          GITHUB ↗
        </a>
      </footer>
    </main>
  );
}
