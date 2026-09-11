//! Native binary entry. Delegates to [`deck_gen::cli`] when the `cli` feature
//! is on and the target is not wasm; otherwise prints a short usage error.

fn main() {
    #[cfg(all(feature = "cli", not(target_arch = "wasm32")))]
    {
        if let Err(err) = deck_gen::cli::run() {
            eprintln!("{err}");
            std::process::exit(1);
        }
        return;
    }

    #[cfg(not(all(feature = "cli", not(target_arch = "wasm32"))))]
    {
        eprintln!("deck_gen CLI requires a native build with `--features cli`");
        std::process::exit(1);
    }
}
