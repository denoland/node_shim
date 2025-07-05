mod node_cli_parser;

use exec::execvp;
use node_cli_parser::ParseResult;
use std::env;
use std::process;

fn main() {
    let args = env::args().skip(1).collect::<Vec<String>>();

    let parsed_args = match node_cli_parser::parse_args(args) {
        Ok(parsed_args) => parsed_args,
        Err(e) => {
            if e.len() == 1 {
                eprintln!("Error: {}", e[0]);
            } else if e.len() > 1 {
                eprintln!("Errors: {}", e.join(", "));
            }
            process::exit(1);
        }
    };

    let deno_args = translate_to_deno(parsed_args);

    if std::env::var("NODE_SHIM_DEBUG").is_ok() {
        eprintln!("deno {:?}", deno_args);
        process::exit(0);
    }

    // Execute deno with the translated arguments
    let err = execvp("deno", &deno_args);
    eprintln!("Failed to execute deno: {}", err);
    process::exit(1);
}

fn translate_to_deno(parsed_args: ParseResult) -> Vec<String> {
    let mut deno_args = vec!["node".to_string()];

    if parsed_args.options.use_system_ca || parsed_args.options.use_openssl_ca {
        unsafe { std::env::set_var("DENO_TLS_CA_STORE", "system") };
    }

    if parsed_args.options.print_help {
        println!("This is a shim that translates Node CLI arguments to Deno CLI arguments.");
        println!("Use exactly like you would use Node.js, but it will run with Deno.");
        process::exit(0);
    }

    if parsed_args.options.print_version {
        deno_args.push("--version".to_string());
        return deno_args;
    }

    if parsed_args.options.print_v8_help {
        deno_args.push("run".to_string());
        deno_args.push("--v8-flags=--help".to_string());
        return deno_args;
    }

    // Handle --run (package.json scripts)
    if !parsed_args.options.run.is_empty() {
        deno_args.push("task".to_string());
        deno_args.push(parsed_args.options.run.clone());
        deno_args.extend(parsed_args.remaining_args);
        return deno_args;
    }

    // Handle -e/--eval or -p/--print
    if parsed_args.options.per_isolate.per_env.has_eval_string {
        deno_args.push("eval".to_string());
        deno_args.push("-A".to_string()); // Allow all permissions
        deno_args.push("--unstable-node-globals".to_string());
        deno_args.push("--unstable-bare-node-builtins".to_string());
        deno_args.push("--unstable-detect-cjs".to_string());
        deno_args.push("--node-modules-dir=manual".to_string());
        deno_args.push("--no-config".to_string());
        if parsed_args.options.per_isolate.per_env.has_env_file_string {
            if parsed_args.options.per_isolate.per_env.env_file.is_empty() {
                deno_args.push("--env-file".to_string());
            } else {
                deno_args.push(format!(
                    "--env-file={}",
                    parsed_args.options.per_isolate.per_env.env_file
                ));
            }
        }
        if parsed_args.options.per_isolate.per_env.print_eval {
            deno_args.push("--print".to_string());
        }
        if !parsed_args.v8_args.is_empty() {
            deno_args.push(format!("--v8-flags={}", parsed_args.v8_args.join(",")));
        }
        deno_args.push(parsed_args.options.per_isolate.per_env.eval_string);
        deno_args.push("--".to_string());
        deno_args.extend(parsed_args.remaining_args);
        return deno_args;
    }

    // Handle REPL (no arguments or only node options)
    if parsed_args.remaining_args.is_empty() || parsed_args.options.per_isolate.per_env.force_repl {
        deno_args.push("repl".to_string());
        deno_args.push("-A".to_string());
        if !parsed_args.v8_args.is_empty() {
            deno_args.push(format!("--v8-flags={}", parsed_args.v8_args.join(",")));
        }
        if !parsed_args
            .options
            .per_isolate
            .per_env
            .conditions
            .is_empty()
        {
            deno_args.push(format!(
                "--conditions={}",
                parsed_args.options.per_isolate.per_env.conditions.join(",")
            ));
        }
        if parsed_args
            .options
            .per_isolate
            .per_env
            .debug_options
            .inspector_enabled
        {
            let arg = if parsed_args
                .options
                .per_isolate
                .per_env
                .debug_options
                .break_first_line
            {
                "--inspect-brk"
            } else {
                "--inspect"
            };
            deno_args.push(format!(
                "{}={}:{}",
                arg,
                parsed_args
                    .options
                    .per_isolate
                    .per_env
                    .debug_options
                    .host_port
                    .host,
                parsed_args
                    .options
                    .per_isolate
                    .per_env
                    .debug_options
                    .host_port
                    .port
            ));
        }
        deno_args.push("--".to_string());
        deno_args.extend(parsed_args.remaining_args);
        return deno_args;
    }

    if parsed_args.options.per_isolate.per_env.test_runner {
        deno_args.push("test".to_string());
        deno_args.push("-A".to_string()); // Allow all permissions for test runner
        deno_args.push("--unstable-node-globals".to_string());
        deno_args.push("--unstable-bare-node-builtins".to_string());
        deno_args.push("--unstable-detect-cjs".to_string());
        deno_args.push("--node-modules-dir=manual".to_string());
        deno_args.push("--no-config".to_string());
        if parsed_args.options.per_isolate.per_env.watch_mode {
            if parsed_args
                .options
                .per_isolate
                .per_env
                .watch_mode_paths
                .is_empty()
            {
                deno_args.push("--watch".to_string());
            } else {
                deno_args.push(format!(
                    "--watch={}",
                    parsed_args
                        .options
                        .per_isolate
                        .per_env
                        .watch_mode_paths
                        .iter()
                        .map(|p| p.replace(",", ",,"))
                        .collect::<Vec<String>>()
                        .join(",")
                ));
            }
        }
        if parsed_args.options.per_isolate.per_env.has_env_file_string {
            if parsed_args.options.per_isolate.per_env.env_file.is_empty() {
                deno_args.push("--env-file".to_string());
            } else {
                deno_args.push(format!(
                    "--env-file={}",
                    parsed_args.options.per_isolate.per_env.env_file
                ));
            }
        }
        if !parsed_args.v8_args.is_empty() {
            deno_args.push(format!("--v8-flags={}", parsed_args.v8_args.join(",")));
        }
        if !parsed_args
            .options
            .per_isolate
            .per_env
            .conditions
            .is_empty()
        {
            deno_args.push(format!(
                "--conditions={}",
                parsed_args.options.per_isolate.per_env.conditions.join(",")
            ));
        }
        if parsed_args
            .options
            .per_isolate
            .per_env
            .debug_options
            .inspector_enabled
        {
            let arg = if parsed_args
                .options
                .per_isolate
                .per_env
                .debug_options
                .break_first_line
            {
                "--inspect-brk"
            } else {
                "--inspect"
            };
            deno_args.push(format!(
                "{}={}:{}",
                arg,
                parsed_args
                    .options
                    .per_isolate
                    .per_env
                    .debug_options
                    .host_port
                    .host,
                parsed_args
                    .options
                    .per_isolate
                    .per_env
                    .debug_options
                    .host_port
                    .port
            ));
        }
        deno_args.extend(parsed_args.remaining_args);
        return deno_args;
    }

    // Handle other cases, like running a script
    deno_args.push("run".to_string());
    deno_args.push("-A".to_string());
    deno_args.push("--unstable-node-globals".to_string());
    deno_args.push("--unstable-bare-node-builtins".to_string());
    deno_args.push("--unstable-detect-cjs".to_string());
    deno_args.push("--node-modules-dir=manual".to_string());
    deno_args.push("--no-config".to_string());
    if parsed_args.options.per_isolate.per_env.watch_mode {
        if parsed_args
            .options
            .per_isolate
            .per_env
            .watch_mode_paths
            .is_empty()
        {
            deno_args.push("--watch".to_string());
        } else {
            deno_args.push(format!(
                "--watch={}",
                parsed_args
                    .options
                    .per_isolate
                    .per_env
                    .watch_mode_paths
                    .iter()
                    .map(|p| p.replace(",", ",,"))
                    .collect::<Vec<String>>()
                    .join(",")
            ));
        }
    }
    if parsed_args.options.per_isolate.per_env.has_env_file_string {
        if parsed_args.options.per_isolate.per_env.env_file.is_empty() {
            deno_args.push("--env-file".to_string());
        } else {
            deno_args.push(format!(
                "--env-file={}",
                parsed_args.options.per_isolate.per_env.env_file
            ));
        }
    }
    if !parsed_args.v8_args.is_empty() {
        deno_args.push(format!("--v8-flags={}", parsed_args.v8_args.join(",")));
    }
    if !parsed_args
        .options
        .per_isolate
        .per_env
        .conditions
        .is_empty()
    {
        deno_args.push(format!(
            "--conditions={}",
            parsed_args.options.per_isolate.per_env.conditions.join(",")
        ));
    }
    if parsed_args
        .options
        .per_isolate
        .per_env
        .debug_options
        .inspector_enabled
    {
        let arg = if parsed_args
            .options
            .per_isolate
            .per_env
            .debug_options
            .break_first_line
        {
            "--inspect-brk"
        } else {
            "--inspect"
        };
        deno_args.push(format!(
            "{}={}:{}",
            arg,
            parsed_args
                .options
                .per_isolate
                .per_env
                .debug_options
                .host_port
                .host,
            parsed_args
                .options
                .per_isolate
                .per_env
                .debug_options
                .host_port
                .port
        ));
    }

    deno_args
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Macro to create a Vec<String> from string literals
    macro_rules! svec {
        ($($x:expr),* $(,)?) => {
            vec![$($x.to_string()),*]
        };
    }

    /// Test that takes a `input: ["node"]` and `expected: ["deno", "repl", "-A", "--"] `
    macro_rules! test {
        ($name:ident, $input:tt , $expected:tt) => {
            #[test]
            fn $name() {
                let parsed_args = node_cli_parser::parse_args(svec! $input).unwrap();
                let result = translate_to_deno(parsed_args);
                assert_eq!(result, svec! $expected);
            }
        };
    }

    test!(test_repl_no_args, [], ["node", "repl", "-A", "--"]);

    test!(
        test_run_script,
        ["foo.js"],
        [
            "node",
            "run",
            "-A",
            "--unstable-node-globals",
            "--unstable-bare-node-builtins",
            "--unstable-detect-cjs",
            "--node-modules-dir=manual",
            "--no-config",
            "foo.js"
        ]
    );
}
