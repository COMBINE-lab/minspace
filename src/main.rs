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
#[command(group(
    clap::ArgGroup::new("input")
    .required(true)
    .args(["input_fasta", "input_concat"])
))]
struct Cli {
    /// input FASTA file
    #[arg(short = 'f', long)]
    input_fasta: Option<PathBuf>,

    /// input CONCAT file
    #[arg(short = 'c', long, conflicts_with = "input_fasta")]
    input_concat: Option<PathBuf>,

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
    fmt::fmt().init();

    let is_fasta = cli.input_fasta.is_some();
    let input = cli
        .input_fasta
        .xor(cli.input_concat)
        .expect("one of FASTA or CONCAT must be provided");
    info!("input: {}", input.display());

    let (mut reader, _format) = niffler::from_path(input)?;
    let mut mins = Vec::<u64>::new();
    let mut max_token = 0_u64;

    if is_fasta {
        let mut reader = Reader::new(reader);

        let mut first_record = true;
        // loop over the input extracting records
        while let Some(record) = reader.next() {
            // if we observe more than one record, then skip the rest and notify the user
            if !first_record {
                info!(
                    "currently, only 1 input record is supported (i.e. no generalized minspace conversion); skipping subsequent records"
                );
                break;
            }
            // assume we have only one record for now
            let record = record.expect("Error reading record");
            let min_iter = MinimizerBuilder::<u64>::new()
                .minimizer_size(cli.l)
                .width(cli.w as u16)
                .canonical()
                .iter(record.seq());
            // loop over the record extracting minimizers
            for (minimizer, _position, _is_rc) in min_iter {
                max_token = max_token.max(minimizer);
                mins.push(minimizer);
            }
            first_record = false;
        }
    } else {
        let mut input_vec = Vec::new();
        let str_len = reader.read_to_end(&mut input_vec)?;
        info!("read concatenated file with string of length {}", str_len);
        let min_iter = MinimizerBuilder::<u64>::new()
            .minimizer_size(cli.l)
            .width(cli.w as u16)
            .canonical()
            .iter(&input_vec);
        // loop over the record extracting minimizers
        for (minimizer, _position, _is_rc) in min_iter {
            max_token = max_token.max(minimizer);
            mins.push(minimizer);
        }
    }

    // open the output file
    let out_file = OpenOptions::new()
        .read(false)
        .write(true)
        .create(true)
        .truncate(true)
        .open(cli.output.clone())?;
    let mut out = BufWriter::new(out_file);

    // if the length or the largest value is >= i32::MAX, then we'll have to use the
    // i64 minspace representation.
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
        // otherwise we can use a 32-bit minspace representation
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
