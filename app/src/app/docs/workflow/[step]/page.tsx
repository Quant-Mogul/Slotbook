import Link from "next/link";
import { notFound } from "next/navigation";
import { WorkflowDiagram } from "@/components/WorkflowDiagram";

const workflows = {
  declare: {
    number: "01",
    eyebrow: "ISSUER / SETUP",
    title: "Declare the distribution",
    summary:
      "The issuer establishes the trust boundary, registers the permissioned mint, and announces a record slot far enough in the future for holders to trade.",
    diagram: "declare" as const,
    actors: "Issuer, Token ACL, ABL gate",
    accounts: "IssuerConfig PDA, bond vault, Distribution PDA, payment vault",
    outcome: "A funded Distribution enters Declared with a unique id and future record_slot.",
    steps: [
      "Create the Token-2022 mint and configure the Token ACL allow list.",
      "Call init_issuer. The program checks the mint authority and Token ACL MintConfig freeze authority, then creates IssuerConfig and its bond vault.",
      "Register the attestor set and policy: quorum, resolver, notice slots, bonds, and claim windows.",
      "Call declare_distribution. The program checks the notice period, creates the Distribution and payment vault, and transfers the distribution total.",
    ],
  },
  commit: {
    number: "02",
    eyebrow: "ATTESTOR / REPLAY",
    title: "Replay history and commit the root",
    summary:
      "The holder list is reconstructed off-chain from the ledger. The program receives only the committed root, manifest hash, register total, and the attestor's bounded commitment.",
    diagram: "commit" as const,
    actors: "Attestor, RPC archive, auditors",
    accounts: "Attestor PDA, Commitment PDA, Distribution PDA",
    outcome:
      "Matching quorum opens the challenge window. A differing root before quorum moves the distribution to Disputed.",
    steps: [
      "Wait until record_slot plus finality_margin_slots. The attestor reads every relevant transfer through the end of the record slot.",
      "Replay balances under the published snapshot spec, build salted leaves, sort-pair hash the tree, and publish the manifest.",
      "Call commit_root with the root, manifest hash, register total, resolved slot, covered range, row count, spec version, and backend.",
      "The program checks attestor membership, bond coverage, duplicate commitments, and root consistency before storing the Commitment.",
    ],
  },
  challenge: {
    number: "03",
    eyebrow: "OPEN DISPUTE / RESOLVE",
    title: "Challenge the record",
    summary:
      "A challenge is a bounded dispute over one owner's balance. The resolver reruns the reconstruction under the same spec and the program moves bonds according to the ruling.",
    diagram: "challenge" as const,
    actors: "Challenger, resolver, cranker",
    accounts: "Challenge PDA, live Commitment PDAs, bond vault",
    outcome:
      "The dispute closes as Declared, Committed, or a still-valid quorum state, and the Challenge PDA is closed.",
    steps: [
      "During the challenge window, anyone deposits challenger_bond and submits an owner plus claimed_balance.",
      "The resolver recomputes the disputed owner from ledger history. The resolver may submit Upheld, Rejected, or Conflict; anyone may submit Expired after the timeout.",
      "The program requires the correct signer and every live Commitment and Attestor account in the transaction.",
      "Upheld slashes the challenged-root committers, Rejected resumes the window, Conflict slashes differing roots, and Expired returns bonds without slashing.",
    ],
  },
  claim: {
    number: "04",
    eyebrow: "FINALIZE / SETTLE",
    title: "Finalize and claim",
    summary:
      "After the challenge window, any cranker can open claims. A holder obtains a proof from the attestor's proofs API and receives payment when the Token-2022 account is payable.",
    diagram: "claim" as const,
    actors: "Cranker, holder of record, issuer",
    accounts: "Distribution PDA, ClaimReceipt PDA, Pending PDA, payment vault",
    outcome:
      "The holder is Paid immediately or recorded as Pending without losing the entitlement.",
    steps: [
      "Call finalize after window_end_slot. The Distribution moves from Committed to Open.",
      "The holder requests balance, salt, and a Merkle branch from the proofs API, then signs claim.",
      "The program checks Open state, expiry, one claim per owner, and the salted leaf against final_root. It computes floor(total × balance / register_total) with u128 intermediates.",
      "If the holder account is initialized, unfrozen, and has ImmutableOwner, the vault pays. Otherwise Pending records the amount until release_pending or sweep_pending applies.",
    ],
  },
} as const;

export function generateStaticParams() {
  return Object.keys(workflows).map((step) => ({ step }));
}

export default async function WorkflowPage({ params }: { params: Promise<{ step: string }> }) {
  const { step } = await params;
  const workflow = workflows[step as keyof typeof workflows];
  if (!workflow) notFound();
  return (
    <>
      <div className="workflow-breadcrumb">
        <Link href="/docs">Docs</Link>
        <span>/</span>
        <span>Workflow</span>
        <span>/</span>
        <strong>{workflow.eyebrow}</strong>
      </div>
      <div className="workflow-heading">
        <p className="eyebrow">
          [ WORKFLOW / {workflow.number} ] {workflow.eyebrow}
        </p>
        <h1>{workflow.title}</h1>
        <p className="docs-lede">{workflow.summary}</p>
      </div>
      <WorkflowDiagram kind={workflow.diagram} />
      <div className="workflow-grid">
        <section>
          <p className="eyebrow">[ EXECUTION ]</p>
          <h2>What happens</h2>
          <ol className="workflow-steps">
            {workflow.steps.map((item, index) => (
              <li key={item}>
                <span>{String(index + 1).padStart(2, "0")}</span>
                <p>{item}</p>
              </li>
            ))}
          </ol>
        </section>
        <aside className="workflow-facts">
          <div>
            <span>ACTORS</span>
            <strong>{workflow.actors}</strong>
          </div>
          <div>
            <span>ACCOUNTS</span>
            <strong>{workflow.accounts}</strong>
          </div>
          <div>
            <span>OUTCOME</span>
            <strong>{workflow.outcome}</strong>
          </div>
        </aside>
      </div>
      <nav className="workflow-nav">
        <Link href="/docs">← All docs</Link>
        <Link
          href={
            step === "declare"
              ? "/docs/workflow/commit"
              : step === "commit"
                ? "/docs/workflow/challenge"
                : step === "challenge"
                  ? "/docs/workflow/claim"
                  : "/docs/workflow/declare"
          }
        >
          {step === "claim" ? "Back to declaration →" : "Next workflow step →"}
        </Link>
      </nav>
    </>
  );
}
