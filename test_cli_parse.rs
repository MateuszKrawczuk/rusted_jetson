use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct TestCli {
    #[arg(long, short)]
    stats: bool,
    
    #[arg(long)]
    export: Option<String>,
    
    #[arg(long, value_name = "SPEED")]
    fan: Option<u8>,
    
    #[arg(long, value_name = "ID")]
    nvpmodel: Option<u8>,
    
    #[arg(long)]
    jetson_clocks: bool,
}

fn main() {
    println!("Testing parse_from...");
    let args: Vec<&str> = vec!["--fan", "50"];
    let cli = TestCli::try_parse_from(args);
    match cli {
        Ok(cli) => println!("Parsed: fan={:?}", cli.fan),
        Err(e) => println!("Error: {}", e),
    }
    
    println!("\nTesting parse_from with jetson_clocks...");
    let args2: Vec<&str> = vec!["--jetson-clocks"];
    let cli2 = TestCli::try_parse_from(args2);
    match cli2 {
        Ok(cli) => println!("Parsed: jetson_clocks={:?}", cli.jetson_clocks),
        Err(e) => println!("Error: {}", e),
    }
}
