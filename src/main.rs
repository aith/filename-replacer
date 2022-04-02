/*
This file replaces a string in filenames and file contents throughout a directory.
Uses a temporary location to safely edit files before writing.
Case-sensitive.
*/
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::fs::{create_dir, File, remove_dir_all};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::exit;
use clap::Parser;

static TEMP_DIR: &str = "./temp/";
// Location of existing notes.
static TARGET_DIR: &str = "../../faust/";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = "Replaces a string in files' names and contents \
throughout a directory.")]
struct Args {
    #[clap(name = "from", help = "The string to remove in the file and filenames.")]
    from: String,

    #[clap(name = "to", help = "The string to add in the file and filenames.")]
    to: String,

    #[clap(name = "is_writing", short = 'w', long = "write", help = "If provided, changes will \
    be written. If not, no changes occur.")]
    is_writing: bool,
}

fn main() -> Result<(), std::io::Error>{
    let args = Args::parse();

    let old_string = args.from;
    let new_string = args.to;

    let old_filepaths = fs::read_dir(TARGET_DIR)?
        // Turn each item into a PathBuf.
        .map(|filename|
            PathBuf::from( &filename
                .unwrap()
                .path()
            )
        )
        // Remove non-markdown files.
        .filter(|path| path.extension() == Some(&OsString::from("md")) )
        .collect::<Vec<_>>();

    // Create temporary dir to write files without overwriting original files.
    if Path::new(TEMP_DIR).exists() {
        println!("Cannot create temp dir because it exists already. Exiting without changes.");
        exit(1);
    }
    if args.is_writing {
        create_dir(TEMP_DIR).unwrap();
    }

    let mut temp_filepaths: HashMap<&PathBuf, PathBuf> = HashMap::new();

    let mut text_update_count = 0;
    for old_filepath in &old_filepaths {
        let reader = BufReader::new(File::open(&old_filepath).unwrap());

        let mut new_text = reader
            .lines()
            .into_iter()
            .map(|line| line.unwrap())
            .collect::<Vec<String>>();
        
        if new_text.iter().any(|line| line.contains(&old_string)) {
            text_update_count += 1;
            new_text = new_text
                .into_iter()
                .map(|line| line.replace(&old_string, &new_string))
                .collect::<Vec<String>>();
        }

        if old_filepath.to_str().unwrap() == "../../faust/test.str.two.md" {
            if args.is_writing {
                let mut test = File::create("test.str.two.md").unwrap();
                write!(test, "{}", new_text.join("\n")).unwrap();
            }
        }

        let temp_filepath = PathBuf::from(
            String::from(TEMP_DIR)
            + old_filepath
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
        );

        // Write contents to temp file.
        if args.is_writing {
            let mut temp_file = File::create(&temp_filepath).unwrap();
            write!(temp_file, "{}", new_text.join("\n")).unwrap();
        }

        temp_filepaths.insert(&old_filepath, temp_filepath);

    }

    // Get new filepaths.
    let mut new_filepaths: HashMap<&PathBuf, PathBuf> = HashMap::new();
    for old_filepath in &old_filepaths {
        let new_filepath = String::from(old_filepath.to_str().unwrap())
            .replace(&old_string, &new_string);

        let old_filepath_string = String::from(old_filepath
            .to_str()
            .unwrap()
        );

        // Check that the new filepath doesn't already exist so that no data is deleted.
        if old_filepath_string != new_filepath
            && Path::new(&new_filepath).exists()
            {
                eprintln!("\
                Error: Filename replacement collision occurs from & to these files: \n\
                {} \n\
                {} \n\
                To avoid data loss, migrate the data to a single file and retry. Exiting without \
                changes.",
                          old_filepath_string,
                          new_filepath
                );
                if args.is_writing {
                    remove_temp_dir(TEMP_DIR);
                }
                exit(1);
        }

        new_filepaths.insert(old_filepath, PathBuf::from(new_filepath));
    }

    // Move temp files to new file locations.
    let mut move_count = 0;
    for old_filepath in &old_filepaths {

        let temp_filepath = temp_filepaths
            .get(&old_filepath)
            .unwrap();

        let new_filepath = new_filepaths
            .get(&old_filepath)
            .unwrap();

        // TODO remove this unwrap because it occurs during file modification.
        if args.is_writing {
            fs::rename(temp_filepath, new_filepath).unwrap();
        }

        // Remove old files if their name changed.
        if old_filepath != new_filepath {
            eprintln!("Old: {}\n\
                       New: {}\n",
                      &old_filepath.to_str().unwrap(),
                      &new_filepath.to_str().unwrap());
            if args.is_writing {
                // TODO remove this unwrap because it occurs during file modification.
                fs::remove_file(&old_filepath).unwrap();
            }
            move_count += 1;
        }
    }

    if args.is_writing {
        remove_temp_dir(TEMP_DIR);
        println!("Filename replacement complete!\n\
            \t{} file texts updated.\n\
            \t{} files moved.",
                 text_update_count,
                 move_count);
    }
    else {
        println!("Debug filename replacement complete!\n\
            \t{} file texts would be updated.\n\
            \t{} files woud be moved.",
                 text_update_count,
                 move_count);
    }
    Ok(())

}

fn remove_temp_dir(temp_dir: &str) -> () {
    remove_dir_all(temp_dir).unwrap();
    if Path::new(temp_dir).exists() {
        println!("Warning: temp dir could not be deleted.");
        exit(1);
    }
}