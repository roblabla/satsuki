use std::{error::Error, path::PathBuf};

use argh::FromArgs;
use capstone::{
    arch::x86::{ArchMode, ArchSyntax},
    prelude::{BuildsCapstone, BuildsCapstoneSyntax},
    Capstone,
};
use satsuki::Mapping;

#[derive(FromArgs)]
/// Disassemble a function by name.
struct Args {
    /// executable file to disassemble.
    #[argh(positional)]
    executable_file: PathBuf,

    /// the function name to disassemble.
    #[argh(positional)]
    function_name: String,

    /// pdb file related to the executable.
    #[argh(option)]
    pdb_file: Option<PathBuf>,

    /// mapping TOML file related to the executable.
    #[argh(option)]
    mapping_file: PathBuf,

    /// force usage of address zero when disassembling.
    #[argh(switch)]
    force_address_zero: bool,

    /// use at&t syntax when printing assembly.
    #[argh(switch)]
    att: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Args = argh::from_env();

    if !args.executable_file.exists() {
        eprintln!("Executable not found!\n");
        std::process::exit(1);
    }

    let capstone = Capstone::new()
        .x86()
        .mode(ArchMode::Mode32)
        .syntax(if args.att {
            ArchSyntax::Att
        } else {
            ArchSyntax::Intel
        })
        .detail(true)
        .build()
        .expect("Cannot create Capstone context");

    if !args.mapping_file.exists() {
        eprintln!("Mapping not found!\n");
        std::process::exit(1);
    }

    let bin_data = std::fs::read(args.executable_file)?;
    let raw_obj = object::File::parse(&*bin_data)?;
    let raw_mapping = std::fs::read_to_string(args.mapping_file)?;
    let mapping = toml::from_str::<Mapping>(&raw_mapping)?;

    if let Some(pdb_file) = args.pdb_file {
        if !pdb_file.exists() {
            eprintln!("PDB not found!\n");
            std::process::exit(1);
        }

        let pdb_file = pdb::PDB::open(std::fs::File::open(pdb_file)?)?;

        let executable =
            satsuki::Executable::from_object_with_pdb(&raw_obj, mapping, pdb_file).unwrap();

        match executable.get_function(&args.function_name) {
            Some(function) => {
                let res = function
                    .disassemble(&capstone, args.force_address_zero)
                    .unwrap();

                println!("{}", res);
            }
            None => {
                eprintln!("Function {} not found in executable!", args.function_name);
                std::process::exit(1);
            }
        }

        Ok(())
    } else {
        let executable = satsuki::Executable::from_object_with_mapping(&raw_obj, mapping).unwrap();

        match executable.get_function(&args.function_name) {
            Some(function) => {
                let res = function
                    .disassemble(&capstone, args.force_address_zero)
                    .unwrap();

                println!("{}", res);
            }
            None => {
                eprintln!("Function {} not found in executable!", args.function_name);
                std::process::exit(1);
            }
        }

        Ok(())
    }
}
