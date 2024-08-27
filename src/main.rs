use core::panic;
use polynomial::Polynomial;
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn shield_brack_parser(op: &str) -> (String, String) {
    let op_arr: Vec<char> = op.chars().collect();
    let digits: String = op.chars().filter(|c: &char| c.is_numeric()).collect();
    let operand: String = op.chars().filter(|c| c.is_alphabetic()).collect();

    if op_arr[0] == '[' {
        let sub = '-';
        let ref_digits = &digits;
        let coeff = sub.to_string() + ref_digits;
        return (coeff, operand);
    } else {
        return (digits, operand);
    }
}

fn parse_circuit() -> Vec<[String; 5]> {
    let path = Path::new("circuit.pika");
    let file = File::open(path);
    let mut contents = String::new();
    let res = file.expect("Unable to open").read_to_string(&mut contents);
    let mut shield_brack: bool = false;
    let mut parsed_operations: Vec<[String; 5]> = Vec::new();

    //Handle error
    match res {
        Ok(_) => {
            println!("Analyzing circuit");
        }
        Err(_) => {
            panic!("Circuit analysis failed");
        }
    }

    //Remove whitespace
    let operations: Vec<String> = contents
        .split("\r\n")
        .filter(|o| *o != "")
        .map(|op: &str| op.chars().filter(|c: &char| !c.is_whitespace()).collect())
        .collect();
    println!("Unparsed Operations: {:?}", operations);

    let _supported_operation = ['*', '+', '-', '/'];
    let _supported_operation_string = "*+-/";

    operations.iter().for_each(|op| {
        let mut sp_idx = None;
        let idx: usize;

        let op_count = op
            .chars()
            .into_iter()
            .filter(|c| {
                //Don't count operator inside sqbracket
                if *c == '[' {
                    shield_brack = true;
                } else if *c == ']' {
                    shield_brack = false;
                }

                if !shield_brack {
                    _supported_operation_string.contains(*c)
                } else {
                    return false;
                }
            })
            .count();

        // println!("Total operator count in the current operation: {}",op_count);

        match op_count {
            0 => {
                panic!("ERROR: Missing operand in circuit");
            }
            1 => {
                //Find operator index
                for sp in _supported_operation {
                    sp_idx = op.find(sp);
                    match sp_idx {
                        Some(_) => break,
                        None => {}
                    }
                }

                match sp_idx {
                    Some(_idx) => idx = _idx,
                    None => {
                        panic!("ERROR: Unsupported operation")
                    }
                }

                //Use operator index to split to 3 parts ie left operand,right operand and result
                let (leftoperand, r) = op.split_at(idx);
                let right = &r[1..];
                let parts: Vec<&str> = right.split("==").collect();
                let rightoperand = parts[0];
                let output = parts[parts.len() - 1];

                let (lcoeff, lop) = shield_brack_parser(leftoperand);
                let (rcoeff, rop) = shield_brack_parser(rightoperand);

                //Check if any operand is empty
                if lop.is_empty() || rop.is_empty() || output.is_empty() {
                    panic!("ERROR: Invalid operands or output in circuit")
                }

                let val_vec: [String; 5] = [lcoeff, lop, rcoeff, rop, String::from(output)];
                println!("Val Vec: {:?}", val_vec);

                //Borrow value
                parsed_operations.push(val_vec);
            }
            _ => {
                panic!("ERROR: More than one operand in circuit");
            }
        }
    });

    println!("Circuit analyzed");
    return parsed_operations;
}

fn compute_op_points(parsed_operations: Vec<[String; 5]>, op_type: i32) -> Vec<Vec<[i32; 2]>> {
    let mut prev_var: &str = "0";
    let mut inner_vec: Vec<[i32; 2]> = Vec::new();
    let mut op_points_list: Vec<Vec<[i32; 2]>> = Vec::new();

    let offset = if op_type == 0 {
        0
    } else if op_type == 1 {
        2
    } else {
        4
    };

    let mut current_index: usize = 0;
    let last_index = parsed_operations.len() - 1;

    for array in &parsed_operations {
        let coeff = if &array[0 + offset] == "" {
            "1"
        } else {
            &array[0 + offset]
        };
        let operand_var: &String;

        let x = 1;
        let y: i32;

        //For output without coeff support
        if op_type == 2 {
            operand_var = &array[0 + offset];
            y = 1;
        } else {
            //For left and right operand
            operand_var = &array[1 + offset];
            y = coeff.parse().expect("Not a valid number");
        }

        //Initial
        if prev_var == "0" {
            prev_var = operand_var;

            //Push into the vector
            inner_vec.push([x, y]);
        } else if prev_var == operand_var {
            let points = [x, y];
            inner_vec.push(points);
            println!("Inner veec {:?}:", inner_vec);
        } else if prev_var != operand_var {
            //Update prev_lvar
            prev_var = operand_var;

            //Update left vec and increment index
            op_points_list.push(inner_vec.clone());

            inner_vec.clear();
            inner_vec.push([x, y])
        }

        //Save points if its the last operation
        if current_index == last_index {
            op_points_list.push(inner_vec.clone());
        }

        current_index = current_index + 1;
    }

    println!("Left operand points: {:?}", op_points_list);
    return op_points_list;
}

fn compute_op_polynomial() {}

fn main() {
    let parsed_operations = parse_circuit();
    println!("Operations: {:?}", parsed_operations);

    let left_op_points = compute_op_points(parsed_operations.clone(), 0);
    let right_op_points = compute_op_points(parsed_operations.clone(), 1);
    let ouput_op_points = compute_op_points(parsed_operations, 2);

    //Lagrange interpolation
}
