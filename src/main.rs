//! dfits — FITS header display
//!
//! Reads FITS files and prints their headers. FITS headers are made of
//! fixed 80-byte "cards"; a header ends with a card whose first three
//! bytes are "END". A file may contain a main header followed by any
//! number of extension headers, each introduced by an XTENSION card.

use clap::Parser;
use std::fs::File;
use std::io::{self, BufReader, IsTerminal, Read, Write};
use std::process::ExitCode;

const CARD_BYTES: usize = 80;
const FITS_MAGIC: &[u8] = b"SIMPLE  =";

#[derive(Parser, Debug)]
#[command(
    name = "dfits",
    about = "Display FITS file headers",
    long_about = "Reads FITS files and prints their headers."
)]
struct Cli {
    /// Extension to print. Omit for main header only;
    /// 0 for main + all extensions; N for extension N only.
    #[arg(
        short = 'x',
        long = "extension",
        value_name = "N",
        allow_hyphen_values = true
    )]
    extension: Option<i32>,

    /// FITS files to read. With no files, reads from stdin if piped.
    #[arg(value_name = "FILE")]
    files: Vec<String>,
}

#[derive(Clone, Copy, Debug)]
enum HeaderSelection {
    MainOnly,
    MainAndAllExtensions,
    SingleExtension(u32),
}

impl HeaderSelection {
    fn from_cli(extension: Option<i32>) -> Self {
        match extension {
            None => HeaderSelection::MainOnly,
            Some(n) if n < 0 => HeaderSelection::MainOnly,
            Some(0) => HeaderSelection::MainAndAllExtensions,
            Some(n) => HeaderSelection::SingleExtension(n as u32),
        }
    }

    fn should_print_main_header(self) -> bool {
        matches!(
            self,
            HeaderSelection::MainOnly | HeaderSelection::MainAndAllExtensions
        )
    }

    fn should_scan_extensions(self) -> bool {
        matches!(
            self,
            HeaderSelection::MainAndAllExtensions | HeaderSelection::SingleExtension(_)
        )
    }

    fn should_print_extension(self, index: u32) -> bool {
        match self {
            HeaderSelection::MainAndAllExtensions => true,
            HeaderSelection::SingleExtension(target) => target == index,
            HeaderSelection::MainOnly => false,
        }
    }

    fn finished_after_extension(self, index: u32) -> bool {
        matches!(self, HeaderSelection::SingleExtension(target) if target == index)
    }
}

#[derive(Debug)]
enum DfitsError {
    Io(io::Error),
    NotFits,
    TruncatedInput,
}

impl From<io::Error> for DfitsError {
    fn from(source: io::Error) -> Self {
        DfitsError::Io(source)
    }
}

impl std::fmt::Display for DfitsError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DfitsError::Io(source) => write!(formatter, "{}", source),
            DfitsError::NotFits => write!(formatter, "not a FITS file"),
            DfitsError::TruncatedInput => write!(formatter, "error reading input"),
        }
    }
}

type Card = [u8; CARD_BYTES];

/// Strip trailing ASCII spaces from a card and return it as a string.
/// FITS cards are ASCII and space-padded to 80 bytes.
fn trim_card_padding(card: &[u8]) -> String {
    let trimmed_len = card
        .iter()
        .rposition(|&byte| byte != b' ')
        .map_or(0, |last_nonspace| last_nonspace + 1);
    String::from_utf8_lossy(&card[..trimmed_len]).into_owned()
}

/// Read exactly one 80-byte card from `reader`.
/// Returns `Ok(None)` on clean EOF (zero bytes available),
/// `Ok(Some(card))` on success, or `Err` on a short read or I/O error.
fn read_one_card<R: Read>(reader: &mut R) -> Result<Option<Card>, DfitsError> {
    let mut card = [0u8; CARD_BYTES];
    let mut bytes_read = 0;
    while bytes_read < CARD_BYTES {
        match reader.read(&mut card[bytes_read..])? {
            0 => break,
            n => bytes_read += n,
        }
    }
    match bytes_read {
        0 => Ok(None),
        CARD_BYTES => Ok(Some(card)),
        _ => Err(DfitsError::TruncatedInput),
    }
}

fn card_keyword_is(card: &Card, keyword: &[u8]) -> bool {
    card.starts_with(keyword)
}

/// Print one header: the given first card, then every card up to and
/// including the END card.
fn print_header<R: Read>(first_card: &Card, reader: &mut R) -> Result<(), DfitsError> {
    println!("{}", trim_card_padding(first_card));
    if card_keyword_is(first_card, b"END") {
        return Ok(());
    }
    while let Some(card) = read_one_card(reader)? {
        println!("{}", trim_card_padding(&card));
        if card_keyword_is(&card, b"END") {
            return Ok(());
        }
    }
    Err(DfitsError::TruncatedInput)
}

/// Consume cards until END, without printing them.
fn skip_to_end_of_header<R: Read>(reader: &mut R) -> Result<(), DfitsError> {
    while let Some(card) = read_one_card(reader)? {
        if card_keyword_is(&card, b"END") {
            return Ok(());
        }
    }
    Err(DfitsError::TruncatedInput)
}

fn dump_fits_headers<R: Read>(
    reader: &mut R,
    selection: HeaderSelection,
) -> Result<(), DfitsError> {
    // The first card of every FITS file must start with "SIMPLE  =".
    let first_card = read_one_card(reader)?.ok_or(DfitsError::TruncatedInput)?;
    if !card_keyword_is(&first_card, FITS_MAGIC) {
        return Err(DfitsError::NotFits);
    }

    if selection.should_print_main_header() {
        print_header(&first_card, reader)?;
    } else if !card_keyword_is(&first_card, b"END") {
        skip_to_end_of_header(reader)?;
    }

    if !selection.should_scan_extensions() {
        return Ok(());
    }

    // FITS data units are padded to 2880-byte (36-card) boundaries, so
    // any XTENSION card lands on an 80-byte boundary too — reading card
    // by card stays in sync.
    let mut extension_index: u32 = 0;
    loop {
        let xtension_card = loop {
            match read_one_card(reader)? {
                None => return Ok(()),
                Some(card) if card_keyword_is(&card, b"XTENSION") => break card,
                Some(_) => continue,
            }
        };

        extension_index += 1;
        if selection.should_print_extension(extension_index) {
            println!("====> xtension {} <====", extension_index);
            print_header(&xtension_card, reader)?;
        } else {
            skip_to_end_of_header(reader)?;
        }

        if selection.finished_after_extension(extension_index) {
            return Ok(());
        }
    }
}

fn process_file(path: &str, selection: HeaderSelection) -> bool {
    let file = match File::open(path) {
        Ok(handle) => handle,
        Err(open_error) => {
            eprintln!("error: cannot open file [{}]: {}", path, open_error);
            return false;
        }
    };
    println!("====> file {} (main) <====", path);
    let mut buffered = BufReader::new(file);
    report_result(dump_fits_headers(&mut buffered, selection))
}

fn report_result(result: Result<(), DfitsError>) -> bool {
    match result {
        Ok(()) => true,
        Err(error) => {
            let _ = writeln!(io::stderr(), "{}", error);
            false
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let selection = HeaderSelection::from_cli(cli.extension);

    if cli.files.is_empty() {
        if io::stdin().is_terminal() {
            eprintln!("dfits: no input files. Try --help.");
            return ExitCode::from(1);
        }
        let stdin = io::stdin();
        let mut stdin_handle = stdin.lock();
        return match dump_fits_headers(&mut stdin_handle, selection) {
            Ok(()) => ExitCode::from(0),
            Err(error) => {
                let _ = writeln!(io::stderr(), "{}", error);
                ExitCode::from(1)
            }
        };
    }

    let failure_count: u32 = cli
        .files
        .iter()
        .map(|path| if process_file(path, selection) { 0 } else { 1 })
        .sum();
    ExitCode::from(failure_count.min(255) as u8)
}
