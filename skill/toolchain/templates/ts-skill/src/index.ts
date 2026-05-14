/**
 * ts-hello — reference TypeScript skill entrypoint.
 *
 * Phase 5: bound via wit-bindgen to the cyberos-skill.wit `invocation.run`
 * export. For now this is a plain TS function the bundler can consume.
 */

interface Input {
  name?: string;
}

interface Output {
  greeting: string;
}

export function run(req: { input: string; capabilities: string[] }): {
  output: string;
  error: string | null;
} {
  let parsed: Input;
  try {
    parsed = JSON.parse(req.input);
  } catch (e) {
    return {
      output: "",
      error: `invalid input JSON: ${(e as Error).message}`,
    };
  }
  const name = parsed.name ?? "world";
  const result: Output = { greeting: `Hello, ${name}!` };
  return { output: JSON.stringify(result), error: null };
}

// Allow this to be invoked directly via Bun for smoke testing.
if (import.meta.main) {
  const out = run({
    input: JSON.stringify({ name: process.argv[2] ?? "CyberSkill" }),
    capabilities: [],
  });
  console.log(out);
}
