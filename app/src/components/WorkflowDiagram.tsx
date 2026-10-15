type DiagramKind = "declare" | "commit" | "challenge" | "claim";

const node = (label: string, detail: string, className = "") => (
  <div className={`workflow-node ${className}`}>
    <strong>{label}</strong>
    <span>{detail}</span>
  </div>
);

export function WorkflowDiagram({ kind }: { kind: DiagramKind }) {
  if (kind === "declare")
    return (
      <div className="workflow-diagram declare-diagram">
        <div className="diagram-kicker">ISSUER CONTROL PLANE / REQ01–04</div>
        <div className="diagram-row">
          {node("ISSUER", "mint authority", "accent")}
          <i />
          {node("ISSUER CONFIG", "PDA / policy", "wide")}
          <i />
          {node("DISTRIBUTION", "future record slot", "wide")}
          <i />
          {node("PAYMENT VAULT", "Distribution PDA", "accent")}
        </div>
        <div className="diagram-foot">
          Token ACL keeps the mint permissioned. Slotbook records the issuer&apos;s configuration
          and locks the payment pot for the declared distribution.
        </div>
      </div>
    );
  if (kind === "commit")
    return (
      <div className="workflow-diagram commit-diagram">
        <div className="diagram-kicker">OFF-CHAIN RECONSTRUCTION → ON-CHAIN COMMITMENT / REQ05</div>
        <div className="diagram-stack">
          <div className="archive-column">
            {node("RPC ARCHIVE", "ledger history")}
            {node("RECORD SLOT", "S + finality margin")}
          </div>
          <div className="diagram-arrow">→</div>
          {node("ATTESTOR BINARY", "replay + salted leaves", "large accent")}
          {node("MERKLE ROOT", "manifest / total", "root-node")}
        </div>
        <div className="diagram-branch">
          <span>matching quorum</span>
          <b>COMMITTED</b>
          <span>different root before quorum</span>
          <b className="danger">DISPUTED / CONFLICT</b>
        </div>
      </div>
    );
  if (kind === "challenge")
    return (
      <div className="workflow-diagram challenge-diagram">
        <div className="diagram-kicker">DISPUTE PLANE / REQ06–07</div>
        <div className="challenge-flow">
          {node("CHALLENGER", "owner + claimed balance", "accent")}
          <i />
          <div className="challenge-core">
            {node("CHALLENGE PDA", "bond locked")}
            {node("RESOLVER", "recompute under spec", "accent")}
          </div>
          <i />
          <div className="ruling-list">
            <b>UPHELD</b>
            <b>REJECTED</b>
            <b>CONFLICT</b>
            <b>EXPIRED</b>
          </div>
        </div>
        <div className="diagram-foot">
          Bonds leave the IssuerConfig vault only through the Slotbook program. The resolver is
          never the issuer or an attestor.
        </div>
      </div>
    );
  return (
    <div className="workflow-diagram claim-diagram">
      <div className="diagram-kicker">CLAIM PLANE / REQ08–11</div>
      <div className="claim-flow">
        {node("PROOFS API", "balance + salt + branch")}
        {node("HOLDER", "signs claim", "accent")}
        {node("VERIFY", "root + expiry + receipt", "root-node")}
      </div>
      <div className="claim-outcomes">
        <div>{node("PAID", "vault → payment account", "success")}</div>
        <div>{node("PENDING", "frozen / no ImmutableOwner", "warning")}</div>
      </div>
      <div className="diagram-foot">
        A pending claim is not forfeited. After the gate thaws the holder, a cranker releases it;
        after the hold window, the issuer may sweep it.
      </div>
    </div>
  );
}
