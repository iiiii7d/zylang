use std::{fs::File, io::Read, panic, process::exit};

use clap::Parser;
use color_eyre::config::HookBuilder;
use zyxt::{
    repl,
    types::{
        element::Element, errors::ZError, interpreter_data::InterpreterData, printer::StdIoPrint,
        typeobj::Type, value::Value,
    },
};

#[derive(Parser)]
#[clap(version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    subcmd: Subcmd,
    /// Enables debugging info
    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}
#[derive(Parser)]
enum Subcmd {
    /// Runs Zyxt source code
    Run(Run),
    /// Start a REPL for Zyxt
    Repl,
}
#[derive(Parser)]
struct Run {
    filename: String,
}

fn main() {
    HookBuilder::new()
        .panic_section("This shouldn't happen!\nOpen an issue on our GitHub: https://github.com/Segmential/zyxt/issues/new")
        .install().unwrap();
    let args = Args::parse();
    let verbose = args.verbose;

    match args.subcmd {
        Subcmd::Run(sargs) => {
            let filename = &sargs.filename;
            let mut content = String::new();
            match File::open(filename) {
                Ok(mut file) => {
                    file.read_to_string(&mut content).unwrap_or_else(|e| {
                        if e.to_string() == *"Is a directory (os error 21)" {
                            ZError::error_1_2(filename.to_owned())
                                .print_exit(&mut StdIoPrint(verbose))
                        } else {
                            panic!("{}", e.to_string())
                        }
                    });
                }
                Err(_) => {
                    ZError::error_1_1(filename.to_owned()).print_exit(&mut StdIoPrint(verbose))
                }
            };
            let mut sip1 = StdIoPrint(verbose);
            let mut sip2 = StdIoPrint(verbose);
            let mut typelist = InterpreterData::<Type<Element>, _>::new(&mut sip1);
            let mut i_data = InterpreterData::<Value, _>::new(&mut sip2);
            let exit_code = zyxt::interpret(
                &zyxt::compile(content, filename, &mut typelist)
                    .unwrap_or_else(|e| e.print_exit(&mut StdIoPrint(verbose))),
                &mut i_data,
            )
            .unwrap_or_else(|e| e.print_exit(&mut StdIoPrint(verbose)));
            exit(exit_code);
        }
        // TODO Compile, Interpret
        Subcmd::Repl => repl::repl(verbose),
    }
}
