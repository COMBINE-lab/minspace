use clap::Parser;
use minimizer_iter::MinimizerBuilder;
use seq_io::fasta::{Reader, Record};
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::fmt;
use zerocopy::IntoBytes;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// input file
    #[arg(short, long)]
    input: PathBuf,

    /// output file
    #[arg(short, long)]
    output: PathBuf,

    /// window length
    #[arg(short, long, default_value_t = 31)]
    w: usize,

    /// minimizer length
    #[arg(short, long, default_value_t = 10)]
    l: usize,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let input = cli.input;
    fmt::fmt().init();

    info!("input: {}", input.display());
    let mut reader = Reader::from_path(input).unwrap();

    let mut mins = Vec::<u64>::new();
    let mut max_token = 0_u64;
    while let Some(record) = reader.next() {
        let record = record.expect("Error reading record");
        let min_iter = MinimizerBuilder::<u64>::new()
            .minimizer_size(10)
            .width(31)
            .canonical()
            .iter(record.seq());

        for (minimizer, _position, _is_rc) in min_iter {
            max_token = max_token.max(minimizer);
            mins.push(minimizer);
        }
    }

    let out_file = OpenOptions::new()
        .read(false)
        .write(true)
        .create(true)
        .truncate(true)
        .open(cli.output.clone())?;
    let mut out = BufWriter::new(out_file);

    if mins.len() >= (i32::MAX as usize) || max_token >= (i32::MAX as u64) {
        info!(
            "length of minimizer string = {}, maximum token = {}",
            mins.len(),
            max_token
        );
        info!(
            "writing output in 64-bit (u64) array to {}",
            cli.output.display()
        );
        let num = mins.len();
        out.write_all(&num.to_le_bytes())?;
        out.write_all(&max_token.to_le_bytes())?;
        out.write_all(mins.as_bytes())?;
    } else {
        info!(
            "length of minimizer string = {}, maximum token = {}",
            mins.len(),
            max_token
        );
        info!(
            "writing output in 32-bit (u32) array to {}",
            cli.output.display()
        );
        let num = mins.len();
        out.write_all(&num.to_le_bytes())?;
        out.write_all(&max_token.to_le_bytes())?;
        let small_mins = mins.iter().map(|x| *x as u32).collect::<Vec<u32>>();
        out.write_all(small_mins.as_bytes())?;
    }
    Ok(())
}
