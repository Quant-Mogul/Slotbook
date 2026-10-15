import fs from "node:fs";
import path from "node:path";

const appRoot = path.resolve(import.meta.dirname, "..");
const repoRoot = path.resolve(appRoot, "..");
const idlPath = path.join(repoRoot, "target", "idl", "slotbook.json");
const rustPath = path.join(repoRoot, "programs", "slotbook", "src", "lib.rs");
const outputPath = path.join(appRoot, "src", "generated", "program-docs.json");

const descriptions = {
  init_issuer: "Register an issuer and verify that the issuer controls the token mint.",
  update_issuer: "Update the issuer configuration for an existing registry.",
  register_attestor: "Authorize an attestor backend to replay history and commit roots.",
  withdraw_bond: "Withdraw an attestor bond after its release conditions are met.",
  declare_distribution:
    "Create a distribution with a total, future record slot, and salt commitment.",
  commit_root: "Commit the attestor's Merkle root for balances reconstructed at the record slot.",
  challenge: "Open a challenge against a holder leaf during the challenge window.",
  resolve_challenge: "Resolve an open challenge with the protocol's ruling.",
  finalize: "Finalize a distribution after the challenge window closes.",
  close_distribution: "Close a completed distribution and reclaim its remaining state rent.",
  release_attestor: "Release an attestor bond after the distribution lifecycle completes.",
  claim: "Claim a finalized holder balance using its Merkle proof.",
  release_pending: "Release a pending frozen-holder claim after the holder is thawed.",
  sweep_pending: "Sweep unresolved pending funds according to the distribution rules.",
};

function typeName(type) {
  if (typeof type === "string") return type.replace(/^.*::/, "").replace(/[<\[\]].*$/, "");
  if (type?.defined) return String(type.defined).replace(/^.*::/, "");
  return null;
}

function rustDefinitions() {
  const statePath = path.join(repoRoot, "programs", "slotbook", "src", "state.rs");
  const source = fs.readFileSync(statePath, "utf8");
  const definitions = {};
  for (const match of source.matchAll(/pub struct (\w+)\s*\{([\s\S]*?)\n\}/g)) {
    const fields = [...match[2].matchAll(/pub (\w+):\s*([^,\n]+)/g)].map(([, name, type]) => ({
      name,
      type: type.trim(),
    }));
    definitions[match[1]] = { kind: "struct", fields };
  }
  for (const match of source.matchAll(/pub enum (\w+)\s*\{([\s\S]*?)\n\}/g)) {
    const variants = [...match[2].matchAll(/^\s*(\w+)(?:\s*\{([^}]*)\})?/gm)].map(
      ([, name, fields]) => ({
        name,
        fields: fields
          ? [...fields.matchAll(/(\w+):\s*([^,]+)/g)].map(([, field, type]) => ({
              name: field,
              type: type.trim(),
            }))
          : [],
      }),
    );
    definitions[match[1]] = { kind: "enum", variants };
  }
  return definitions;
}

function idlDefinitions(idl) {
  return Object.fromEntries(
    (idl.types ?? []).map((item) => {
      const definition = item.type ?? {};
      if (definition.kind === "struct")
        return [
          item.name,
          {
            kind: "struct",
            fields: (definition.fields ?? []).map((field) => ({
              name: field.name,
              type: JSON.stringify(field.type),
            })),
          },
        ];
      if (definition.kind === "enum")
        return [
          item.name,
          {
            kind: "enum",
            variants: (definition.variants ?? []).map((variant) => ({
              name: variant.name,
              fields: (variant.fields ?? []).map((field) => ({
                name: field.name,
                type: JSON.stringify(field.type),
              })),
            })),
          },
        ];
      return [item.name, { kind: definition.kind ?? "type", fields: [] }];
    }),
  );
}

function rustDocs() {
  const source = fs.readFileSync(rustPath, "utf8");
  const programModule = source.match(/pub mod slotbook \{([\s\S]*?)\n\}/)?.[1] ?? "";
  const instructions = [];
  const fnPattern = /pub fn (\w+)(?:<[^>]+>)?\s*\(([^)]*)\)(?:\s*->\s*[^\{]+)?\s*\{/g;
  for (const match of programModule.matchAll(fnPattern)) {
    const [, name, params] = match;
    const context = params.match(/ctx:\s*Context<([^>]+)>/)?.[1] ?? "Context";
    const args = [...params.matchAll(/,\s*(\w+):\s*([^,\n]+)/g)].map(([, argName, type]) => ({
      name: argName,
      type: type.trim(),
    }));
    instructions.push({
      name,
      signature: `${name}(${args.map((arg) => `${arg.name}: ${arg.type}`).join(", ")})`,
      description: descriptions[name] ?? "Execute a Slotbook protocol instruction.",
      args,
      context,
    });
  }
  return instructions;
}

function idlDocs(idl) {
  return idl.instructions.map((instruction) => ({
    name: instruction.name,
    signature: `${instruction.name}(${(instruction.args ?? []).map((arg) => `${arg.name}: ${JSON.stringify(arg.type)}`).join(", ")})`,
    description:
      instruction.docs?.join(" ") ??
      descriptions[instruction.name] ??
      "Execute a Slotbook protocol instruction.",
    args: (instruction.args ?? []).map((arg) => ({
      name: arg.name,
      type: JSON.stringify(arg.type),
    })),
    context: `${instruction.name[0].toUpperCase()}${instruction.name.slice(1)}Context`,
  }));
}

const fromIdl = fs.existsSync(idlPath);
const source = fromIdl ? JSON.parse(fs.readFileSync(idlPath, "utf8")) : null;
const output = {
  generatedFrom: fromIdl
    ? "target/idl/slotbook.json"
    : "programs/slotbook/src/lib.rs (IDL bootstrap)",
  instructions: fromIdl ? idlDocs(source) : rustDocs(),
  definitions: fromIdl ? idlDefinitions(source) : rustDefinitions(),
};
fs.mkdirSync(path.dirname(outputPath), { recursive: true });
fs.writeFileSync(outputPath, `${JSON.stringify(output, null, 2)}\n`);
console.log(`Generated ${output.instructions.length} program methods from ${output.generatedFrom}`);
