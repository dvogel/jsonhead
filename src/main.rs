use clap::Parser;
use std::io::Write;
use utf8_read::Reader;

#[derive(Parser)]
struct Cli {
    #[clap(short, default_value_t = 1)]
    n: usize,
}

#[derive(Debug)]
enum NestedElem {
    JsonObject(usize),
    JsonArray(usize),
    JsonString(usize),
}

fn main() -> Result<(), String> {
    let args = Cli::parse();
    let mut found_count = 0;

    let mut depth_stack: Vec<NestedElem> = vec![];
    let mut pos: usize = 0;
    let mut rdr = Reader::new(std::io::stdin());
    let mut outf = std::io::stdout();
    let mut b = [0; 2];
    let mut in_escape = false;
    let mut in_string = false;
    for ch in rdr.into_iter() {
        if in_escape {
            in_escape = false;
        } else {
            match ch {
                Ok('\\') => {
                    if in_string {
                        in_escape = true;
                    } // Should this have an else clause?
                }
                Ok('{') => {
                    depth_stack.push(NestedElem::JsonObject(pos));
                }
                Ok('}') => {
                    if let Some(elem) = depth_stack.pop() {
                        match elem {
                            NestedElem::JsonObject(_) => {}
                            _ => {
                                return Err(format!(
                                    "Expected {:?} terminator but found object terminator.",
                                    elem
                                ));
                            }
                        }
                    }
                }
                Ok('[') => {
                    depth_stack.push(NestedElem::JsonArray(pos));
                }
                Ok(']') => {
                    if let Some(elem) = depth_stack.pop() {
                        match elem {
                            NestedElem::JsonArray(_) => {}
                            _ => {
                                return Err(format!(
                                    "Expected {:?} terminator but found array terminator.",
                                    elem
                                ));
                            }
                        }
                    }
                }
                Ok('"') => {
                    if in_string {
                        if let Some(elem) = depth_stack.pop() {
                            match elem {
                                NestedElem::JsonString(_) => {
                                    in_string = false;
                                }
                                _ => {
                                    return Err(format!(
                                        "Expected {:?} terminator but found double-quote character",
                                        elem
                                    ))
                                }
                            };
                        } else {
                            return Err("Run for the hills. Logic has failed us all!".to_string());
                        }
                    } else {
                        in_string = true;
                        depth_stack.push(NestedElem::JsonString(pos));
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    return Err(e.to_string());
                }
            }
        }
        pos = pos + 1;
        outf.write_all(ch.unwrap().encode_utf8(&mut b).as_bytes());

        if depth_stack.len() == 0 {
            found_count = found_count + 1;
            if found_count >= args.n {
                return Ok(());
            }
        }
    }

    match depth_stack.pop() {
        None => Ok(()),
        Some(expected) => Err(format!("Expected {:?} terminator but found EOF.", expected)),
    }
}
