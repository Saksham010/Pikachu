use core::panic;
use std::vec;
use ark_ec::Group;
use polynomial::Polynomial;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use ark_bn254::{G1Projective as G, Fr as ScalarField};
use ark_std::{Zero, UniformRand, ops::Mul};


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

    let mut x = 1;
    let mut y: i32;


    for array in &parsed_operations {
        let coeff = if &array[0 + offset] == "" {
            "1"
        } else {
            &array[0 + offset]
        };
        let operand_var: &String;

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

        } else if prev_var != operand_var {
            //Update prev_lvar
            prev_var = operand_var;

            //Reset x
            x = 1;

            //Update left vec and increment index
            op_points_list.push(inner_vec.clone());

            inner_vec.clear();
            inner_vec.push([x, y]);
        }

        // Increment x
        x = x+1;
        

        //Save points if its the last operation
        if current_index == last_index {
            op_points_list.push(inner_vec.clone());
        }

        current_index = current_index + 1;
    }
    println!("Operand points: {:?}", op_points_list);
    return op_points_list;
}

fn compute_op_polynomial(op_points: Vec<Vec<[i32; 2]>>) ->(Vec<Polynomial<f64>>,Polynomial<f64>) {

    let mut polynomial_array:Vec<Polynomial<f64>> = Vec::new();
    let mut final_polynomial:Polynomial<f64> = Polynomial::new(vec![0.0]);

    for points in &op_points{
        //First variable polynomial
        let mut x_point_list:Vec<i32> = Vec::new();
        let mut y_point_list:Vec<i32> = Vec::new();
    
        //Seperate x and y point list
        for point in points{
            
            let x = point[0];
            let y = point[1];
                    
            x_point_list.push(x);
            y_point_list.push(y);
            
        }

        println!("X_POINT_LIST: {:?}",x_point_list);
        println!("Y_POINT_LIST: {:?}",y_point_list);

        let x_point_list_f64:Vec<f64> = x_point_list.iter().map(|&p| p as f64).collect();
        let y_point_list_f64:Vec<f64>= y_point_list.iter().map(|&p| p as f64).collect();

        //Interpolate polynomial from those points
        let c0_polynomial = Polynomial::lagrange(&x_point_list_f64, &y_point_list_f64).unwrap();
        println!("Polynomial: {:?}",c0_polynomial);

        polynomial_array.push(c0_polynomial);

    }

    println!("Polynomial in the operand : {:?}",polynomial_array);

    //Compute final polynomial
    for poly in &polynomial_array{
        final_polynomial = final_polynomial + poly;
    }

    println!("Final polynomial: {:?}",final_polynomial);

    return (polynomial_array,final_polynomial);


}

fn main() {
    let parsed_operations = parse_circuit();
    println!("Operations: {:?}", parsed_operations);

    let left_op_points = compute_op_points(parsed_operations.clone(), 0);
    let right_op_points = compute_op_points(parsed_operations.clone(), 1);
    let ouput_op_points = compute_op_points(parsed_operations, 2);

    //Lagrange interpolation
    let (left_operand_polynomial_array,left_operand_polynomial) = compute_op_polynomial(left_op_points);
    let (right_operand_polynomial_array,right_operand_polynomial) = compute_op_polynomial(right_op_points);
    let (output_operand_polynomial_array,output_operand_polynomial) = compute_op_polynomial(ouput_op_points);

    for lop in &left_operand_polynomial_array{

        println!("Left polynomial data: {:?}",lop.data());
    }
    // Select generator 
    // let p:i64 = 21888242871839275222246405745257275088696311157297823662689037894645226208583; //Prime number for bn254

    //Sample random generator
    let mut rng = ark_std::test_rng();
    // let g = G::rand(&mut rng);

    let g = G::generator(); //Generator on the curve

    let s = ScalarField::rand(&mut rng);
    let rohl = ScalarField::rand(&mut rng);
    let rohr = ScalarField::rand(&mut rng);
    let roho = rohl * rohr;
    let alphal = ScalarField::rand(&mut rng);
    let alphar = ScalarField::rand(&mut rng);
    let alphao = ScalarField::rand(&mut rng);
    let beta = ScalarField::rand(&mut rng);
    let gamma =  ScalarField::rand(&mut rng);

    let gl = g*rohl;
    let gr = g*rohr;
    let go = g*roho;

    println!("Generator: {:?}",g);
    println!("Secret s: {:?}",s);
    println!("Rohl: {:?}",rohl);
    println!("Rohr: {:?}",rohr);
    println!("Roho: {:?}",roho);
    println!("alphal: {:?}",alphal);
    println!("alphar: {:?}",alphar);
    println!("alphao: {:?}",alphao);
    println!("beta: {:?}",beta);
    println!("gamma: {:?}",gamma);
    println!("gl: {:?}",gl);
    println!("gr: {:?}",gr);
    println!("go: {:?}",go);


    let result = Polynomial::lagrange(&[1.0,2.0,3.0], &[3.0,1.0,1.0]).unwrap();
    println!("Result: {:?}",result);





}
