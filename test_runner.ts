// TODO: specific tests
// TODO: run in release / debug mode

function die(msg: string): never {
	console.error("died!", msg);
	Deno.exit(1);
}

const decoder = new TextDecoder();

const test_name = "operator_precedence";
const test_file = `tests/${test_name}.loki`;
const out_c_file = `target/tests/${test_name}.c`;
const out_exe_file = `target/tests/${test_name}.exe`;

Deno.mkdirSync("target/tests", { recursive: true });

const cmd = new Deno.Command("cargo", {
	args: [ "run", "-q" ],
	env: {
		"LOKI_RUNNING_TESTS": "yes",
		"LOKI_FILE": test_file,
		"LOKI_OUTPUT_FILE": out_c_file,
	},
});

const compiler_out = await cmd.output();

if (compiler_out.code != 0) {
	const stderr = decoder.decode(compiler_out.stderr).trim();
	console.log("Errors!", stderr);
	die("The compiler emitted errors / panicked");
}

const checks = decoder.decode(compiler_out.stdout).trim()
	.split('\n')
	.filter(l => l.startsWith("__t_"))
	.map(l => l.split('='));

if ((await new Deno.Command("clang", { args: [ out_c_file, "-o", out_exe_file ] }).output()).code != 0) {
	const stderr = decoder.decode(compiler_out.stderr).trim();
	console.log("Errors!", stderr);
	die("Clang emitted errors");
}

const out = await new Deno.Command(out_exe_file).output();

for (const [check, expected] of checks) {
	if (check == "__t_expected_status") {
		if (out.code != +expected)
			die(`Status code check failed: Expected ${+expected}, got ${out.code}`);
	} else {
		die("Unknown check " + check);
	}
}

console.log("Test succesful");
