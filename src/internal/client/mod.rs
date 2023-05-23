pub mod args;

pub struct Command {
    pub conf_path: String,
    pub ip_src: String,
}

impl Command {
    pub fn init() -> Self {
        let cmd = args::new_cmd().get_matches();
        println!("command: {:?}", cmd);

        let mut conf_path = args::DEFAULT_CONF.to_string();
        if let Some(p) = cmd.get_one::<String>("config") {
            conf_path = p.to_owned();
        }

        let mut ip_src = args::DEFAULT_IP_FILE.to_string();
        if let Some(p) = cmd.get_one::<String>("src") {
            ip_src = p.to_owned();
        }

        Self { conf_path, ip_src }
    }
}
