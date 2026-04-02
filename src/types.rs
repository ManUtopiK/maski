use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Maskfile {
    #[allow(dead_code)]
    pub title: String,
    #[allow(dead_code)]
    pub description: String,
    pub commands: Vec<Command>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Command {
    pub name: String,
    pub description: String,
    #[allow(dead_code)]
    pub level: u8,
    pub script: Option<Script>,
    pub subcommands: Vec<Command>,
    pub required_args: Vec<RequiredArg>,
    pub optional_args: Vec<OptionalArg>,
    pub named_flags: Vec<NamedFlag>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Script {
    pub executor: String,
    pub source: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RequiredArg {
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OptionalArg {
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NamedFlag {
    pub name: String,
    #[allow(dead_code)]
    pub short: String,
    pub long: String,
    pub description: String,
    pub takes_value: bool,
    pub choices: Vec<String>,
    pub required: bool,
    pub validate_as_number: bool,
}
