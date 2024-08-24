use core::panic;
use std::fs::File;
use std::path::Path;
use std::io::Read;

fn parse_circuit(){
    let path = Path::new("circuit.pika");
    let file = File::open(path);
    let mut contents = String::new();
    let res = file.expect("Unable to open").read_to_string(&mut contents);

    //Handle error
    match res {
        Ok(_) => {println!("Circuit analyzed");},
        Err(_) => {println!("Circuit analysis failed");},
    }

    //Remove whitespace
    let operations:Vec<String> = contents.split("\r\n").map(|op| op.chars().filter(|c| !c.is_whitespace()).collect()).collect();

    let _supported_operation = ['*','+','-','/'];
    let _supported_operation_string= "*+-/";

    operations.iter().for_each(|op| {
        let mut sp_idx = None;
        let mut idx:usize = 9999;

        let op_count = op.chars().into_iter().filter(|c| _supported_operation_string.contains(*c)).count();
        
        // println!("Total operator count in the current operation: {}",op_count);

        match op_count {
           0 => {panic!("ERROR: Missing operand in circuit");}
           1 => {
                //Find operator index
                for sp in _supported_operation{           
                    
                    sp_idx=op.find(sp);
                    match sp_idx {
                        Some(_) => {break},
                        None => {}
                    }
                }

                match sp_idx {
                    Some(_idx) => {idx=_idx},
                    None => {panic!("ERROR: Unsupported operation")}
                }

                //Use operator index to split to 3 parts ie left operand,right operand and result
                let (leftoperand, r) = op.split_at(idx);
                let right = &r[idx..];
                // println!("Right : {}",right);
                let parts:Vec<&str> = right.split("==").collect();
                let rightoperand = parts[0];
                let output = parts[parts.len()-1];

                if leftoperand.is_empty() || rightoperand.is_empty() || output.is_empty(){
                    panic!("ERROR: Invalid operands or output in circuit")
                }
                

                // println!("Left operand: {:}",leftoperand);
                // println!("Right operand: {:}",rightoperand);
                // println!("Output: {:}",output);

                let val_vec:[&str;3] = [leftoperand,rightoperand,output];

                println!("Val Vec: {:?}",val_vec);

           }
           _ => {panic!("ERROR: More than one operand in circuit");}
        }

    });

    println!("{:?}",operations);
}


fn main() {
    println!("Analyzing circuit");
    parse_circuit();
}
