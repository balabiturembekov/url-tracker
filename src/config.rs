use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about = "Анализатор истории Safari")]
pub struct Config {
    #[arg(short, long, default_value_t = 15)]
    pub limit: usize,

    #[arg(short, long, default_value_t = 7)]
    pub days: u32,

    #[arg(short, long)]
    pub filter: Option<String>,

    #[arg(short, long, default_value = "report.html")]
    pub output: String,
}
