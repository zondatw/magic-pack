use clap::{ArgGroup, Parser};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(group(
    ArgGroup::new("functional")
        .required(true)
        .args(["compress", "decompress"]),
))]
struct Args {
    /// Compress flag
    #[arg(short, long)]
    compress: bool,

    /// Decompress flag
    #[arg(short, long)]
    decompress: bool,

    // file / directory input path
    #[arg(short)]
    input: String,

    // file / directory output path
    #[arg(short)]
    output: String,
}

fn main() {
    let args = Args::parse();

    if args.compress {
        println!("Compress");
    }
    if args.decompress {
        println!("Decompress");
    }

    println!("Input path: {:?}", args.input);
    println!("Output path: {:?}", args.output);

    // TODO: get file type
}
