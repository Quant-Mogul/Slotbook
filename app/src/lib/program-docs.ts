import docs from "@/generated/program-docs.json";

export type ProgramDoc = {
  name: string;
  signature: string;
  description: string;
  args: { name: string; type: string }[];
  context: string;
};

export type TypeDefinition = {
  kind: string;
  fields?: { name: string; type: string }[];
  variants?: { name: string; fields: { name: string; type: string }[] }[];
};

export const programDocs = docs as {
  generatedFrom: string;
  instructions: ProgramDoc[];
  definitions: Record<string, TypeDefinition>;
};
