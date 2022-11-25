#[derive(Clone, Debug)]
pub enum RunType {
    Master,
    Slave,
}

impl RunType {
    pub fn from_str(input: &str) -> RunType {
        match input.to_lowercase().as_str() {
            "master" => RunType::Master,
            _ => RunType::Slave,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Args {
    pub address: String,
    pub runType: RunType,
}

impl Args {
    pub fn handle() -> Args {
        let args: Vec<String> = std::env::args().collect();
        // TODO: handle errors in case args are not set
        Args {
            address: String::from(&args[1]),
            runType: RunType::from_str(&args[2]),
        }
    }
}
