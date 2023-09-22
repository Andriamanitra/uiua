#[cfg(not(feature = "binary"))]
compile_error!("To compile the uiua interpreter binary, you must enable the `binary` feature flag");

use std::{
    env, fs,
    io::{self, stderr, Write},
    path::{Path, PathBuf},
    process::{exit, Child, Command},
    sync::{
        atomic::{self, AtomicU64},
        mpsc::channel,
        Arc,
    },
    thread::sleep,
    time::Duration,
};

use clap::{error::ErrorKind, Parser};
use instant::Instant;
use notify::{EventKind, RecursiveMode, Watcher};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use uiua::{
    format::{format_file, FormatConfig},
    run::RunMode,
    Uiua, UiuaError, UiuaResult,
};

fn main() {
    color_backtrace::install();

    let _ = ctrlc::set_handler(|| {
        let mut child = WATCH_CHILD.lock();
        if let Some(ch) = &mut *child {
            _ = ch.kill();
            *child = None;
            println!("# Program interrupted");
            print_watching();
        } else {
            if let Ok(App::Watch) | Err(_) = App::try_parse() {
                clear_watching_with(" ", "");
            }
            exit(0)
        }
    });

    if let Err(e) = run() {
        println!("{}", e.show(true));
        exit(1);
    }
}

static WATCH_CHILD: Lazy<Mutex<Option<Child>>> = Lazy::new(Default::default);

fn run() -> UiuaResult {
    if cfg!(feature = "profile") {
        uiua::profile::run_profile();
        return Ok(());
    }
    match App::try_parse() {
        Ok(app) => {
            let config = FormatConfig::default();
            match app {
                App::Init => {
                    if let Some(path) = working_file_path() {
                        eprintln!("File already exists: {}", path.display());
                    } else {
                        fs::write("main.ua", "\"Hello, World!\"").unwrap();
                        _ = open::that("main.ua");
                    }
                }
                App::Fmt { path } => {
                    if let Some(path) = path {
                        format_file(path, &config)?;
                    } else {
                        for path in uiua_files() {
                            format_file(path, &config)?;
                        }
                    }
                }
                App::Run {
                    path,
                    no_format,
                    mode,
                    #[cfg(feature = "audio")]
                    audio_time,
                    #[cfg(feature = "audio")]
                    audio_port,
                } => {
                    if let Some(path) = path.or_else(working_file_path) {
                        if !no_format {
                            format_file(&path, &config)?;
                        }
                        let mode = mode.unwrap_or(RunMode::Normal);
                        #[cfg(feature = "audio")]
                        if let Some(time) = audio_time {
                            uiua::set_audio_stream_time(time);
                        }
                        #[cfg(feature = "audio")]
                        if let Some(port) = audio_port {
                            if let Err(e) = uiua::set_audio_stream_time_port(port) {
                                eprintln!("Failed to set audio time port: {e}");
                            }
                        }
                        let mut rt = Uiua::with_native_sys().with_mode(mode);
                        rt.load_file(path)?;
                        for value in rt.take_stack() {
                            println!("{}", value.show());
                        }
                    } else {
                        eprintln!("{NO_UA_FILE}");
                    }
                }
                App::Test { path } => {
                    if let Some(path) = path.or_else(working_file_path) {
                        format_file(&path, &config)?;
                        Uiua::with_native_sys()
                            .with_mode(RunMode::Test)
                            .load_file(path)?;
                        println!("No failures!");
                    } else {
                        eprintln!("{NO_UA_FILE}");
                        return Ok(());
                    }
                }
                App::Watch => {
                    if let Some(path) = working_file_path() {
                        _ = open::that(&path);
                        if let Err(e) = watch(&path) {
                            eprintln!("Error watching file: {e}");
                        }
                    } else {
                        eprintln!("{NO_UA_FILE}");
                    }
                }
                #[cfg(feature = "lsp")]
                App::Lsp => uiua::lsp::run_server(),
            }
        }
        Err(e) if e.kind() == ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
            if let Some(path) = working_file_path() {
                _ = open::that(&path);
                if let Err(e) = watch(&path) {
                    eprintln!("Error watching file: {e}");
                }
            } else {
                _ = e.print();
                eprintln!("\n{NO_UA_FILE}");
            }
        }
        Err(e) => _ = e.print(),
    }
    Ok(())
}

const NO_UA_FILE: &str =
    "No .ua file found nearby. Initialize one in the current directory with `uiua init`";

fn working_file_path() -> Option<PathBuf> {
    let in_src = PathBuf::from("src.main.ua");
    let main = if in_src.exists() {
        in_src
    } else {
        PathBuf::from("main.ua")
    };
    Some(if main.exists() {
        main
    } else if let Some(entry) = fs::read_dir("")
        .into_iter()
        .chain(fs::read_dir("src"))
        .flatten()
        .filter_map(Result::ok)
        .find(|entry| entry.path().extension().map_or(false, |ext| ext == "ua"))
    {
        entry.path()
    } else {
        return None;
    })
}

fn watch(open_path: &Path) -> io::Result<()> {
    let (send, recv) = channel();
    let mut watcher = notify::recommended_watcher(send).unwrap();
    watcher
        .watch(Path::new(""), RecursiveMode::Recursive)
        .unwrap();

    println!("Watching for changes... (end with ctrl+C, use `uiua help` to see options)");

    let config = FormatConfig::default();
    let audio_time = Arc::new(AtomicU64::new(0f64.to_bits()));
    let audio_time_clone = audio_time.clone();
    #[cfg(feature = "audio")]
    let (audio_time_socket, audio_time_port) = {
        let socket = std::net::UdpSocket::bind(("127.0.0.1", 0))?;
        let port = socket.local_addr()?.port();
        socket.set_nonblocking(true)?;
        (socket, port)
    };
    let run = |path: &Path| -> io::Result<()> {
        if let Some(mut child) = WATCH_CHILD.lock().take() {
            _ = child.kill();
            print_watching();
        }
        const TRIES: u8 = 10;
        for i in 0..TRIES {
            match format_file(path, &config) {
                Ok(formatted) => {
                    if formatted.is_empty() {
                        clear_watching();
                        print_watching();
                        return Ok(());
                    }
                    clear_watching();
                    let audio_time =
                        f64::from_bits(audio_time_clone.load(atomic::Ordering::Relaxed))
                            .to_string();
                    #[cfg(feature = "audio")]
                    let audio_port = audio_time_port.to_string();
                    *WATCH_CHILD.lock() = Some(
                        Command::new(env::current_exe().unwrap())
                            .arg("run")
                            .arg(path)
                            .args([
                                "--no-format",
                                "--mode",
                                "all",
                                "--audio-time",
                                &audio_time,
                                #[cfg(feature = "audio")]
                                "--audio-port",
                                #[cfg(feature = "audio")]
                                &audio_port,
                            ])
                            .spawn()
                            .unwrap(),
                    );
                    return Ok(());
                }
                Err(UiuaError::Format(..)) => sleep(Duration::from_millis((i as u64 + 1) * 10)),
                Err(e) => {
                    clear_watching();
                    println!("{}", e.show(true));
                    print_watching();
                    return Ok(());
                }
            }
        }
        println!("Failed to format file after {TRIES} tries");
        Ok(())
    };
    run(open_path)?;
    let mut last_time = Instant::now();
    loop {
        sleep(Duration::from_millis(10));
        if let Some(path) = recv
            .try_iter()
            .filter_map(Result::ok)
            .filter(|event| matches!(event.kind, EventKind::Modify(_)))
            .flat_map(|event| event.paths)
            .filter(|path| path.extension().map_or(false, |ext| ext == "ua"))
            .last()
        {
            if last_time.elapsed() > Duration::from_millis(100) {
                run(&path)?;
                last_time = Instant::now();
            }
        }
        let mut child = WATCH_CHILD.lock();
        if let Some(ch) = &mut *child {
            if ch.try_wait()?.is_some() {
                print_watching();
                *child = None;
            }
            #[cfg(feature = "audio")]
            {
                let mut buf = [0; 8];
                if audio_time_socket.recv(&mut buf).is_ok_and(|n| n == 8) {
                    let time = f64::from_be_bytes(buf);
                    audio_time.store(time.to_bits(), atomic::Ordering::Relaxed);
                }
            }
        }
    }
}

#[derive(Parser)]
enum App {
    #[clap(about = "Initialize a new main.ua file")]
    Init,
    #[clap(about = "Format and run a file")]
    Run {
        path: Option<PathBuf>,
        #[clap(long, help = "Don't format the file before running")]
        no_format: bool,
        #[clap(long, help = "Run the file in a specific mode")]
        mode: Option<RunMode>,
        #[cfg(feature = "audio")]
        #[clap(long, help = "The start time of audio streaming")]
        audio_time: Option<f64>,
        #[cfg(feature = "audio")]
        #[clap(long, help = "The port to update audio time on")]
        audio_port: Option<u16>,
    },
    #[clap(about = "Format and test a file")]
    Test { path: Option<PathBuf> },
    #[clap(about = "Run a main.ua in watch mode")]
    Watch,
    #[clap(about = "Format a uiua file or all files in the current directory")]
    Fmt { path: Option<PathBuf> },
    #[cfg(feature = "lsp")]
    #[clap(about = "Run the Language Server")]
    Lsp,
}

fn uiua_files() -> Vec<PathBuf> {
    fs::read_dir("")
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "ua"))
        .map(|entry| entry.path())
        .collect()
}

const WATCHING: &str = "watching for changes...";
fn print_watching() {
    eprint!("{}", WATCHING);
    stderr().flush().unwrap();
}
fn clear_watching() {
    clear_watching_with("―", "\n")
}

fn clear_watching_with(s: &str, end: &str) {
    print!(
        "\r{}{}",
        s.repeat(term_size::dimensions().map_or(10, |(w, _)| w)),
        end,
    );
}
