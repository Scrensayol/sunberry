mod decompiler;
mod generator;
mod luau;
mod reduce;
mod rng;
mod roundtrip;

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::{Args, Parser, Subcommand};
use decompiler::{Decompiler, External, Sunberry};
use generator::Grammar;
use luau::Profile;
use rayon::prelude::*;
use roundtrip::{Kind, Verdict};
use serde::Serialize;

#[derive(Parser)]
#[command(about = "Round-trip tests for Luau decompilers: compile, run, decompile, recompile, run again, compare")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Round-trip hand-written .luau programs (files or directories)
    Check {
        #[command(flatten)]
        common: Common,
        /// Defaults to harness/fixtures, luau-lifter/tests/cases and core/tests/cases
        paths: Vec<PathBuf>,
    },
    /// Round-trip generated programs
    Fuzz {
        #[command(flatten)]
        common: Common,
        /// A count (500) or a range of seeds (1000..1500)
        #[arg(long, default_value = "200")]
        seeds: String,
        /// Shrink each failing program to a minimal one that fails the same way
        #[arg(long)]
        reduce: bool,
        #[arg(long, value_enum, default_value = "full")]
        grammar: Grammar,
    },
    /// Print the program a seed generates
    Gen {
        seed: u64,
        #[arg(long, value_enum, default_value = "full")]
        grammar: Grammar,
    },
    /// Compile a .luau file (or a seed's program) with the harness's Luau, to reproduce a case
    Compile {
        /// a .luau file, or seed<N> for a generated program
        input: String,
        #[arg(long, default_value = "O1g1")]
        profile: String,
        #[arg(long, value_enum, default_value = "full")]
        grammar: Grammar,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Check that every .lua file under a directory compiles
    Syntax { dir: PathBuf },
    /// Check that generated programs compile, finish and don't error at top level
    Selfcheck {
        #[arg(long, default_value = "500")]
        seeds: String,
        #[arg(long, value_enum, default_value = "full")]
        grammar: Grammar,
    },
}

#[derive(Args)]
struct Common {
    /// sunberry or next (the new core), both in-process, or name=command where {} is the bytecode path, e.g.
    /// tovek='/path/luau-lifter {}'. Repeat to compare several.
    #[arg(long = "decompiler", default_value = "sunberry")]
    decompilers: Vec<String>,
    /// Comma-separated profiles such as O1g1,O2g2, or "all"
    #[arg(long, default_value = "all")]
    profiles: String,
    /// Where to write failing cases
    #[arg(long)]
    work: Option<PathBuf>,
    /// Write a JSON summary here
    #[arg(long)]
    report: Option<PathBuf>,
    /// Failures to print per decompiler
    #[arg(long, default_value_t = 15)]
    show: usize,
    /// Known failures: only failures missing from this file fail the run
    #[arg(long)]
    baseline: Option<PathBuf>,
    /// Rewrite --baseline with this run's failures
    #[arg(long, requires = "baseline")]
    update_baseline: bool,
}

#[derive(Serialize, serde::Deserialize, Default)]
struct Baseline {
    generator: String,
    // decompiler -> "case profile"
    failing: BTreeMap<String, Vec<String>>,
    // the same for refusals: a decompiler that refuses instead of guessing shouldn't start
    // refusing something it used to do
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    refused: BTreeMap<String, Vec<String>>,
}

fn main() -> ExitCode {
    let default_hook = std::panic::take_hook();
    // the decompiler's panics are caught and reported as failures; only the harness's own get printed
    std::panic::set_hook(Box::new(move |info| {
        if info.location().is_some_and(|l| l.file().contains("harness")) {
            default_hook(info);
        }
    }));

    match Cli::parse().command {
        Command::Gen { seed, grammar } => {
            print!("{}", generator::generate(seed, grammar).render());
            ExitCode::SUCCESS
        }
        Command::Compile { input, profile, grammar, output } => {
            let Some(profile) = Profile::parse(&profile) else {
                eprintln!("profiles look like O1g1");
                return ExitCode::from(2);
            };
            let source = match input.strip_prefix("seed").and_then(|n| n.parse().ok()) {
                Some(seed) => generator::generate(seed, grammar).render(),
                None => fs::read_to_string(&input).expect("read input"),
            };
            match luau::compile(&source, profile) {
                Ok(bytecode) => {
                    fs::write(&output, bytecode).expect("write output");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("{e}");
                    ExitCode::FAILURE
                }
            }
        }
        Command::Syntax { dir } => {
            let files = fs::read_dir(&dir).expect("read dir").flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "lua")).collect::<Vec<_>>();
            let bad = files
                .par_iter()
                .filter_map(|f| {
                    let text = fs::read_to_string(f).ok()?;
                    luau::compile(&text, Profile { opt: 1, debug: 1 }).err().map(|e| format!("{}: {}", f.display(), e.lines().next().unwrap_or_default()))
                })
                .collect::<Vec<_>>();
            println!("{} files, {} don't compile", files.len(), bad.len());
            for b in bad.iter().take(10) {
                println!("  {b}");
            }
            if bad.is_empty() { ExitCode::SUCCESS } else { ExitCode::FAILURE }
        }
        Command::Selfcheck { seeds, grammar } => {
            let Some(seeds) = parse_seeds(&seeds) else {
                eprintln!("--seeds takes a count or a range like 100..200");
                return ExitCode::from(2);
            };
            let problems = seeds
                .into_par_iter()
                .flat_map_iter(|seed| {
                    let source = generator::generate(seed, grammar).render();
                    Profile::ALL.into_iter().filter_map(move |profile| {
                        let problem = match luau::compile(&source, profile) {
                            Err(e) => format!("compile: {e}"),
                            Ok(bytecode) => match luau::run(&bytecode).ending {
                                luau::Ending::Returned => return None,
                                luau::Ending::TimedOut => "timeout".to_string(),
                                luau::Ending::Errored(e) => format!("error: {e}"),
                            },
                        };
                        Some((seed, profile, problem))
                    })
                })
                .collect::<Vec<_>>();
            let mut kinds = BTreeMap::<String, (usize, u64)>::new();
            for (seed, _, problem) in &problems {
                let key = problem.split(" with ").next().unwrap().to_string();
                kinds.entry(key).or_insert((0, *seed)).0 += 1;
            }
            for (problem, (count, seed)) in &kinds {
                println!("{count:5}  {problem}  (e.g. seed {seed})");
            }
            println!("{} problem runs", problems.len());
            if problems.is_empty() { ExitCode::SUCCESS } else { ExitCode::FAILURE }
        }
        Command::Check { common, paths } => {
            let paths = if paths.is_empty() {
                let root = Path::new(env!("CARGO_MANIFEST_DIR"));
                vec![root.join("fixtures"), root.join("../luau-lifter/tests/cases"), root.join("../core/tests/cases")]
            } else {
                paths
            };
            let mut cases = Vec::new();
            for path in paths {
                collect_luau(&path, &mut cases);
            }
            cases.sort();
            let cases = cases
                .into_iter()
                .map(|p| Case { name: p.file_stem().unwrap().to_string_lossy().into_owned(), source: Source::File(p) })
                .collect();
            run(&common, cases, false, generator::GENERATOR_VERSION)
        }
        Command::Fuzz { common, seeds, reduce, grammar } => {
            let Some(seeds) = parse_seeds(&seeds) else {
                eprintln!("--seeds takes a count or a range like 100..200");
                return ExitCode::from(2);
            };
            let cases = seeds.map(|seed| Case { name: format!("seed{seed}"), source: Source::Seed(seed, grammar) }).collect();
            let generator = match grammar {
                Grammar::Full => generator::GENERATOR_VERSION.to_string(),
                Grammar::Straight => format!("{}-straight", generator::GENERATOR_VERSION),
                Grammar::Branches => format!("{}-branches", generator::GENERATOR_VERSION),
            };
            run(&common, cases, reduce, &generator)
        }
    }
}

fn parse_seeds(text: &str) -> Option<std::ops::Range<u64>> {
    match text.split_once("..") {
        Some((a, b)) => Some(a.parse().ok()?..b.parse().ok()?),
        None => Some(0..text.parse().ok()?),
    }
}

fn collect_luau(path: &Path, out: &mut Vec<PathBuf>) {
    if path.is_dir() {
        for entry in fs::read_dir(path).into_iter().flatten().flatten() {
            collect_luau(&entry.path(), out);
        }
    } else if path.extension().is_some_and(|e| e == "luau") {
        out.push(path.to_path_buf());
    }
}

enum Source {
    File(PathBuf),
    Seed(u64, Grammar),
}

struct Case {
    name: String,
    source: Source,
}

impl Case {
    fn text(&self) -> String {
        match &self.source {
            Source::File(p) => fs::read_to_string(p).unwrap_or_default(),
            Source::Seed(s, g) => generator::generate(*s, *g).render(),
        }
    }
}

#[derive(Serialize)]
struct Failure {
    case: String,
    profile: String,
    kind: Kind,
    detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reduced_lines: Option<usize>,
}

#[derive(Serialize)]
struct Report {
    generator: String,
    cases: usize,
    profiles: Vec<String>,
    // decompiler -> kind -> count
    totals: BTreeMap<String, BTreeMap<Kind, usize>>,
    // decompiler -> profile -> failures
    by_profile: BTreeMap<String, BTreeMap<String, usize>>,
    failures: BTreeMap<String, Vec<Failure>>,
}

fn decompilers(specs: &[String]) -> Result<Vec<Box<dyn Decompiler>>, String> {
    specs
        .iter()
        .map(|spec| -> Result<Box<dyn Decompiler>, String> {
            if spec == "sunberry" {
                Ok(Box::new(Sunberry))
            } else if spec == "next" {
                Ok(Box::new(decompiler::Next))
            } else { Ok(Box::new(External::parse(spec)?)) }
        })
        .collect()
}

fn run(common: &Common, cases: Vec<Case>, reduce: bool, generator_version: &str) -> ExitCode {
    let profiles = if common.profiles == "all" {
        Profile::ALL.to_vec()
    } else {
        match common.profiles.split(',').map(Profile::parse).collect::<Option<Vec<_>>>() {
            Some(p) => p,
            None => {
                eprintln!("profiles look like O1g1 (optimization 0-2, debug 0-2)");
                return ExitCode::from(2);
            }
        }
    };
    let decompilers = match decompilers(&common.decompilers) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("--decompiler: {e}");
            return ExitCode::from(2);
        }
    };

    let decompiler_count = decompilers.len();
    let jobs = cases
        .iter()
        .flat_map(|case| profiles.iter().flat_map(move |p| (0..decompiler_count).map(move |d| (case, *p, d))))
        .collect::<Vec<_>>();
    let started = std::time::Instant::now();
    let results = jobs
        .par_iter()
        .map(|&(case, profile, d)| {
            let source = case.text();
            let decompiler = decompilers[d].as_ref();
            let verdict = roundtrip::check(&source, profile, decompiler);
            let mut reduced = None;
            if verdict.kind.is_failure() {
                if reduce && let Source::Seed(seed, grammar) = case.source {
                    let kind = verdict.kind;
                    let (program, _) = reduce::reduce(
                        &generator::generate(seed, grammar),
                        |p| roundtrip::check(&p.render(), profile, decompiler).kind == kind,
                        400,
                    );
                    reduced = Some(program.render());
                }
                if let Some(work) = &common.work {
                    save(work, decompiler.name(), &case.name, profile, &source, &verdict, reduced.as_deref());
                }
            }
            (case, profile, d, verdict.kind, verdict.detail, reduced.map(|r| r.lines().count()))
        })
        .collect::<Vec<_>>();

    let mut report = Report {
        generator: generator_version.to_string(),
        cases: cases.len(),
        profiles: profiles.iter().map(Profile::to_string).collect(),
        totals: BTreeMap::new(),
        by_profile: BTreeMap::new(),
        failures: BTreeMap::new(),
    };
    let refusals = results
        .iter()
        .filter(|r| r.3 == Kind::Refused)
        .map(|(case, profile, d, kind, detail, _)| (case.name.clone(), *profile, *d, *kind, detail.clone()))
        .collect::<Vec<_>>();
    for (case, profile, d, kind, detail, reduced_lines) in results {
        let name = decompilers[d].name().to_string();
        *report.totals.entry(name.clone()).or_default().entry(kind).or_default() += 1;
        if kind.is_failure() {
            *report.by_profile.entry(name.clone()).or_default().entry(profile.to_string()).or_default() += 1;
            report.failures.entry(name).or_default().push(Failure {
                case: case.name.clone(),
                profile: profile.to_string(),
                kind,
                detail,
                reduced_lines,
            });
        }
    }

    let mut failed = false;
    for decompiler in &decompilers {
        let name = decompiler.name();
        let totals = report.totals.get(name).cloned().unwrap_or_default();
        let count = |k| totals.get(&k).copied().unwrap_or(0);
        let usable = totals.values().sum::<usize>() - count(Kind::Unusable);
        println!(
            "{name}: {}/{usable} pass | {} mismatch, {} don't recompile, {} decompile failures, {} refused | {} unusable inputs",
            count(Kind::Pass),
            count(Kind::Mismatch),
            count(Kind::RecompileFailed),
            count(Kind::DecompileFailed),
            count(Kind::Refused),
            count(Kind::Unusable),
        );
        if let Some(by_profile) = report.by_profile.get(name) {
            let parts = by_profile.iter().map(|(p, n)| format!("{p} {n}")).collect::<Vec<_>>();
            println!("  failures by profile: {}", parts.join(", "));
        }
        for failure in report.failures.get(name).into_iter().flatten().take(common.show) {
            let detail = failure.detail.lines().next().unwrap_or_default();
            let detail = if detail.len() > 160 { format!("{}...", &detail[..detail.floor_char_boundary(160)]) } else { detail.to_string() };
            let reduced = failure.reduced_lines.map(|n| format!(" (reduced to {n} lines)")).unwrap_or_default();
            println!("  {} {} {:?}{reduced}: {detail}", failure.case, failure.profile, failure.kind);
        }
        failed |= report.failures.contains_key(name);
        let mut reasons = BTreeMap::<String, (usize, String)>::new();
        for (case, profile, d, kind, detail) in &refusals {
            if decompilers[*d].name() == name {
                let key = detail.chars().map(|c| if c.is_ascii_digit() { 'N' } else { c }).collect::<String>().replace("NN", "N").replace("NN", "N");
                reasons.entry(key).or_insert((0, format!("{case} {profile}"))).0 += 1;
                let _ = kind;
            }
        }
        let mut reasons = reasons.into_iter().collect::<Vec<_>>();
        reasons.sort_by(|a, b| b.1.0.cmp(&a.1.0).then(a.0.cmp(&b.0)));
        for (reason, (count, example)) in reasons.iter().take(8) {
            println!("  refused {count:5}x  {reason}  (e.g. {example})");
        }
    }
    println!("{} round trips in {:.1}s", jobs.len(), started.elapsed().as_secs_f64());

    if let Some(path) = &common.report {
        fs::write(path, serde_json::to_string_pretty(&report).unwrap()).expect("write report");
    }

    if let Some(path) = &common.baseline {
        let current = Baseline {
            generator: generator_version.to_string(),
            failing: report
                .failures
                .iter()
                .map(|(name, failures)| {
                    let mut keys = failures.iter().map(|f| format!("{} {}", f.case, f.profile)).collect::<Vec<_>>();
                    keys.sort();
                    (name.clone(), keys)
                })
                .collect(),
            refused: {
                let mut refused = BTreeMap::<String, Vec<String>>::new();
                for (case, profile, d, _, _) in &refusals {
                    refused.entry(decompilers[*d].name().to_string()).or_default().push(format!("{case} {profile}"));
                }
                refused.values_mut().for_each(|keys| keys.sort());
                refused
            },
        };
        if common.update_baseline {
            fs::write(path, serde_json::to_string_pretty(&current).unwrap() + "\n").expect("write baseline");
            println!("baseline written to {}", path.display());
            return ExitCode::SUCCESS;
        }
        let known: Baseline = match fs::read_to_string(path).map(|t| serde_json::from_str(&t)) {
            Ok(Ok(b)) => b,
            _ => {
                eprintln!("can't read baseline {}", path.display());
                return ExitCode::from(2);
            }
        };
        if known.generator != current.generator {
            eprintln!("baseline is for generator {}, this is {}; rerun with --update-baseline", known.generator, current.generator);
            return ExitCode::from(2);
        }
        let mut regressed = false;
        for decompiler in &decompilers {
            let name = decompiler.name();
            for (what, known, current) in [("failures", &known.failing, &current.failing), ("refusals", &known.refused, &current.refused)] {
                let empty = Vec::new();
                let (was, now) = (known.get(name).unwrap_or(&empty), current.get(name).unwrap_or(&empty));
                let new = now.iter().filter(|k| was.binary_search(k).is_err()).collect::<Vec<_>>();
                let fixed = was.iter().filter(|k| now.binary_search(k).is_err()).count();
                if !new.is_empty() {
                    regressed = true;
                    println!("{name}: {} new {what} against the baseline: {}", new.len(), new.iter().take(20).map(|k| k.as_str()).collect::<Vec<_>>().join(", "));
                }
                if fixed > 0 {
                    println!("{name}: {fixed} baseline {what} are gone now; rerun with --update-baseline to lock that in");
                }
            }
        }
        return if regressed { ExitCode::FAILURE } else { ExitCode::SUCCESS };
    }
    if failed { ExitCode::FAILURE } else { ExitCode::SUCCESS }
}

fn save(work: &Path, decompiler: &str, case: &str, profile: Profile, source: &str, verdict: &Verdict, reduced: Option<&str>) {
    let dir = work.join(decompiler).join(format!("{case}-{profile}"));
    if fs::create_dir_all(&dir).is_err() {
        return;
    }
    let _ = fs::write(dir.join("input.luau"), source);
    let _ = fs::write(dir.join("detail.txt"), format!("{:?}: {}\n", verdict.kind, verdict.detail));
    if let Some(output) = &verdict.output {
        let _ = fs::write(dir.join("output.lua"), output);
    }
    if let Some(want) = &verdict.want {
        let _ = fs::write(dir.join("want.txt"), want.render());
    }
    if let Some(got) = &verdict.got {
        let _ = fs::write(dir.join("got.txt"), got.render());
    }
    if let Some(reduced) = reduced {
        let _ = fs::write(dir.join("reduced.luau"), reduced);
    }
}
