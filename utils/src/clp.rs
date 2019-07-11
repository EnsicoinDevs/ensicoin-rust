use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Args {
    #[structopt(short="p", long="port", default_value="4224", parse(try_from_str = "is_port"))]
    pub port    : u16,
}

pub fn args() -> Args {
    Args::from_args()
}

fn is_port(value : &str) -> Result<u16, std::num::ParseIntError> {
    let n_value : Result<u16, std::num::ParseIntError> = value.parse();
    match n_value {
        Ok(v) if v > 1024   => Ok(v),
        Ok(_)               => "Not in range".parse::<u16>(),
        Err(s)              => Err(s),
    }
}
