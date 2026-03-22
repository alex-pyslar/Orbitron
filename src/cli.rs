// ── CLI: Backend enum, BuildOpts, help text, arg parser ──────────────────────

const VERSION: &str = env!("CARGO_PKG_VERSION");

// ── Backend ───────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
pub enum Backend { Llvm, Jvm }

impl Backend {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "llvm" => Some(Backend::Llvm),
            "jvm"  => Some(Backend::Jvm),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self { Backend::Llvm => "llvm", Backend::Jvm => "jvm" }
    }
}

// ── Build options ─────────────────────────────────────────────────────────────

pub struct BuildOpts {
    pub output:     Option<String>,
    /// CLI override; None → read from manifest / default (llvm)
    pub backend:    Option<Backend>,
    pub emit_llvm:  bool,
    pub emit_java:  bool,
    pub save_temps: bool,
    pub verbose:    bool,
}

// ── Help text ─────────────────────────────────────────────────────────────────

pub fn print_help() {
    println!(
"Orbitron {ver} — compiler for the .ot language

USAGE:
  orbitron new <name>            Create a new project
  orbitron build [options]       Build the project (searches for orbitron.toml)
  orbitron run   [options]       Build and run the project
  orbitron fmt [--write] [files] Format source code (gofmt-style)
  orbitron [options] <file.ot>   Compile a single file

OPTIONS:
  -h, --help              Show this help and exit
      --version           Show version and exit
  -o <file>               Output file name
      --backend llvm|jvm  Compilation backend (default: llvm)
      --emit-llvm         Save LLVM IR and stop (llvm backend)
      --emit-java         Save Java source and stop (jvm backend)
      --save-temps        Keep intermediate files (.ll, .s)
  -v, --verbose           Print compilation steps

BACKENDS:
  llvm   -> native binary (via llc + clang)
  jvm    -> JVM bytecode  (via javac + jar, run with: java -jar ...)
             Works on standard JVM and GraalVM JDK.
             GraalVM native image: native-image -jar <file>.jar

PROJECT CONFIG (orbitron.toml):
  [project]
  name = \"myapp\"
  version = \"0.1.0\"

  [build]
  main    = \"src/main.ot\"
  output  = \"bin/myapp\"
  backend = \"llvm\"    # or \"jvm\"

STANDARD LIBRARY:
  import \"std/math\";   # math functions (abs, max, min, gcd, ...)
  import \"std/bits\";   # bit operations (bit_count, is_pow2, ...)
  import \"std/algo\";   # algorithms (min3, max3, lerp, ipow, ...)
  import \"std/sys\";    # Linux syscalls (sys_alloc, sys_write, ...)
  import \"std/net\";    # networking (tcp_socket, tcp_connect, ...)

  stdlib/ must be next to the orbitron binary,
  or set the ORBITRON_HOME environment variable.

EXAMPLES:
  orbitron new mycalc                # create a project
  cd mycalc && orbitron build        # build (llvm)
  cd mycalc && orbitron run          # build and run
  orbitron hello.ot                  # single file (llvm)
  orbitron hello.ot --backend jvm    # single file (jvm)
  orbitron build --backend jvm       # project (jvm)
  orbitron build --emit-llvm         # generate LLVM IR only
  orbitron build -v                  # verbose output

PROJECT LAYOUT:
  myproject/
  ├── orbitron.toml
  └── src/
      ├── main.ot
      └── utils.ot

PIPELINE (llvm):
  .ot -> Lexer -> Parser -> Resolver -> AST -> CodeGen -> LLVM IR -> llc -> clang -> binary

PIPELINE (jvm):
  .ot -> Lexer -> Parser -> Resolver -> AST -> JvmCodeGen -> Main.java -> javac -> .jar",
        ver = VERSION
    );
}

// ── Argument parser ───────────────────────────────────────────────────────────

pub fn parse_build_opts(args: &[String]) -> Result<BuildOpts, String> {
    let mut output     = None;
    let mut backend    = None;
    let mut emit_llvm  = false;
    let mut emit_java  = false;
    let mut save_temps = false;
    let mut verbose    = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" => {
                i += 1;
                if i >= args.len() { return Err("-o flag requires an argument".into()); }
                output = Some(args[i].clone());
            }
            "--backend" => {
                i += 1;
                if i >= args.len() { return Err("--backend requires an argument: llvm | jvm".into()); }
                backend = Some(Backend::from_str(&args[i])
                    .ok_or_else(|| format!("Unknown backend '{}'. Use llvm or jvm", args[i]))?);
            }
            "--emit-llvm"      => emit_llvm  = true,
            "--emit-java"      => emit_java   = true,
            "--save-temps"     => save_temps  = true,
            "-v" | "--verbose" => verbose     = true,
            flag if flag.starts_with('-') => {
                return Err(format!("Unknown flag '{}'. Use -h for help", flag));
            }
            _ => {}
        }
        i += 1;
    }

    Ok(BuildOpts { output, backend, emit_llvm, emit_java, save_temps, verbose })
}
