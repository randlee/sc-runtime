//! Optional benchmarks: CPython hook vs Monty host-bridge lab for sc_compose.
//!
//! Each run writes two files under `target/sc-compose-bench/` (or
//! `$CARGO_TARGET_DIR/sc-compose-bench`):
//!
//! - `sc-compose-hook-comparison.report.html`
//! - `sc-compose-hook-comparison.report.json`
//!
//! `scripts/benchmark/monty/campaign.py` copies the HTML and the case
//! metrics into `site/reports/benchmark-monty/`. Default CI does not run this
//! test (`required-features = ["monty"]`, and the test is `#[ignore]`d).
//!
//! Run:
//! ```text
//! just --justfile scripts/benchmark/justfile benchmark-monty
//! ```

#![cfg(unix)]
#![expect(clippy::expect_used, reason = "benchmark harness")]

use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

const TEMPLATE: &str = "Hello {{ name }}";
/// Code shown in the HTML report beside the timing table. Excerpts only; the
/// executed code is `monty_host_bridge_once` and `CliRenderFixtures` below.
const EXAMPLE_MONTY_HOOK: &str = "def cron_tick(hook):
    # render_template is the reserved host binding (resumed by the host).
    out = render_template(hook[\"payload\"][\"template\"], hook[\"payload\"][\"context\"])
    return None  # normal return is RunOutcome::Ok
";
/// Lab `monty_host_bridge` harness. Production campaigns publish
/// [`EXAMPLE_PRODUCTION_RESUME_LOOP`] instead of this excerpt.
const EXAMPLE_RESUME_LOOP: &str = r#"let mut progress = repl.feed_start(snippet, inputs, PrintWriter::Disabled)?;
loop {
    progress = match progress {
        ReplProgress::FunctionCall(call) if call.function_name == "render_template" => {
            let template = call.args.arg(0)?.as_str()?;
            let rendered = run_sc_compose(template, context); // host spawns the CLI
            call.resume(ExtFunctionResult::Return(MontyObject::string(rendered)),
                        PrintWriter::Disabled)?
        }
        ReplProgress::Complete { .. } => break,
        _ => return Err(denied()),
    };
}"#;
/// Production path inside `execute_feed`. The bench does not call `feed_start`
/// itself for `production_bridge`; this is the host loop that does.
const EXAMPLE_PRODUCTION_RESUME_LOOP: &str = r#"// production path: execute_feed (not the lab feed_start harness)
let mut progress = match repl.feed_start(
    phase_snippet(phase),
    snippet_inputs(hook),
    PrintWriter::Disabled,
) {
    Ok(progress) => progress,
    Err(error) => return from_exception(error.error, stage),
};
loop {
    progress = match progress {
        ReplProgress::FunctionCall(call) if call.function_name == "render_template" => {
            match resume_render(call, stage, compose_program, host_deadline) {
                Ok(next) => next,
                Err(done) => return done,
            }
        }
        ReplProgress::FunctionCall(call) if call.function_name == "sc_compose_run" => {
            match resume_compose_run(call, stage, compose_program, host_deadline) {
                Ok(next) => next,
                Err(done) => return done,
            }
        }
        ReplProgress::Complete { repl, value } => {
            return finish_phase(phase, generation, repl, &value);
        }
        ReplProgress::FunctionCall(call) => {
            return denied(stage, &format!("host function {}", call.function_name));
        }
        ReplProgress::OsCall(call) => {
            let operation: &'static str = call.function_call.into();
            return denied_os(stage, operation);
        }
        ReplProgress::NameLookup(lookup) => return undefined_name(stage, &lookup.name),
        ReplProgress::ResolveFutures(_) => return denied(stage, "unresolved futures"),
    };
}
"#;
const EXAMPLE_CLI_ARGV: &str =
    "sc-compose render --root <temp> --mode file --file t.j2 --var-file <temp>/v.json";
const EXAMPLE_COMPOSE_RUN: &str = r#"def cron_tick(hook):
    # Only render is allowed. The host stages a temp root and ignores --root.
    # bead, init, and extract fail the tick with COMPOSE_SUBCOMMAND.
    return sc_compose_run("render", [hook["payload"]["template"], '{"name": "bench"}'])
"#;
const EXAMPLE_PRODUCTION: &str = r#"def cron_stop(hook):
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    return render_template(hook["payload"]["template"], hook["payload"]["context"])
"#;

/// Timed iterations per case (plus one untimed warmup before the timer starts).
const TIMED_ITERS: u32 = 30;

struct BenchCase {
    id: &'static str,
    label: &'static str,
    strategy: &'static str,
    skipped: bool,
    skip_reason: Option<&'static str>,
    result: Option<BenchResult>,
}

struct BenchResult {
    timed_iters: u32,
    warmup_iters: u32,
    total_elapsed: Duration,
    per_run: Duration,
    subprocess_peak_rss: Option<RssNote>,
    in_process_rss: Option<RssNote>,
}

struct RssNote {
    bytes: u64,
    method: &'static str,
}

struct ReportContext {
    hostname: String,
    sc_compose_cli_version: Option<String>,
    python_sc_compose: bool,
    cargo_profile: &'static str,
}

fn bench_enabled() -> bool {
    std::env::var("SC_CRON_BENCH").ok().as_deref() == Some("1")
}

fn sc_compose_python_available() -> bool {
    Command::new("python3")
        .args(["-c", "import sc_compose"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn sc_compose_cli_available() -> bool {
    Command::new("sc-compose")
        .arg("--version")
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn sc_compose_cli_version() -> Option<String> {
    let output = Command::new("sc-compose").arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Fixed template tree for `sc-compose render --mode file` (Rust CLI, no Python).
struct CliRenderFixtures {
    _dir: tempfile::TempDir,
    root: PathBuf,
    var_file: PathBuf,
}

impl CliRenderFixtures {
    fn shared() -> &'static Self {
        static FIX: OnceLock<CliRenderFixtures> = OnceLock::new();
        FIX.get_or_init(|| {
            let dir = tempfile::TempDir::new().expect("bench temp root");
            let root = dir.path().to_path_buf();
            std::fs::write(root.join("t.j2"), TEMPLATE).expect("template");
            let var_file = root.join("v.json");
            std::fs::write(&var_file, r#"{"name":"bench"}"#).expect("vars");
            CliRenderFixtures {
                _dir: dir,
                root,
                var_file,
            }
        })
    }

    fn render_string_template(&self, _template: &str, _context: &serde_json::Value) -> String {
        let output = Command::new("sc-compose")
            .args(["render", "--root"])
            .arg(&self.root)
            .args(["--mode", "file", "--file", "t.j2", "--var-file"])
            .arg(&self.var_file)
            .output()
            .expect("sc-compose");
        assert!(
            output.status.success(),
            "sc-compose: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout)
            .expect("utf8")
            .lines()
            .next()
            .unwrap_or("")
            .trim()
            .to_string()
    }

    fn render_command(&self) -> Command {
        let mut cmd = Command::new("sc-compose");
        cmd.args(["render", "--mode", "file", "--file", "t.j2"])
            .arg("--root")
            .arg(&self.root)
            .arg("--var-file")
            .arg(&self.var_file);
        cmd
    }
}

fn sc_compose_cli_render_once() {
    let rendered = CliRenderFixtures::shared()
        .render_string_template(TEMPLATE, &serde_json::json!({"name": "bench"}));
    assert_eq!(rendered, "Hello bench");
}

fn fixture_module() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sc_compose_tick.py")
}

fn python_render_once() {
    let script =
        format!("import sc_compose; sc_compose.render_template({TEMPLATE:?}, {{'name': 'bench'}})");
    let status = Command::new("python3")
        .args(["-c", &script])
        .status()
        .expect("python3");
    assert!(status.success());
}

fn python_hook_once() {
    use sc_cron::{Clock, HookCall, HookEvent, ManualClock, PythonHook, RunTimeout};

    let module = fixture_module();
    let hook = PythonHook::new(&module).expect("hook path");
    let clock = ManualClock::new("2026-10-19T00:00:00Z".parse().expect("ts"));
    let schedule = sc_cron::ScheduleName::new("bench").expect("name");
    let payload: serde_json::Value = serde_json::json!({
        "template": TEMPLATE,
        "context": {"name": "bench"},
    });
    let call = HookCall {
        event: HookEvent::Tick,
        schedule: &schedule,
        payload: &payload,
        timeout: RunTimeout::new(Duration::from_secs(30)).expect("timeout"),
        deadline: clock.monotonic_now() + Duration::from_secs(30),
    };
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    runtime
        .block_on(hook.invoke(&clock, &call)) // allow: bench drives PythonHook on a local runtime
        .expect("invoke");
}

#[cfg(feature = "monty")]
mod common;

#[cfg(feature = "monty")]
fn monty_host_bridge_once() {
    use monty::{MontyRepl, ReplProgress};
    use monty_types::{
        CompileOptions, ExtFunctionResult, MontyObject, NamedValues, PrintWriter, ResourceTracker,
    };

    let snippet = format!("render_template({TEMPLATE:?}, {{'name': 'bench'}})");
    let mut inputs = NamedValues::new();
    inputs.push(
        "render_template",
        MontyObject::function("render_template".to_owned(), None),
    );
    let repl = MontyRepl::new(
        "sc-compose-bench",
        ResourceTracker::default(),
        CompileOptions::default(),
    );
    let fixtures = CliRenderFixtures::shared();
    let mut progress = repl
        .feed_start(&snippet, inputs, PrintWriter::Disabled)
        .expect("feed_start");
    loop {
        progress = match progress {
            ReplProgress::FunctionCall(call) => {
                assert_eq!(call.function_name, "render_template");
                let template = call
                    .args
                    .arg(0)
                    .expect("arg0")
                    .as_str()
                    .expect("str")
                    .to_owned();
                let context_json = serde_json::json!({"name": "bench"});
                let rendered = fixtures.render_string_template(&template, &context_json);
                call.resume(
                    ExtFunctionResult::Return(MontyObject::string(rendered)),
                    PrintWriter::Disabled,
                )
                .expect("resume")
            }
            ReplProgress::Complete { .. } => break,
            other => panic!("unexpected progress: {other:?}"),
        };
    }
}

#[cfg(feature = "monty")]
fn production_bridge_once() {
    const SOURCE: &str = r#"
def cron_stop(hook):
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    out = render_template(hook["payload"]["template"], hook["payload"]["context"])
    if out.strip() != "Hello bench":
        raise RuntimeError("unexpected render: " + out)
    return None
"#;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    runtime.block_on(production_bridge_run(SOURCE)) // allow: bench local runtime
}

#[cfg(feature = "monty")]
async fn production_bridge_run(source: &str) {
    use common::{Fixture, WAIT, idle_job};
    use sc_cron::{HookTarget, RunOutcome, ScheduleSpec};
    use serde_json::json;

    let fixture = Fixture::new();
    let spec = ScheduleSpec::with_hook(
        "bench-production-bridge",
        "* * * * *",
        Duration::from_secs(20),
        json!({
            "template": TEMPLATE,
            "context": {"name": "bench"},
        }),
        HookTarget::monty(source).expect("source"),
    )
    .expect("spec");
    fixture.start(spec);
    fixture.clock.advance(chrono::Duration::seconds(60));
    let runs = match tokio::time::timeout(WAIT, fixture.scheduler.run_due(idle_job())).await {
        Ok(result) => result.expect("run_due"),
        Err(_elapsed) => panic!("run_due timed out after {WAIT:?}"),
    };
    assert_eq!(runs.len(), 1, "{runs:?}");
    assert_eq!(runs[0].outcome, RunOutcome::Ok, "{runs:?}");
}

/// Peak RSS of a one-shot subprocess (macOS `/usr/bin/time -l`).
fn subprocess_peak_rss_bytes(cmd: Command) -> Option<u64> {
    let program = cmd.get_program().to_string_lossy().into_owned();
    let args: Vec<String> = cmd
        .get_args()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    let mut time_cmd = Command::new("/usr/bin/time");
    time_cmd
        .arg("-l")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .arg(program);
    for arg in args {
        time_cmd.arg(arg);
    }
    let output = time_cmd.output().ok()?;
    if !output.status.success() {
        return None;
    }
    parse_time_l_peak_bytes(&String::from_utf8_lossy(&output.stderr))
}

fn parse_time_l_peak_bytes(stderr: &str) -> Option<u64> {
    for line in stderr.lines() {
        let line = line.trim();
        if line.ends_with("peak memory footprint") || line.ends_with("maximum resident set size") {
            return line.split_whitespace().next().and_then(|s| s.parse().ok());
        }
    }
    None
}

fn rusage_max_rss_bytes(who: libc::c_int) -> Option<u64> {
    let mut usage = std::mem::MaybeUninit::<libc::rusage>::uninit();
    // SAFETY: rusage is plain C struct; getrusage writes only on success.
    let rc = unsafe { libc::getrusage(who, usage.as_mut_ptr()) };
    if rc != 0 {
        return None;
    }
    let usage = unsafe { usage.assume_init() };
    // macOS reports bytes. Linux reports kilobytes.
    #[cfg(target_os = "macos")]
    {
        Some(usage.ru_maxrss as u64)
    }
    #[cfg(not(target_os = "macos"))]
    {
        Some(usage.ru_maxrss as u64 * 1024)
    }
}

fn run_timed_bench(
    timed_iters: u32,
    mut f: impl FnMut(),
    subprocess_rss_cmd: Option<Command>,
    measure_in_process_after: bool,
) -> BenchResult {
    f();
    let start = Instant::now();
    for _ in 0..timed_iters {
        f();
    }
    let total_elapsed = start.elapsed();
    let subprocess_peak_rss = subprocess_rss_cmd.and_then(|cmd| {
        subprocess_peak_rss_bytes(cmd).map(|bytes| RssNote {
            bytes,
            method: "One-shot subprocess via /usr/bin/time -l (peak memory footprint)",
        })
    });
    let in_process_rss = if measure_in_process_after {
        rusage_max_rss_bytes(libc::RUSAGE_SELF).map(|bytes| RssNote {
            bytes,
            method: "getrusage(RUSAGE_SELF) high-water after timed loop (includes cargo test + Monty VM; not an isolated Monty-only process)",
        })
    } else {
        None
    };
    BenchResult {
        timed_iters,
        warmup_iters: 1,
        total_elapsed,
        per_run: total_elapsed / timed_iters,
        subprocess_peak_rss,
        in_process_rss,
    }
}

fn harness_output_dir() -> PathBuf {
    match std::env::var_os("CARGO_TARGET_DIR") {
        Some(dir) => PathBuf::from(dir).join("sc-compose-bench"),
        None => PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/sc-compose-bench"),
    }
}

fn report_html_path() -> PathBuf {
    harness_output_dir().join("sc-compose-hook-comparison.report.html")
}

fn report_json_path() -> PathBuf {
    harness_output_dir().join("sc-compose-hook-comparison.report.json")
}

fn rss_pair(note: Option<&RssNote>) -> (Option<u64>, Option<&str>) {
    match note {
        Some(note) => (Some(note.bytes), Some(note.method)),
        None => (None, None),
    }
}

fn case_metrics(case: &BenchCase) -> serde_json::Value {
    let result = case.result.as_ref();
    let (subprocess_rss_bytes, subprocess_rss_note) =
        rss_pair(result.and_then(|result| result.subprocess_peak_rss.as_ref()));
    let (in_process_rss_bytes, in_process_rss_note) =
        rss_pair(result.and_then(|result| result.in_process_rss.as_ref()));
    serde_json::json!({
        "id": case.id,
        "label": case.label,
        "strategy": case.strategy,
        "skipped": case.skipped,
        "skip_reason": case.skip_reason,
        "ms_per_run": result.map(|result| result.per_run.as_secs_f64() * 1000.0),
        "timed_iters": result.map(|result| result.timed_iters),
        "total_ms": result.map(|result| result.total_elapsed.as_secs_f64() * 1000.0),
        "subprocess_rss_bytes": subprocess_rss_bytes,
        "subprocess_rss_note": subprocess_rss_note,
        "in_process_rss_bytes": in_process_rss_bytes,
        "in_process_rss_note": in_process_rss_note,
    })
}

/// Machine-readable sidecar the campaign publisher reads.
fn write_metrics_json(ctx: &ReportContext, cases: &[BenchCase]) {
    let path = report_json_path();
    let doc = serde_json::json!({
        "schema": "sc-runtime/benchmark-monty-harness/v1",
        "host": ctx.hostname,
        "cargo_profile": ctx.cargo_profile,
        "sc_compose_cli_version": ctx.sc_compose_cli_version,
        "python_sc_compose": ctx.python_sc_compose,
        "cases": cases.iter().map(case_metrics).collect::<Vec<_>>(),
        "examples": {
            "hook_source": EXAMPLE_PRODUCTION,
            "resume_loop": EXAMPLE_PRODUCTION_RESUME_LOOP,
            "lab_resume_loop": EXAMPLE_RESUME_LOOP,
            "cli_argv": EXAMPLE_CLI_ARGV,
            "compose_run": EXAMPLE_COMPOSE_RUN,
        },
    });
    let body = serde_json::to_string_pretty(&doc).expect("metrics json");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("report dir");
    }
    std::fs::write(&path, body + "\n").expect("write metrics json");
    eprintln!("Wrote {}", path.display());
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn format_mib(bytes: u64) -> String {
    format!("{:.2}", bytes as f64 / (1024.0 * 1024.0))
}

fn write_html_report(ctx: &ReportContext, cases: &[BenchCase]) {
    let generated = chrono_lite_now();
    let path = report_html_path();
    let mut rows = String::new();
    for case in cases {
        if case.skipped {
            rows.push_str(&format!(
                "<tr id=\"{}\"><td><strong>{}</strong><p class=\"strategy\">{}</p></td><td colspan=\"4\" class=\"muted\">Skipped: {}</td></tr>\n",
                html_escape(case.id),
                html_escape(case.label),
                html_escape(case.strategy),
                html_escape(case.skip_reason.unwrap_or("unknown")),
            ));
            continue;
        }
        let Some(r) = &case.result else { continue };
        let rss_cell = match (&r.subprocess_peak_rss, &r.in_process_rss) {
            (Some(sub), Some(parent)) => format!(
                "subprocess {} MiB<br><span class=\"muted\">{}</span><br>parent {} MiB<br><span class=\"muted\">{}</span>",
                format_mib(sub.bytes),
                html_escape(sub.method),
                format_mib(parent.bytes),
                html_escape(parent.method),
            ),
            (Some(sub), None) => format!(
                "{} MiB<br><span class=\"muted\">{}</span>",
                format_mib(sub.bytes),
                html_escape(sub.method),
            ),
            (None, Some(parent)) => format!(
                "{} MiB (in-process)<br><span class=\"muted\">{}</span>",
                format_mib(parent.bytes),
                html_escape(parent.method),
            ),
            (None, None) => "<span class=\"muted\">Not measured</span>".to_string(),
        };
        rows.push_str(&format!(
            "<tr id=\"{}\"><td><strong>{}</strong><p class=\"strategy\">{}</p></td><td class=\"num\">{:.3} ms</td><td class=\"num\">{}</td><td class=\"num\">{} + {} warmup</td><td>{rss_cell}</td></tr>\n",
            html_escape(case.id),
            html_escape(case.label),
            html_escape(case.strategy),
            r.per_run.as_secs_f64() * 1000.0,
            html_escape(&format!("{:?} total", r.total_elapsed)),
            r.timed_iters,
            r.warmup_iters,
        ));
    }

    let cli_ver = ctx
        .sc_compose_cli_version
        .as_deref()
        .unwrap_or("(not detected)");
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>sc-compose hook benchmark report</title>
<style>
:root {{ --bg:#0f1419; --panel:#1a2332; --text:#e7ecf3; --muted:#9aa8bc; --accent:#6cb6ff; --border:#30363d; }}
body {{ font-family: system-ui,sans-serif; background:var(--bg); color:var(--text); line-height:1.55; margin:0; padding:2rem 1.5rem 3rem; max-width:56rem; margin-inline:auto; }}
h1 {{ font-size:1.45rem; margin:0 0 .25rem; }}
.subtitle {{ color:var(--muted); font-size:.95rem; margin-bottom:1.5rem; }}
section {{ background:var(--panel); border:1px solid var(--border); border-radius:10px; padding:1.2rem 1.3rem; margin-bottom:1.2rem; }}
h2 {{ font-size:1.05rem; margin:0 0 .65rem; color:var(--accent); }}
table {{ width:100%; border-collapse:collapse; font-size:.9rem; }}
th,td {{ text-align:left; padding:.5rem; border-bottom:1px solid var(--border); vertical-align:top; }}
th {{ color:var(--muted); font-size:.75rem; text-transform:uppercase; letter-spacing:.03em; }}
.strategy {{ color:var(--muted); font-size:.85rem; margin:.35rem 0 0; }}
.num {{ font-variant-numeric:tabular-nums; }}
.muted {{ color:var(--muted); font-size:.82rem; }}
ul {{ margin:.4rem 0 0; padding-left:1.2rem; }}
code {{ font-family:ui-monospace,Menlo,monospace; font-size:.85em; }}
h3 {{ font-size:.9rem; margin:.9rem 0 .3rem; color:var(--muted); }}
pre {{ background:#0b0f14; border:1px solid var(--border); border-radius:6px; padding:.75rem; overflow-x:auto; }}
</style>
</head>
<body>
<h1>sc-compose render: hook strategies compared</h1>
<p class="subtitle">Auto-generated {generated} · host {host} · profile {profile} · sc-compose {cli_ver} · python sc_compose: {py}</p>

<section>
<h2>Methodology</h2>
<ul>
<li><strong>Timed iterations:</strong> {timed_iters} measured runs per case, each preceded by <strong>1 untimed warmup</strong> ({total_execs} executions of the workload per case).</li>
<li><strong>Wall clock:</strong> Rust <code>Instant</code> over the timed block only (warmup excluded).</li>
<li><strong>Subprocess RSS:</strong> macOS <code>/usr/bin/time -l</code> on a <em>separate</em> one-shot command (isolates child peak).</li>
<li><strong>Monty in-process RSS:</strong> <code>getrusage(RUSAGE_SELF)</code> after the timed Monty block—includes the test binary and Monty VM, not a standalone daemon.</li>
<li><strong>Why Monty had no RSS earlier:</strong> the harness only attached <code>time -l</code> to subprocess cases; the Monty path runs inside <code>cargo test</code> until we added in-process <code>getrusage</code> and documented the compose child peak separately.</li>
</ul>
</section>

<section>
<h2>Results</h2>
<table>
<thead><tr><th>Case &amp; strategy</th><th>Mean / run</th><th>Total (timed block)</th><th>Iterations</th><th>Memory</th></tr></thead>
<tbody>
{rows}
</tbody>
</table>
<p class="muted">Template: <code>{template}</code> · Harness: <code>crates/sc-cron/tests/sc_compose_bench.rs</code></p>
</section>

<section>
<h2>Code examples</h2>
<p class="muted">What each Monty case runs. The timing table above measures this path.</p>
<h3>Monty hook</h3>
<pre><code>{ex_hook}</code></pre>
<h3>production execute_feed resume loop</h3>
<pre><code>{ex_loop}</code></pre>
<h3>sc-compose CLI argv</h3>
<pre><code>{ex_argv}</code></pre>
<h3>sc_compose_run hook</h3>
<pre><code>{ex_compose}</code></pre>
<h3>lab feed_start / resume loop (monty_host_bridge only)</h3>
<pre><code>{ex_lab}</code></pre>
</section>

<section>
<h2>Reproduce</h2>
<pre>just --justfile scripts/benchmark/justfile benchmark-monty</pre>
<p class="muted">Harness output (not published): <code>target/sc-compose-bench/sc-compose-hook-comparison.report.html</code> and <code>target/sc-compose-bench/sc-compose-hook-comparison.report.json</code>. Published copies: <code>site/reports/benchmark-monty/</code>.</p>
</section>
</body>
</html>"#,
        generated = html_escape(&generated),
        host = html_escape(&ctx.hostname),
        profile = ctx.cargo_profile,
        cli_ver = html_escape(cli_ver),
        py = if ctx.python_sc_compose { "yes" } else { "no" },
        timed_iters = TIMED_ITERS,
        total_execs = TIMED_ITERS + 1,
        rows = rows,
        template = html_escape(TEMPLATE),
        ex_hook = html_escape(EXAMPLE_MONTY_HOOK),
        ex_loop = html_escape(EXAMPLE_PRODUCTION_RESUME_LOOP),
        ex_argv = html_escape(EXAMPLE_CLI_ARGV),
        ex_compose = html_escape(EXAMPLE_COMPOSE_RUN),
        ex_lab = html_escape(EXAMPLE_RESUME_LOOP),
    );

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("report dir");
    }
    std::fs::write(&path, html).expect("write report");
    eprintln!("Wrote {}", path.display());
}

fn chrono_lite_now() -> String {
    use std::time::SystemTime;
    let Ok(d) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) else {
        return "unknown time".to_string();
    };
    format!("{} UTC (unix {})", d.as_secs(), d.as_secs())
}

fn hostname() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::fs::read_to_string("/etc/hostname"))
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string()
}

fn log_result(label: &str, r: &BenchResult) {
    eprintln!(
        "{label}: {} timed runs (+{} warmup) in {:?} ({:?}/run)",
        r.timed_iters, r.warmup_iters, r.total_elapsed, r.per_run
    );
    if let Some(sub) = &r.subprocess_peak_rss {
        eprintln!(
            "  subprocess peak RSS: {} MiB ({})",
            format_mib(sub.bytes),
            sub.method
        );
    }
    if let Some(parent) = &r.in_process_rss {
        eprintln!(
            "  in-process RSS high-water: {} MiB ({})",
            format_mib(parent.bytes),
            parent.method
        );
    }
}

#[test]
#[ignore = "set SC_CRON_BENCH=1 to run; needs sc_compose Python + sc-compose CLI"]
fn bench_sc_compose_hook_paths() {
    if !bench_enabled() {
        return;
    }

    let have_py = sc_compose_python_available();
    let have_cli = sc_compose_cli_available();
    if !have_py && !have_cli {
        eprintln!("skip: neither sc_compose (python) nor sc-compose CLI found");
        return;
    }

    let ctx = ReportContext {
        hostname: hostname(),
        sc_compose_cli_version: sc_compose_cli_version(),
        python_sc_compose: have_py,
        cargo_profile: if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        },
    };

    let render_script =
        format!("import sc_compose; sc_compose.render_template({TEMPLATE:?}, {{'name': 'bench'}})");

    eprintln!("--- sc-compose hook benchmarks ({TIMED_ITERS} timed + 1 warmup each) ---");

    let mut cases: Vec<BenchCase> = Vec::new();

    if have_cli {
        let fix = CliRenderFixtures::shared();
        let result = run_timed_bench(
            TIMED_ITERS,
            sc_compose_cli_render_once,
            Some(fix.render_command()),
            false,
        );
        log_result("sc_compose_cli_only (Rust binary, no Python)", &result);
        cases.push(BenchCase {
            id: "cli",
            label: "sc_compose_cli_only",
            strategy: "Direct `sc-compose render --mode file` against a fixed temp-root template and JSON var-file. No sc-cron, no Monty, no Python—baseline for what a host bridge should call.",
            skipped: false,
            skip_reason: None,
            result: Some(result),
        });
    } else {
        cases.push(BenchCase {
            id: "cli",
            label: "sc_compose_cli_only",
            strategy: "Direct Rust sc-compose CLI render.",
            skipped: true,
            skip_reason: Some("sc-compose not on PATH"),
            result: None,
        });
    }

    if have_py {
        let mut py_cmd = Command::new("python3");
        py_cmd.args(["-c", &render_script]);
        let result = run_timed_bench(TIMED_ITERS, python_render_once, Some(py_cmd), false);
        log_result("python_render_only (PyO3 in fresh python3 -c)", &result);
        cases.push(BenchCase {
            id: "py_render",
            label: "python_render_only",
            strategy: "Fresh `python3 -c` each tick: import PyO3 `sc_compose`, call `render_template(str, dict)`. Models per-tick cold subprocess + interpreter startup (worst case for compose in Python).",
            skipped: false,
            skip_reason: None,
            result: Some(result),
        });

        let mut hook_rss = Command::new("python3");
        hook_rss.args(["-c", &render_script]);
        let result = run_timed_bench(TIMED_ITERS, python_hook_once, Some(hook_rss), false);
        log_result("cpython_hook (PythonHook::invoke)", &result);
        cases.push(BenchCase {
            id: "py_hook",
            label: "cpython_hook (PythonHook::invoke)",
            strategy: "Full sc-cron path: `PythonHook` spawns `python3 <fixture>` with `cron.tick`, hook JSON on stdin; fixture calls `sc_compose.render_template` from payload. Production `HookTarget::Python` pattern (isolated child + timeout/kill).",
            skipped: false,
            skip_reason: None,
            result: Some(result),
        });
    } else {
        for (id, label, strategy) in [
            (
                "py_render",
                "python_render_only",
                "PyO3 sc_compose in python3 -c.",
            ),
            (
                "py_hook",
                "cpython_hook",
                "PythonHook::invoke with fixture module.",
            ),
        ] {
            cases.push(BenchCase {
                id,
                label,
                strategy,
                skipped: true,
                skip_reason: Some("import sc_compose failed"),
                result: None,
            });
        }
    }

    #[cfg(feature = "monty")]
    if have_cli {
        let fix = CliRenderFixtures::shared();
        let result = run_timed_bench(
            TIMED_ITERS,
            monty_host_bridge_once,
            Some(fix.render_command()),
            true,
        );
        log_result(
            "monty_host_bridge + sc-compose CLI (target pattern)",
            &result,
        );
        cases.push(BenchCase {
            id: "monty",
            label: "monty_host_bridge + sc-compose CLI",
            strategy: "Lab only (not production sc-cron yet): `MontyRepl::feed_start` runs snippet `render_template(...)`; host registers ext fn `render_template`, resumes with output from Rust `sc-compose` CLI (same temp-root files). Hook logic stays in Monty; compose stays out of CPython.",
            skipped: false,
            skip_reason: None,
            result: Some(result),
        });
    } else {
        cases.push(BenchCase {
            id: "monty",
            label: "monty_host_bridge + sc-compose CLI",
            strategy: "Monty VM + host bridge to sc-compose CLI.",
            skipped: true,
            skip_reason: Some("sc-compose CLI or --features monty missing"),
            result: None,
        });
    }

    #[cfg(not(feature = "monty"))]
    cases.push(BenchCase {
        id: "monty",
        label: "monty_host_bridge + sc-compose CLI",
        strategy: "Monty VM + host bridge to sc-compose CLI.",
        skipped: true,
        skip_reason: Some("build without --features monty"),
        result: None,
    });

    #[cfg(feature = "monty")]
    if have_cli {
        let fix = CliRenderFixtures::shared();
        let result = run_timed_bench(
            TIMED_ITERS,
            production_bridge_once,
            Some(fix.render_command()),
            true,
        );
        log_result(
            "production_bridge (Scheduler::run_due / execute_feed)",
            &result,
        );
        cases.push(BenchCase {
            id: "production_bridge",
            label: "production monty_host bridge",
            strategy: "Production path: `HookTarget::Monty` schedule row, `Scheduler::run_due`, `execute_feed` resumes `render_template` and spawns `sc-compose` (no lab `feed_start` in this harness).",
            skipped: false,
            skip_reason: None,
            result: Some(result),
        });
    } else {
        cases.push(BenchCase {
            id: "production_bridge",
            label: "production monty_host bridge",
            strategy: "Production `execute_feed` + `render_template`.",
            skipped: true,
            skip_reason: Some("sc-compose CLI missing"),
            result: None,
        });
    }

    #[cfg(not(feature = "monty"))]
    cases.push(BenchCase {
        id: "production_bridge",
        label: "production monty_host bridge",
        strategy: "Production `execute_feed` + `render_template`.",
        skipped: true,
        skip_reason: Some("build without --features monty"),
        result: None,
    });

    write_metrics_json(&ctx, &cases);
    write_html_report(&ctx, &cases);
    eprintln!("See docs/sc-cron/hooks-sc-compose.md and site/reports/benchmark-monty/.");
}
