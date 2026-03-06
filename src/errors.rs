/// Sets the panic-hook to be customized color-eyre `panic_hook`.
///
/// Additionally the panic handler prints different information
/// based on debug / release builds.
pub fn init() -> color_eyre::Result<()> {
    let (_, eyre_hook) = color_eyre::config::HookBuilder::default()
        .panic_section(format!(
            "This is a bug. Please report it at {}",
            env!("CARGO_PKG_REPOSITORY")
        ))
        .capture_span_trace_by_default(false)
        .display_location_section(false)
        .display_env_section(false)
        .into_hooks();

    eyre_hook.install()?;
    std::panic::set_hook(Box::new(move |panic_info| {
        //TODO: exit from terminal if the app is in a terminal

        // in release mode, do human_panic printing
        #[cfg(not(debug_assertions))]
        {
            use human_panic::{handle_dump, metadata, print_msg};
            let metadata = metadata!();
            let file_path = handle_dump(&metadata, panic_info);
            print_msg(file_path, &metadata)
                .expect("human-panic: printing error message to console failed");
            eprintln!("{}", panic_hook.panic_report(panic_info));
        }

        // in debug mode do better panic printing
        #[cfg(debug_assertions)]
        {
            better_panic::Settings::auto()
                .most_recent_first(false)
                .lineno_suffix(true)
                .verbosity(better_panic::Verbosity::Full)
                .create_panic_handler()(panic_info);
        }

        // 1 = failure
        std::process::exit(1)
    }));

    Ok(())
}

/// Akin to `dbg!` macro, except that it generates `tracing::Event`s instead
/// of printing to standard error.
///
/// Can customize level by providing a `tracing::Level`, but its debug by default.
#[macro_export]
macro_rules! trace_dbg {
    (target: $target:expr, level: $level:expr, $ex:expr) => {
        {
            match $ex {
                value => {
                    tracing::event!(target: $target, $level, ?value, stringify!($ex));
                    value
                }
            }
        }
    };
    (level: $level:expr, $ex:expr) => {
        trace_dbg!(target: module_path!(), level: $level, $ex)
    };

    (target: $target:expr, $ex:expr) => {
        trace_dbg!(target: $target, level: tracing::Level::DEBUG, $ex)
    };
    ($ex:expr) => {
        trace_dbg!(level: tracing::Level::DEBUG, $ex)
    };
}
