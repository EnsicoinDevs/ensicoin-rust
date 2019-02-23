use clap::{Arg, App, SubCommand};

pub struct Args {
    pub port    : u16,
}

pub fn initiate_cli() -> Args {
    let app = create_cli_app().get_matches();

    let port = app.value_of("port").unwrap();
    match app.subcommand() {
        ("completions", Some(sub_matches)) => {
            let shell = sub_matches.value_of("SHELL").unwrap();
            create_cli_app().gen_completions_to(
                "ensicoin-rust",
                shell.parse().unwrap(),
                &mut std::io::stdout()
            );
        },
        (_, _) => ()
    }

    Args {
        port    : port.parse().unwrap()
    }
}

fn create_cli_app() -> clap::App<'static, 'static> {
    app_from_crate!()
        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .value_name("PORT")
            .help("set the port to listen to.")
            .default_value("4224")
            .validator(is_port))
        .subcommand(SubCommand::with_name("completions")
        .about("Generates completion scripts for your shell")
        .arg(Arg::with_name("SHELL")
            .required(true)
            .possible_values(&["bash", "fish", "zsh"])
            .help("The shell to generate the script for")))
}

fn is_port(value : String) -> Result<(), String> {
    let n_value : Result<u16, std::num::ParseIntError> = value.parse();
    match n_value {
        Ok(v) if v > 1024   => Ok(()),
        Ok(_)               => Err("Port value must be higher than 1024.".to_string()),
        Err(s)              => Err(s.to_string())
    }
}
