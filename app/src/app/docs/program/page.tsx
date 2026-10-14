import { programDocs } from "@/lib/program-docs";

function definitionName(type: string) {
  try {
    const parsed = JSON.parse(type);
    return parsed.defined ?? null;
  } catch {
    return type.match(/^[A-Z]\w+/)?.[0] ?? null;
  }
}

function Definition({ type }: { type: string }) {
  const name = definitionName(type);
  const definition = name ? programDocs.definitions[name] : undefined;
  if (!definition)
    return (
      <pre className="param-definition">
        <code>{type}</code>
      </pre>
    );
  const lines =
    definition.kind === "enum"
      ? (definition.variants ?? []).map((variant) =>
          variant.fields.length
            ? `  ${variant.name} { ${variant.fields.map((field) => `${field.name}: ${field.type}`).join(", ")} }`
            : `  ${variant.name}`,
        )
      : (definition.fields ?? []).map((field) => `  ${field.name}: ${field.type}`);
  return (
    <pre className="param-definition">
      <code>{`${definition.kind} ${name} {\n${lines.join("\n")}\n}`}</code>
    </pre>
  );
}

function ParameterMeaning({ name, type }: { name: string; type: string }) {
  const definition = definitionName(type);
  const isObjectType = Boolean(definition && programDocs.definitions[definition]);
  if (!isObjectType) return <span>Value supplied by the caller.</span>;
  return (
    <details className="param-detail">
      <summary>View definition</summary>
      <Definition type={type} />
    </details>
  );
}

export default function ProgramDocs() {
  return (
    <>
      <p className="eyebrow">[ DOCS / 01 ] / PROGRAM API</p>
      <h1>
        Program
        <br />
        <em>methods.</em>
      </h1>
      <p className="docs-lede">
        These signatures are generated from the Rust program surface. Run{" "}
        <code>npm run docs:generate</code> after changing an instruction or rebuilding the Anchor
        IDL.
      </p>
      <div className="generated-note">
        SOURCE <strong>{programDocs.generatedFrom}</strong>
      </div>
      <div className="method-list">
        {programDocs.instructions.map((method, index) => (
          <section className="method" id={method.name} key={method.name}>
            <div className="method-heading">
              <span className="method-index">{String(index + 1).padStart(2, "0")}</span>
              <h2>{method.name}</h2>
            </div>
            <p>{method.description}</p>
            <pre>
              <code>{method.signature}</code>
            </pre>
            <div className="method-meta">
              <span>
                CONTEXT <b>{method.context}</b>
              </span>
            </div>
            {method.args.length > 0 && (
              <div className="param-table">
                <div className="param-header">
                  <span>PARAMETER</span>
                  <span>TYPE</span>
                  <span>MEANING</span>
                </div>
                {method.args.map((arg) => (
                  <div className="param-row" key={arg.name}>
                    <code>{arg.name}</code>
                    <code>{arg.type}</code>
                    <ParameterMeaning name={arg.name} type={arg.type} />
                  </div>
                ))}
              </div>
            )}
          </section>
        ))}
      </div>
    </>
  );
}
