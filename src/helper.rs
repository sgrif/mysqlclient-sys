use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process::Command;

// The output is wrapped in a Result to allow matching on errors.
// Returns an Iterator to the Reader of the lines of the file.
#[allow(dead_code)]
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

#[allow(dead_code)]
fn parse_version_id(path: String, substr: &str) -> u32 {
    if let Ok(lines) = read_lines(path) {
        for line in lines.flatten() {
            println!("{line}");
            if let Some(s) = line.find(substr) {
                return line
                    .get(s + substr.len()..)
                    .unwrap()
                    .trim()
                    .to_string()
                    .parse::<u32>()
                    .unwrap();
            }
        }
    }

    0
}

#[allow(dead_code)]
pub fn mysql_config(arg: &str, var_name: &str) -> Option<String> {
    let cmd_arg = if arg == "variable" {
        format!("--variable={}", var_name)
    } else {
        format!("--{arg}")
    };
    return Command::new("mysql_config")
        .arg(format!("{cmd_arg}"))
        .output()
        .into_iter()
        .filter(|output| output.status.success())
        .flat_map(|output| String::from_utf8(output.stdout).ok())
        .map(|output| output.trim().to_string())
        .next();
}

#[allow(dead_code)]
pub fn mysql_config_variable(var_name: &str) -> Option<String> {
    return mysql_config("variable", var_name);
}

#[allow(dead_code)]
pub fn get_libmysql_version_id(
    include_dir: String,
) -> (
    u32,  /* version_id */
    bool, /* is_mysql */
    bool, /* is_mariadb */
) {
    if Path::new(&format!("{include_dir}/mariadb_version.h")).exists() {
        return (
            parse_version_id(
                format!("{include_dir}/mariadb_version.h"),
                "MYSQL_VERSION_ID",
            ),
            false,
            true,
        );
    } else if Path::new(&format!("{include_dir}/mysql_version.h")).exists() {
        return (
            parse_version_id(
                format!("{include_dir}/mysql_version.h"),
                "LIBMYSQL_VERSION_ID",
            ),
            true,
            false,
        );
    } else if Path::new(&format!("{include_dir}/mysql_version.h")).exists() {
        return (
            parse_version_id(
                format!("{include_dir}/mysql_version.h"),
                "LIBMYSQL_VERSION_ID",
            ),
            true,
            false,
        );
    }

    (0, false, false)
}
