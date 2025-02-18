use crate::types::{Config, MemType, Time};
use ansi_term::{self, Color::*};
pub use serde::Deserialize;

use std::iter::repeat;
use std::{fs::File, path::Path, str};
use std::{io::Read, process::Command};
use sysinfo::{ComponentExt, CpuExt, DiskExt, System, SystemExt};

impl Config {
    /// Fetches config and returns a new [Config] instance.
    ///
    /// # Panic
    /// This code should not panic under normal circumstances.

    pub fn new() -> Config {
        // Default configs.
        // These are completely ignored in case a config file is found.
        let default_config = r#"
        show_os = true
        show_hostname = true
        show_uptime = true
        show_kernel_version = true
        show_memory = true
        show_swap = true
        show_de = true
        show_colors = true
        show_cpu = true
        show_cores = true
        show_disks = true
        show_temperature = false
        colors_height = 3
        colors_width = 3
        uptime_type = "Minute"
        memory_type = "GB"
    "#;

        let config_path = std::format!(
            "{}/ru_fetch/config.toml",
            dirs::config_dir().unwrap().as_path().to_str().unwrap()
        );

        let config: Config = if Path::new(&config_path).exists() {
            let f = File::open(&config_path); // no unwrap() since other errors can occur too.
            match f {
                Ok(mut file) => {
                    let mut contents = String::new();
                    match file.read_to_string(&mut contents) {
                        Ok(_) => {}
                        Err(_) => toml::from_str(default_config).unwrap(),
                    }

                    match toml::from_str(&contents) {
                        Ok(config) => config,
                        Err(error) => {
                            println!(
                                "{}",
                                Red.bold()
                                    .paint("Error in config, falling back to default config.")
                            );
                            println!(
                                "{} {}",
                                Red.bold().paint(error.to_string()),
                                Blue.bold().paint("(line, column may differ from actual)")
                            );
                            toml::from_str(default_config).unwrap()
                        }
                    }
                }
                Err(_) => toml::from_str(default_config).unwrap(),
            }
        } else {
            toml::from_str(default_config).unwrap()
        };

        config
    }

    /// Prints the fetch results to the console.
    ///
    /// The result depends on the config file or the fallback defaults.

    pub fn print(&self, sys: &System) {
        if self.show_hostname {
            let host_name = sys.host_name();
            // Getting the user
            let user = Config::get_user();

            if let Some(host_name) = &host_name {
                println!(
                    "{}@{}",
                    Blue.bold().paint(user),
                    Blue.bold().paint(host_name)
                );
            }

            println!("{}", "-".repeat(30));
            Config::print_hostname(host_name);
        }

        if self.show_os {
            Config::print_os(&self, &sys);
        }

        if self.show_de {
            Config::print_desktop_environment();
        }

        if self.show_uptime {
            Config::print_uptime(&self, &sys);
        }

        if self.show_kernel_version {
            Config::print_kernel_ver(&sys);
        }

        if self.show_disks {
            Config::print_disks(&sys);
        }

        if self.show_cpu {
            Config::print_cpu(&self, &sys);
        }

        if self.show_memory {
            Config::print_mem(&self, &sys);
        }

        if self.show_swap {
            Config::print_swap(&self, &sys);
        }

        if self.show_temperature {
            Config::print_temps(&sys);
        }

        if self.show_colors {
            Config::print_colors(&self);
        }
    }

    fn print_hostname(host_name: Option<String>) {
        if let Some(host_name) = &host_name {
            println!("{} {}", Blue.bold().paint("Host:"), *host_name);
        }
    }

    fn print_os(&self, sys: &System) {
        let os = sys.long_os_version();

        if let Some(os) = &os {
            println!("{} {}", Blue.bold().paint("OS:"), os);
        }
    }

    fn print_uptime(&self, sys: &System) {
        match &self.uptime_type {
            Time::Second => {
                let uptime_sec = sys.uptime();
                println!("{} {:.2} sec(s)", Blue.bold().paint("Uptime: "), uptime_sec);
            }
            Time::Minute => {
                let uptime_min: f64 = sys.uptime() as f64 / 60 as f64;
                println!("{} {:.2} min(s)", Blue.bold().paint("Uptime:"), uptime_min);
            }
            Time::Hour => {
                let uptime_hour: f64 = sys.uptime() as f64 / 3600 as f64;
                println!(
                    "{} {:.2} hour(s)",
                    Blue.bold().paint("Uptime: "),
                    uptime_hour
                );
            }
        }
    }

    fn print_kernel_ver(sys: &System) {
        let kernel_ver = sys.kernel_version();
        if let Some(kernel_ver) = &kernel_ver {
            println!("{} {}", Blue.bold().paint("Kernel Version:"), *kernel_ver);
        }
    }

    fn print_cpu(&self, sys: &System) {
        let cpu_str = format!(
            "{} {}",
            Blue.bold().paint("CPU:"),
            sys.global_cpu_info().brand()
        );

        if *&self.show_cores {
            println!("{} ({})", cpu_str, sys.cpus().len());
        } else {
            println!("{}", cpu_str);
        }
    }

    fn print_disks(sys: &System) {
        for disk in sys.disks() {
            println!(
                "{}: {} ({:.2} GB / {:.2} GB)",
                Blue.bold().paint("Disk"),
                Yellow.bold().paint(disk.name().to_string_lossy()),
                (disk.total_space() - disk.available_space()) as f64 / (1024.0 * 1024.0 * 1024.0),
                disk.total_space() as f64 / (1024 * 1024 * 1024) as f64
            )
        }
    }

    fn print_mem(&self, sys: &System) {
        match &self.memory_type {
            MemType::KB => println!(
                "{} {:.2} KB / {:.2} KB",
                Blue.bold().paint("Memory:"),
                sys.used_memory() as f64 / 1e+3,
                sys.total_memory() as f64 / 1e+3
            ),
            MemType::MB => {
                println!(
                    "{} {:.2} MB / {:.2} MB",
                    Blue.bold().paint("Memory:"),
                    sys.used_memory() as f64 / 1e+6,
                    sys.total_memory() as f64 / 1e+6
                )
            }
            MemType::GB => {
                println!(
                    "{} {:.2} GB / {:.2} GB",
                    Blue.bold().paint("Memory:"),
                    sys.used_memory() as f64 / 1e+9,
                    sys.total_memory() as f64 / 1e+9
                )
            }
        }
    }

    fn print_swap(&self, sys: &System) {
        sys.global_cpu_info().brand();
        match &self.memory_type {
            MemType::KB => println!(
                "{} {:.2} KB / {:.2} KB",
                Blue.bold().paint("Swap:"),
                sys.used_swap() as f64 / 1e+3,
                sys.total_swap() as f64 / 1e+3
            ),
            MemType::MB => {
                println!(
                    "{} {:.2} MB / {:.2} MB",
                    Blue.bold().paint("Swap:"),
                    sys.used_swap() as f64 / 1e+6,
                    sys.total_swap() as f64 / 1e+6
                )
            }
            MemType::GB => {
                println!(
                    "{} {:.2} GB / {:.2} GB",
                    Blue.bold().paint("Swap:"),
                    sys.used_swap() as f64 / 1e+9,
                    sys.total_swap() as f64 / 1e+9
                )
            }
        }
    }

    fn print_colors(&self) {
        for _ in 0..self.colors_height {
            println!(
                "{}{}{}{}",
                Red.on(Red)
                    .paint(format!("{:width$}", width = &self.colors_width * 2 + 1)),
                Green
                    .on(Green)
                    .paint(format!("{:width$}", width = &self.colors_width * 2 + 1)),
                Blue.on(Blue)
                    .paint(format!("{:width$}", width = &self.colors_width * 2 + 1)),
                Yellow
                    .on(Yellow)
                    .paint(format!("{:width$}", width = &self.colors_width * 2 + 1))
            );
        }
        for _ in 0..self.colors_height {
            println!(
                "{}{}{}{}",
                Black
                    .on(Black)
                    .paint(format!("{:width$}", width = &self.colors_width * 2 + 1)),
                White
                    .on(White)
                    .paint(format!("{:width$}", width = &self.colors_width * 2 + 1)),
                Purple
                    .on(Purple)
                    .paint(format!("{:width$}", width = &self.colors_width * 2 + 1)),
                Cyan.on(Cyan)
                    .paint(format!("{:width$}", width = &self.colors_width * 2 + 1))
            );
        }
    }

    fn print_temps(sys: &System) {
        println!();
        println!("{}", Red.bold().paint("Temperature"));
        println!(
            "{}",
            Red.bold().paint(repeat('-').take(20).collect::<String>())
        );

        for component in sys.components() {
            println!(
                "{}: {}°C",
                Blue.bold().paint(component.label()),
                component.temperature()
            );
        }
        println!();
    }

    fn get_user() -> String {
        let mut user_out = if cfg!(target_os = "windows") || cfg!(target_os = "linux") {
            // linux, windows
            Command::new("whoami").output().unwrap()
        } else {
            // darwin(mac)
            Command::new("id -un").output().expect("none")
        };
        let user: String = if (str::from_utf8(&user_out.stdout).unwrap()).ends_with("\n") {
            user_out.stdout.pop();
            str::from_utf8(&user_out.stdout).unwrap().to_string()
        } else {
            str::from_utf8(&user_out.stdout).unwrap().to_string()
        };
        user
    }

    /// --------------- Linux only --------------------
    ///
    /// Gets the current desktop enviroment. The information might not be 100% accurate.
    /// Skips if nothing useful is found.
    fn print_desktop_environment() {
        if cfg!(target_os = "linux") {
            // linux
            match option_env!("DESKTOP_SESSION") {
                Some(val) => {
                    let de_str = format!("{} {}", Blue.bold().paint("DE:"), val);

                    println!("{}", de_str);
                }
                None => {}
            }
        } else {
            {}
        };
    }
}
