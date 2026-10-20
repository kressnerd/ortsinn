fn main() {
    if let Err(error) = benchmark::cli::run() {
        eprintln!("error: {error:#}");
        std::process::exit(2);
    }
}
