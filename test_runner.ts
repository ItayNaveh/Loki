// TODO: specific tests
// TODO: run in release / debug mode

import * as path from "https://deno.land/std@0.200.0/path/mod.ts";

const decoder = new TextDecoder();
Deno.mkdirSync("target/tests", { recursive: true });
await run_cmd("cargo build -q", "Cargo errored");

async function run_cmd(cmd: string, err_msg: string) {
	const [cmd_, ...args] = cmd.split(" ");
	const out = await new Deno.Command(cmd_, { args }).output();
	if (out.code != 0) {
		console.error(err_msg, decoder.decode(out.stderr));
		die(err_msg);
	}
}

function die(msg: string): never {
	console.error("died!", msg);
	Deno.exit(1);
}

async function run_test(name: string) {
	name = path.parse(name).name;
	const test_file = `tests/${name}.loki`;
	// const out_c_file = `target/tests/${name}.c`;
	const out_exe_file = `target/tests/${name}.exe`;

	const compiler_out = await new Deno.Command("target/debug/loki.exe", {
		args: [ test_file, "-o", out_exe_file ],
		env: { "LOKI_RUNNING_TESTS": "yes" },
		stderr: "inherit",
	}).output();

	if (compiler_out.code != 0) {
		die(`[${name}] The compiler emitted errors / panicked`);
	}

	// await run_cmd(`clang ${out_c_file} -o ${out_exe_file}`, `[${name}] Clang errored`);

	const out = await new Deno.Command(out_exe_file).output();

	const checks = decoder.decode(compiler_out.stdout).trim()
		.split('\n')
		.filter(l => l.startsWith("__t_"))
		.map(l => l.split('='));

	for (const [check, expected] of checks) {
		if (check == "__t_expected_status") {
			if (out.code != +expected)
				die(`[${name}] Status code check failed: Expected ${+expected}, got ${out.code}`);
		} else {
			die(`[${name}] Unknown check: ${check}`);
		}
	}

	console.log(name, "Test succesful");
}

for await (const test of Deno.readDir("tests")) {
	if (!test.isFile) continue;
	run_test(test.name);
}
