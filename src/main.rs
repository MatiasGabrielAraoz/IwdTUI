use crossterm::{
    ExecutableCommand, cursor::{MoveTo}, event::{self, Event, KeyCode}, terminal::{Clear, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode, is_raw_mode_enabled}
};

use strip_ansi_escapes;
use std::{process::{Command, Stdio}};
use std::io::{stdout, Write};


fn main() -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(target_os = "windows"){
        return Err(Box::from("Error: iwdtui no es compatible con windows"));
    }

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;

    let options = ["scan", "get-networks", "connect"];
    let interfaces = get_interfaces()?;
    let mut interface: String = prompt_interfaces(interfaces)?;

    if interface.is_empty() {
        disable_raw_mode()?;
        panic!("NO TENÉS INTERFAZ")
    }
    let mut networks: Vec<String> = Vec::new();

    let mut selected: usize = 0;

    loop {
        stdout().execute(MoveTo(0, 0))?;
        stdout().execute(Clear(crossterm::terminal::ClearType::All))?;

        display_options(&options, selected)?;
        let mut n_i: u16 = 20;
        for network in networks.clone() {
            MoveTo(0, n_i);
            println!("{} \r\n", network);
            n_i += 1;
        }


        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Char('q') => break,
                KeyCode::Left => {
                    let interfaces = get_interfaces()?;
                    interface = prompt_interfaces(interfaces)?;
                },
                KeyCode::Down | KeyCode::Char('j') => {
                    if selected < options.len() - 1 {
                        selected += 1;
                        networks = get_networks(&interface)?;
                    }
                },
                KeyCode::Up | KeyCode::Char('k') => {
                    if selected > 0 {
                        selected -= 1;
                        networks = get_networks(&interface)?;
                    }
                },
                KeyCode::Enter | KeyCode::Right => {
                    stdout().execute(MoveTo(0, (options.len() + 1) as u16))?;
                    match selected {
                        0 => { // scan
                            scan_networks(&interface)?;
                            networks = get_networks(&interface)?;
                        },
                        1 => { // get-networks
                            networks = get_networks(&interface)?;
                        }
                        2 => { // Connect
                            prompt_connect(networks.clone(), &interface)?;
                            networks = get_networks(&interface)?;
                        }
                        _ => {

                        }
                        
                    }
                }

                _ => {}
            }

        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn prompt_connect(networks_list: Vec<String>, interface: &String) -> Result<(), Box<dyn std::error::Error>>{
    if networks_list.is_empty() {
        return Err("".into());
    }
    let mut selected = 0;

    let (_, term_height) = crossterm::terminal::size()?;
    let max_display = (term_height as usize).saturating_sub(2);

    loop {
        stdout().execute(Clear(crossterm::terminal::ClearType::All))?;
        stdout().execute(MoveTo(0,0))?;

        let start_index = if selected >= max_display {
            selected - max_display + 1
        } else {
            0
        };
        let end_index = (start_index + max_display).min(networks_list.len());

        for i in start_index..end_index {
            let network = &networks_list[i];
            if i == selected {
                print!("> {}\x1b[K\r\n", network);
            }
            else {
                print!("  {}\x1b[K\r\n", network);
            }
        }
        print!("\r\n-- (q) Volver | Redes: {}/{} --", selected + 1, networks_list.len());
        stdout().flush()?;
        

        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Char('q') | KeyCode::Left => {
                    return Ok(());
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if selected+1 < networks_list.len() {
                        selected += 1;
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if selected > 0 {
                        selected -= 1;
                    }
                }
                KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
                    if networks_list.get(selected).unwrap().contains(">") {
                        return Err("Ya estás conectado a esa red".into());

                    }
                        connect_network(networks_list.get(selected).unwrap(), interface).expect("Error connecting");
                        return Ok(())
                }
                _ => {

                }
            }
        }
    }

}

fn connect_network(network_name: &String, interface: &String) -> Result<(), Box<dyn std::error::Error>>{


    let mut cmd = Command::new("timeout");
    cmd.args(["10s", "iwctl","station", &interface, "connect", &network_name]);
    cmd.stdin(Stdio::null());

    let output = cmd.output()?;
    if output.status.success() {
        return Ok(());
    }


    let password = prompt_password()?;

    let mut cmd = Command::new("timeout");
    cmd.args(["10s", "iwctl", "station", interface, "connect", network_name, "--passphrase", &password]);
    cmd.stdin(Stdio::null());

    print!("Connecting...");
    stdout().flush()?;

    let output = cmd.output()?;
    if output.status.success() {
        scan_networks(&interface)?;
        Ok(())
    }
    else {
        Err("No se pudo conectar a la red".into())
    }

}
fn prompt_password() -> Result<String, Box<dyn std::error::Error>>{
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    stdout().execute(Clear(crossterm::terminal::ClearType::All))?;
    stdout().execute(MoveTo(0,0))?;
    print!("Insert the password: ");
    stdout().flush()?;

    let mut password: String = String::new();
    std::io::stdin().read_line(&mut password)?;

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    Ok(password.trim().to_string())
}

fn get_interfaces() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut get_interfaces_cmd = Command::new("iwctl");
    let mut interfaces_list = Vec::new();
    get_interfaces_cmd.args(["station", "list"]);
    let result =  get_interfaces_cmd.output()?;
    let clean_bytes = strip_ansi_escapes::strip(&result.stdout);
    let text  = String::from_utf8_lossy(&clean_bytes);

    // println!("DEBUG RAW OUTPUT: {:?}", text);
    for line in text.lines().skip(4) {
        if line.contains("---") || line.contains("Devices") || line.contains("Name") {
            continue;
        }

        let words: Vec<&str> = line
            .split_whitespace()
            .collect();

        if let Some(name) = words.first() {
            interfaces_list.push(name.to_string());
        }
    }
    Ok(interfaces_list)
}

fn prompt_interfaces(interfaces: Vec<String>) -> Result<String, Box<dyn std::error::Error>>{
    let mut selected = 0;
    if !is_raw_mode_enabled()?{
        panic!();
    }
    loop {
        stdout().execute(MoveTo(0, 0))?;
        stdout().execute(Clear(crossterm::terminal::ClearType::All))?;
        for (idx, option) in interfaces.iter().enumerate(){
            if idx == selected{
                print!("> {}\r\n", option);
            }
            else {
                print!("  {}\r\n", option);
            }
        }

        stdout().flush()?;

        if let Event::Key(key_event) = event::read()? {
            
            match key_event.code{
                KeyCode::Char('q') => {
                    break;
                }

                KeyCode::Up => {
                    if selected > 0{
                        selected -= 1;
                    }
                }
                KeyCode::Down => {
                    if selected + 1 < interfaces.len() {
                        selected += 1;
                    }
                }
                KeyCode::Enter => {
                    let interface = interfaces.get(selected).cloned().unwrap_or_default();
                    return Ok(interface);
                    
                }
                _ => {}
            }

        }

    }


    Ok(String::new())
}

fn scan_networks(interface: &str) -> Result<(), Box<dyn std::error::Error>>{
    let mut scan_cmd = Command::new("iwctl");
    scan_cmd.args(["station", &interface, "scan"]);
    
    scan_cmd.output()?;


    Ok(())
}

fn get_networks(interface: &str) -> Result<Vec<String>, Box<dyn std::error::Error>>{
    let mut get_networks_cmd = Command::new("iwctl");
    get_networks_cmd.args(["station", &interface, "get-networks"]);
    
    let result = get_networks_cmd.output()?;
    let text = String::from_utf8_lossy(&result.stdout);
    let networks = text
        .lines()
        .skip(4);

    let mut networks_list = Vec::new();
    for line in networks {

        let line_trimmed = line.trim();
        let line_noprefix = line_trimmed.strip_prefix("> ").unwrap_or(line_trimmed);
        let line_clean = line_noprefix.trim();

        if line_clean.is_empty() {
            continue;
        }

        let words: Vec<&str> = line_clean.split_whitespace().collect();

        if words.len() > 2 {
            let name_range = &words[0..words.len() - 2];
            let network_name = name_range.join(" ");
            networks_list.push(network_name);
        }
    }


    Ok(networks_list)
}


fn display_options(options: &[&str], selected: usize) -> Result<(), Box<dyn std::error::Error>>{
    if !is_raw_mode_enabled()?{
        enable_raw_mode()?;
    }

    print!("Iwdtui: \r\n");
    for (idx, option) in options.iter().enumerate(){
        if idx == selected{
            print!("> {}\x1b[K\r\n", option);
        }
        else {
            print!("  {}\x1b[K\r\n", option);
        }
    }
    stdout().flush()?;

    Ok(())
}
