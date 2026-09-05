//! Size probes through the concept layer: conceive, actualize, textualize,
//! protosize and drop of deep and wide datoms in bounded, linear memory.

use std::process::Command;

use datomic::{Actualizable, Conceivable, Datom, Datomic, Potential, Protosizable, Textualizable};

const SIZES: [usize; 3] = [1_000, 10_000, 100_000];
const MODES: [&str; 5] = [
    "read-brackets",
    "read-chain",
    "read-vector",
    "write-brackets",
    "write-vector",
];

fn peak_kb() -> u64 {
    let status = std::fs::read_to_string("/proc/self/status").unwrap();
    let line = status.lines().find(|l| l.starts_with("VmHWM:")).unwrap();
    line.split_whitespace().nth(1).unwrap().parse().unwrap()
}

fn nested(n: usize) -> Datom {
    let mut datom = Datom::Vector(vec![]);
    for _ in 0..n {
        datom = Datom::Vector(vec![datom]);
    }
    datom
}

fn probe(mode: &str, n: usize) {
    match mode {
        "read-brackets" => {
            let text = format!("{}{}", "[".repeat(n), "]".repeat(n));
            let conceived = text.protosize().unwrap().conceive().unwrap();
            assert!(matches!(conceived.1, Datom::Vector(_)));
            drop(conceived);
        }
        "read-chain" => {
            let text = format!("{}A", "A.".repeat(n));
            let conceived = text.protosize().unwrap().conceive().unwrap();
            assert!(matches!(conceived.1, Datom::Variant(..)));
            drop(conceived);
        }
        "read-vector" => {
            let text = format!("[ {}]", "1 ".repeat(n));
            let v: Vec<i64> = Potential::from(text.as_str()).actualize().unwrap();
            assert_eq!(v.len(), n);
        }
        "write-brackets" => {
            let datom = nested(n);
            let text = Textualizable::textualize(&datom);
            assert_eq!(text.len(), 4 * n + 2);
            let Ok(delineation) = datom.protosize();
            assert_eq!(Textualizable::textualize(&delineation), text);
            drop(delineation);
            drop(datom);
        }
        "write-vector" => {
            let v: Vec<i64> = vec![1; n];
            let text = Datomic::textualize(&v);
            assert_eq!(text.len(), 2 * n + 3);
            let datom = v.conceive();
            let Ok(delineation) = datom.protosize();
            drop(delineation);
            drop(datom);
        }
        other => panic!("unknown probe {other}"),
    }
    println!("peak-kb {}", peak_kb());
}

fn run_child(mode: &str, n: usize) -> u64 {
    let exe = std::env::current_exe().unwrap();
    let output = Command::new("sh")
        .arg("-c")
        .arg("ulimit -v 2000000; exec timeout 120 \"$0\" \"$@\"")
        .arg(exe)
        .arg(mode)
        .arg(n.to_string())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{mode} {n} failed: {}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let line = stdout.lines().find(|l| l.starts_with("peak-kb ")).unwrap();
    line[8..].trim().parse().unwrap()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 3 {
        probe(&args[1], args[2].parse().unwrap());
        return;
    }
    let mut failures = 0;
    for mode in MODES {
        let peaks: Vec<u64> = SIZES.iter().map(|&n| run_child(mode, n)).collect();
        let (small, medium, large) = (peaks[0], peaks[1], peaks[2]);
        let step = medium.saturating_sub(small);
        let linear_bound = small + 15 * step + 16 * 1024;
        let ok = large <= linear_bound && large < 1024 * 1024;
        println!(
            "{} {mode}: 1000 -> {small} kB, 10000 -> {medium} kB, 100000 -> {large} kB (bound {linear_bound} kB)",
            if ok { "ok  " } else { "FAIL" }
        );
        if !ok {
            failures += 1;
        }
    }
    assert_eq!(failures, 0, "{failures} probes exceeded linear memory");
}
