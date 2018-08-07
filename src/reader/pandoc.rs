extern crate serde_json;
use self::serde_json::{Value, Error};

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::Command;

pub fn read_pandoc(file_path: &Path) -> Result<Vec<String>, Error> {
    let mut pd_str = String::new();
    let json_str: String = match file_path.extension().unwrap().to_string_lossy().as_ref() {
        "ast" | "json" => {
            File::open(file_path).expect("file not found!").read_to_string(&mut pd_str)
                .expect("file read error");
            pd_str
        },
        "txt" | "markdown" | "md" => {
            let result = Command::new("pandoc").arg("-f").arg("markdown").arg("-t").arg("json").arg(file_path).output()
                .expect("Failed to execute pandoc!");
            if result.status.success() {
                String::from_utf8_lossy(&result.stdout).into_owned()
            } else {
                panic!(result.stderr)
            }
        },
        _ => panic!("Inputs should be either json or markdown!")
    };
    let tokens: Value = serde_json::from_str(&json_str)?;
    let mut output: Vec<String> = Vec::new();
    if let Value::Array(ref root) = tokens["blocks"] {
        for section in root.iter() {
            if let Value::Array(ref section_content) = section["c"] {
                for paragraph in section_content.iter() {
                    if let Value::Object(ref para_content) = paragraph {
                        if para_content["t"] == "Cite" {
                            if let Value::Array(ref token_content) = para_content["c"][0] {
                                for citation in token_content.into_iter() {
                                    if let Value::String(ref citation_content) = citation["citationId"] {
                                        output.push(citation_content.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    return Ok(output);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_read_pandoc() {
        let citations = read_pandoc(Path::new("test/data/extract_test.txt")).unwrap();
        assert_eq!(citations, vec!["ehrhart2016", "kriaucionis2004", "dragich2007", "kerr2012", "itoh2012", "kerr2012", "kerr2012", "dragich2007"])
    }
}
