use clap::ArgMatches;

#[derive(Debug, Clone)]
pub struct Config {
    pub name: String,
    pub hostname: String,
    pub port: u16,
    pub secret_token: String,
    pub temperatures: Vec<String>,
}

pub fn parse_config(args: ArgMatches) -> Result<Config, String> {
    let name = String::from(args.value_of("name").expect("The name is not present."));
    let hostname = String::from(
        args.value_of("hostname")
            .expect("The hostname is not present"),
    );
    let port = args
        .value_of("port")
        .expect("The port is not given")
        .parse::<u16>()
        .expect("The port is invalid");
    let secret_token = String::from(
        args.value_of("secret-token")
            .expect("The token is not given"),
    );

    let temperatures: Vec<String> = args
        .values_of("temperature")
        .expect("No temperatures to simulate given")
        .map(|s| s.to_owned())
        .collect();

    Ok(Config {
        name,
        hostname,
        port,
        secret_token,
        temperatures,
    })
}
