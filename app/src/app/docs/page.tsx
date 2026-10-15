import Link from "next/link";

export default function DocsOverview() {
  return (
    <>
      <p className="eyebrow">[ DOCS / 00 ]</p>
      <h1>
        Understand the
        <br />
        <em>record.</em>
      </h1>
      <p className="docs-lede">
        Slotbook is a disputable record-date holder register for permissioned SPL tokens. This guide
        follows the state machine from issuer setup to holder claim.
      </p>
      <div className="docs-callout">
        <span>READ FIRST</span>
        <p>
          The issuer owns the mint and funds the vault. Slotbook verifies the record at a chosen
          slot, then makes the resulting state claimable.
        </p>
      </div>
      <section className="doc-section" id="lifecycle">
        <p className="eyebrow">[ 01 ] / LIFECYCLE</p>
        <h2>Four moments in a distribution</h2>
        <div className="doc-list">
          <Link href="/docs/workflow/declare">
            <b>01 / DECLARE</b>
            <span>Set the total, future record slot, and salt commitment.</span>
          </Link>
          <Link href="/docs/workflow/commit">
            <b>02 / COMMIT</b>
            <span>An attestor replays history and commits a Merkle root.</span>
          </Link>
          <Link href="/docs/workflow/challenge">
            <b>03 / CHALLENGE</b>
            <span>Review and dispute an incorrect holder leaf during the window.</span>
          </Link>
          <Link href="/docs/workflow/claim">
            <b>04 / FINALIZE + CLAIM</b>
            <span>Close the window and let valid holders claim USDC.</span>
          </Link>
        </div>
      </section>
      <section className="doc-section" id="concepts">
        <p className="eyebrow">[ 02 ] / CORE CONCEPTS</p>
        <h2>Three owners. One record.</h2>
        <div className="concept-grid">
          <div>
            <span>ISSUER</span>
            <p>Controls the token mint and funds the distribution.</p>
          </div>
          <div>
            <span>ATTESTOR</span>
            <p>Replays known history and commits the holder root.</p>
          </div>
          <div>
            <span>SLOTBOOK</span>
            <p>Stores lifecycle state and releases finalized claims.</p>
          </div>
        </div>
      </section>
      <Link className="text-link" href="/docs/program">
        Browse generated program methods →
      </Link>
    </>
  );
}
