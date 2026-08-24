use std::{
    env, fs,
    io::{self, Write},
    path::Path,
    process, time,
};

fn main() {
    let opts = match parse_args() {
        Ok(o) => o,
        Err(e) => match e {
            OptionsError::Help => help_and_exit(io::stdout(), 0),
            OptionsError::Usage => {
                eprint!("disk-size: bad usage\n\n");
                help_and_exit(io::stderr(), 1);
            }
        },
    };

    let start = match opts.track_time {
        true => Some(time::Instant::now()),
        false => None,
    };
    let bytes = match process_dir(opts.dirname) {
        Ok(r) => r,
        Err(err) => {
            eprintln!("error processing dir: {err}");
            process::exit(1);
        }
    };
    let elapsed = start.map(|t| t.elapsed());

    match opts.output {
        Output::Bytes => print!("{}B", bytes),
        Output::HumanReadable => print_human_readable(bytes),
    }

    if let Some(elapsed) = elapsed {
        println!(" ({:?})", elapsed);
    } else {
        println!();
    }
}

fn help_and_exit(mut w: impl Write, sc: i32) -> ! {
    w.write_all(
        b"usage: disk-size [dir] [-h] [-b] [-H]

flags:
  -h, --help                print this message
  -t, --time                print time it took to calculate size
  -b, --bytes               print size in bytes
  -H, --human-readable      print size in human readable format (default)
",
    )
    .expect("error writing help");
    w.flush().expect("error flushing help");
    process::exit(sc)
}

fn parse_args() -> Result<Options, OptionsError> {
    let mut output: Option<Output> = None;
    let mut dirname: Option<String> = None;
    let mut track_time = false;

    let mut parsing_flags = true;
    for arg in env::args().skip(1) {
        if parsing_flags {
            if arg == "--" {
                parsing_flags = false;
                continue;
            }

            if arg == "-h" || arg == "--help" {
                return Err(OptionsError::Help);
            }

            if arg == "-t" || arg == "--time" {
                track_time = true;
                continue;
            }

            if arg == "-b" || arg == "--bytes" {
                output = Some(Output::Bytes);
                continue;
            }

            if arg == "-H" || arg == "--human-readable" {
                output = Some(Output::HumanReadable);
                continue;
            }

            if arg.starts_with('-') {
                return Err(OptionsError::Usage);
            }
        }
        if dirname.is_some() {
            return Err(OptionsError::Usage);
        }
        dirname = Some(arg);
    }

    let dirname = dirname.unwrap_or_else(|| String::from("."));
    let output = output.unwrap_or(Output::HumanReadable);

    Ok(Options {
        dirname,
        track_time,
        output,
    })
}

enum OptionsError {
    Help,
    Usage,
}

enum Output {
    HumanReadable,
    Bytes,
}

struct Options {
    dirname: String,
    track_time: bool,
    output: Output,
}

fn print_human_readable(bytes: u64) {
    let bytes: f64 = bytes as f64;

    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    const GIB: f64 = MIB * 1024.0;

    match bytes {
        0.0..KIB => print!("{}B", bytes),
        KIB..MIB => print!("{:.2}KiB", bytes / KIB),
        MIB..GIB => print!("{:.2}MiB", bytes / MIB),
        GIB.. => print!("{:.2}GiB", bytes / GIB),
        _ => unreachable!(),
    }
}

fn process_dir<P: AsRef<Path>>(p: P) -> io::Result<u64> {
    let mut len = 0;

    let entries = fs::read_dir(p)?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                eprintln!("error reading entry: {err}");
                continue;
            }
        };

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(err) => {
                eprintln!(
                    "error reading metadata for {}: {}",
                    entry.file_name().to_string_lossy(),
                    err
                );
                continue;
            }
        };

        // TODO - option to follow symlinks
        if metadata.is_symlink() {
            continue;
        }

        if metadata.is_dir() {
            let dir_size = match process_dir(entry.path()) {
                Ok(s) => s,
                Err(err) => {
                    eprintln!(
                        "error processing dir {}: {}",
                        entry.file_name().to_string_lossy(),
                        err
                    );
                    continue;
                }
            };
            len += dir_size;
        } else {
            len += metadata.len();
        }
    }

    Ok(len)
}
